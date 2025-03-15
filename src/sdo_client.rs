use crate::{
    canopen::{RxMessageParsed, RxMessageType},
    error::CoError,
    interface::CanOpenInterface,
    proto::sdo::{ResponseData, SdoRequest, SdoResponse, UploadResponseData},
    transmitter::TxPacket,
};
use binrw::{BinRead, BinWrite};
use std::{sync::Arc, time::Duration};
use tokio::{signal::ctrl_c, sync::Mutex, time::timeout};

/// Enum representing an SDO (Service Data Object) message.
#[derive(Clone)]
pub enum SdoMessage {}

/// Struct representing an SDO client.
///
/// The SDO client handles communication with a specific CANopen node, sending
/// and receiving SDO messages to perform various operations such as uploading
/// and downloading data.
#[derive(Clone)]
pub struct SdoClient {
    interface: CanOpenInterface,
    node_id: u8,
    timeout: Duration,
    retry_count: usize,
}

/// Default timeout for SDO communication in milliseconds.
pub const SDO_TIMEOUT_DEFAULT: u64 = 20;

/// Default number of retries for SDO communication.
pub const SDO_RETRY_COUNT_DEFAULT: usize = 3;

impl SdoClient {
    /// Creates a new SdoClient.
    ///
    /// ! Do not use this function in your code, you need to call `CanOpenInterface::get_sdo_client(node_id)` instead.
    ///
    /// # Arguments
    ///
    /// * `interface` - A CANopen interface.
    /// * `node_id` - The node ID of the CANopen device to communicate with.
    pub(crate) fn new(interface: CanOpenInterface, node_id: u8) -> Self {
        Self {
            interface,
            node_id,
            timeout: Duration::from_millis(SDO_TIMEOUT_DEFAULT),
            retry_count: SDO_RETRY_COUNT_DEFAULT,
        }
    }

    /// Sends an SDO request and waits for a response.
    ///
    /// # Arguments
    ///
    /// * `request` - The SDO request to send.
    ///
    /// # Returns
    ///
    /// * `Result<ServerResponse, CoError>` - The response from the server, or an error.
    async fn send_and_wait(&mut self, request: &SdoRequest) -> Result<SdoResponse, CoError> {
        // Serialize the request
        let mut writer = binrw::io::Cursor::new(Vec::new());
        request.write(&mut writer)?;

        let send_result = self.interface.tx.send_timeout(
            TxPacket::new(0x600 + u16::from(self.node_id), &writer.into_inner())?,
            self.timeout,
        );

        tokio::select! {
            _ = ctrl_c() => return Err(CoError::TxPackerTimeout),
            send_result = send_result => send_result?,
        };

        let started = tokio::time::Instant::now();
        while started.elapsed() < self.timeout {
            let res = timeout(self.timeout, self.interface.rx.recv());

            let res = tokio::select! {
                _ = ctrl_c() => return Err(CoError::TxPackerTimeout),
                res = res => res?,
            };

            let Ok(rx) = res else {
                continue;
            };

            let rx = RxMessageParsed::new(rx);
            let Some(node_id) = rx.parsed_node_id else {
                continue;
            };

            if rx.parsed_type == RxMessageType::SdoTx && node_id == self.node_id {
                let mut m = binrw::io::Cursor::new(rx.msg.data);
                let a = SdoResponse::read(&mut m)?;
                return Ok(a);
            }
        }

        Err(CoError::SdoRetryError(self.retry_count))
    }

    /// Retries sending an SDO request and waiting for a response, up to a specified number of retries.
    ///
    /// # Arguments
    ///
    /// * `request` - The SDO request to send.
    ///
    /// # Returns
    ///
    /// * `Result<ServerResponse, CoError>` - The response from the server, or an error.
    async fn retry_send_and_wait(&mut self, request: SdoRequest) -> Result<SdoResponse, CoError> {
        let mut res: Result<SdoResponse, CoError> = Err(CoError::SdoRetryError(self.retry_count));
        for _ in 0..self.retry_count {
            res = self.send_and_wait(&request).await;
            match res {
                Ok(_) => return res,
                _ => continue,
            }
        }
        res
    }

    /// Uploads data from the specified index and subindex.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the data to upload.
    /// * `subindex` - The subindex of the data to upload.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<u8>, CoError>` - The uploaded data, or an error.
    pub async fn upload(&mut self, index: u16, subindex: u8) -> Result<Vec<u8>, CoError> {
        let resp = self
            .retry_send_and_wait(SdoRequest::initiate_upload(index, subindex))
            .await?;

        let ResponseData::Upload(init) = resp.resp else {
            return Err(CoError::SdoWrongAnswer(format!(
                "initiate_upload expected InitiateUpload but: {resp:?}"
            )));
        };

        // Handle expedited respone
        match init.data {
            // e = 1, s = 1: d contains the data of length 4-n to be uploaded
            UploadResponseData::DataExpedited(d) => return Ok(d),
            // e = 1, s = 0: d contains unspecified number of bytes to be uploaded. Not supported
            UploadResponseData::ExpeditedWithoutSize => {
                return Err(CoError::SdoWrongAnswer(format!(
                    "resp EXPEDITED=1 but SIZE_SPECIFIED=1. Not supported: {:?}",
                    resp.cmd
                )))
            }
            // e = 1, s = 0: d contains unspecified number of bytes to be uploaded. Not supported
            UploadResponseData::E0S0NotSupported => {
                return Err(CoError::SdoWrongAnswer(format!(
                    "resp EXPEDITED=0 but SIZE_SPECIFIED=0. Not supported: {:?}",
                    resp.cmd
                )))
            }
            // will handle below
            UploadResponseData::Size(_) => {}
        };

        // Continue with not expedited data
        let UploadResponseData::Size(size) = init.data else {
            return Err(CoError::SdoWrongAnswer(format!(
                "Unknown response: {:?}",
                resp.cmd
            )));
        };

        let size: usize = size.try_into().unwrap();

        let mut toggle = false;
        let mut data: Vec<u8> = Vec::new(); // output data
        loop {
            let resp = self
                .retry_send_and_wait(SdoRequest::upload_segment(toggle))
                .await?;
            toggle = !toggle;

            let ResponseData::UploadSegment(seg_resp) = resp.resp else {
                return Err(CoError::SdoWrongAnswer(format!(
                    "upload_segment: expected UploadSegment but: {resp:?}"
                )));
            };

            data.extend_from_slice(&seg_resp.data);
            if data.len() >= size {
                data.resize(size, 0);
                return Ok(data);
            }
        }
    }

    /// Downloads expedited data to the specified index and subindex.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to download to.
    /// * `subindex` - The subindex to download to.
    /// * `data` - The data to download.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoError>` - Result indicating success or failure.
    async fn download_expedited(
        &mut self,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> Result<(), CoError> {
        let resp = self
            .retry_send_and_wait(SdoRequest::initiate_download_expedited(
                index, subindex, data,
            ))
            .await?;
        let ResponseData::Download(_) = resp.resp else {
            return Err(CoError::SdoWrongAnswer(format!(
                "initiate_download_expedited: expected InitiateDownload but: {resp:?}"
            )));
        };

        Ok(())
    }

    /// Downloads segmented data to the specified index and subindex.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to download to.
    /// * `subindex` - The subindex to download to.
    /// * `data` - The data to download.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoError>` - Result indicating success or failure.
    async fn download_segmented(
        &mut self,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> Result<(), CoError> {
        let resp = self
            .retry_send_and_wait(SdoRequest::initiate_download(index, subindex, data.len()))
            .await?;
        let ResponseData::Download(_) = resp.resp else {
            return Err(CoError::SdoWrongAnswer(format!(
                "initiate_download: expected InitiateDownload but: {resp:?}"
            )));
        };

        let mut toggle = false;
        let data = data.chunks(7);
        for chunk in data {
            let resp = self
                .retry_send_and_wait(SdoRequest::download_segment(toggle, chunk))
                .await?;
            toggle = !toggle;

            let ResponseData::DownloadSegment(_) = resp.resp else {
                return Err(CoError::SdoWrongAnswer(format!(
                    "download_segment: expected DownloadSegment but: {resp:?}"
                )));
            };
        }

        Ok(())
    }

    /// Downloads data to the specified index and subindex.
    ///
    /// # Arguments
    ///
    /// * `index` - The index to download to.
    /// * `subindex` - The subindex to download to.
    /// * `data` - The data to download.
    ///
    /// # Returns
    ///
    /// * `Result<(), CoError>` - Result indicating success or failure.
    pub async fn download(&mut self, index: u16, subindex: u8, data: &[u8]) -> Result<(), CoError> {
        if data.len() <= 4 {
            self.download_expedited(index, subindex, data).await
        } else {
            self.download_segmented(index, subindex, data).await
        }
    }
}

/// Creates SDO clients for all possible node IDs and inserts them into the CANopen interface.
///
/// # Arguments
///
/// * `interface` - The CANopen interface to create the clients for.
pub(crate) fn create_all_clients(interface: &mut CanOpenInterface) {
    for node_id in 2..=127 {
        interface.sdo_clients.insert(
            node_id,
            Arc::new(Mutex::new(SdoClient::new(interface.clone(), node_id))),
        );
    }
}
