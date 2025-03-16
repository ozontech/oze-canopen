use std::time::Duration;
use oze_canopen::{
    canopen::{start, JoinHandles},
    interface::{CanOpenInterface, SEND_TIMOUT},
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
    transmitter::TxPacket,
};
use pyo3::{exceptions::PyRuntimeError, prelude::*};

/// Main CANopen interface handling communication with CAN network
/// 
/// Provides methods for SDO communication, SYNC management, NMT control,
/// and raw CAN message transmission/reception.
#[pyclass]
pub struct OzeCO {
    interface: CanOpenInterface,
    _handle: JoinHandles,
    sync_server: oze_canopen::sync::Server,
    nmt_server: oze_canopen::nmt::Server,
}

/// Represents a received CAN message
#[pyclass]
pub struct Msg {
    /// The data payload of the CAN message
    pub data: Vec<u8>,
    /// CAN Identifier (COB-ID) of the message
    pub cob: u16,
}

/// Network Management (NMT) commands for node control
#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, PartialEq)]
pub enum NmtCmd {
    /// Start the remote node - transitions node to Operational state
    StartRemoteNode = 0x01,

    /// Stop the remote node - transitions node to Stopped state
    StopRemoteNode = 0x02,

    /// Enter pre-operational state - node remains initialized but doesn't process PDOs
    EnterPreOperational = 0x80,

    /// Reset the node - performs full hardware reset of the node
    ResetNode = 0x81,

    /// Reset communication - resets communication parameters of the node
    ResetCommunication = 0x82,
}

#[pymethods]
impl Msg {
    /// Get the data payload of the message
    #[getter]
    pub fn data(&self) -> Vec<u8> {
        self.data.clone()
    }

    /// Get the COB-ID of the message
    #[getter]
    pub fn cob(&self) -> u16 {
        self.cob
    }
}

#[pymethods]
impl OzeCO {
    /// Initialize the CANopen interface
    ///
    /// # Arguments
    /// * `name` - CAN interface name (e.g. "can0")
    /// * `bitrate` - Optional bitrate in bits/sec. Uses default if not specified
    ///
    /// # Returns
    /// OzeCO instance wrapped in a Python awaitable
    #[staticmethod]
    #[pyo3(signature = (name, bitrate=None))]
    pub fn start(py: Python, name: String, bitrate: Option<u32>) -> Bound<'_, PyAny> {
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let (interface, _handle) = start(name, bitrate);
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(OzeCO {
                sync_server: oze_canopen::sync::Server::start(interface.clone()),
                nmt_server: oze_canopen::nmt::Server::start(interface.clone()),
                interface,
                _handle,
            })
        })
        .unwrap()
    }

    /// Read data from a node's object dictionary using SDO upload
    ///
    /// # Arguments
    /// * `node_id` - Target node ID (1-127)
    /// * `index` - Object dictionary index
    /// * `subindex` - Object dictionary subindex
    ///
    /// # Returns
    /// Awaitable that resolves to the read data as bytes
    /// 
    /// # Raises
    /// PyRuntimeError if communication fails or timeout occurs
    pub fn sdo_upload<'a>(
        &mut self,
        py: Python<'a>,
        node_id: u8,
        index: u16,
        subindex: u8,
    ) -> PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        let client = self.interface.sdo_clients.get(&node_id).unwrap().clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut a = client.lock().await;
            a.upload(index, subindex)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("{e:?}")))
        })
    }

    /// Write data to a node's object dictionary using SDO download
    ///
    /// # Arguments
    /// * `node_id` - Target node ID (1-127)
    /// * `index` - Object dictionary index
    /// * `subindex` - Object dictionary subindex
    /// * `data` - Data to write as bytes
    ///
    /// # Returns
    /// Awaitable that resolves when write completes
    /// 
    /// # Raises
    /// PyRuntimeError if communication fails or timeout occurs
    pub fn sdo_download<'a>(
        &mut self,
        py: Python<'a>,
        node_id: u8,
        index: u16,
        subindex: u8,
        data: &[u8],
    ) -> PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        let client = self.interface.sdo_clients.get(&node_id).unwrap().clone();
        let data = Vec::from(data);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let mut a = client.lock().await;
            a.download(index, subindex, &data)
                .await
                .map_err(|e| PyErr::new::<PyRuntimeError, _>(format!("{e:?}")))
        })
    }

    /// Send a raw CAN message
    ///
    /// # Arguments
    /// * `cob_id` - CAN Identifier (COB-ID)
    /// * `data` - Payload data as bytes (max 8 bytes)
    ///
    /// # Returns
    /// Awaitable that resolves when message is sent
    pub fn send<'a>(
        &mut self,
        py: Python<'a>,
        cob_id: u16,
        data: &[u8],
    ) -> PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        let interface = self.interface.clone();
        let data = Vec::from(data);
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            interface
                .tx
                .send_timeout(
                    TxPacket { cob_id, data },
                    Duration::from_millis(SEND_TIMOUT),
                )
                .await
                .unwrap();
            Ok(())
        })
    }

    /// Receive a CAN message (blocking)
    ///
    /// # Returns
    /// Received message or blocks until one is available
    pub fn recv_blocking(&mut self) -> Msg {
        let msg = self.interface.rx.blocking_recv().unwrap();
        let d = &msg.data[0..msg.dlc];
        Msg {
            data: d.to_vec(),
            cob: msg.cob_id,
        }
    }

    /// Send an NMT command to a specific node
    ///
    /// # Arguments
    /// * `cmd` - NMT command to send
    /// * `node_id` - Target node ID (0 for broadcast)
    ///
    /// # Returns
    /// Awaitable that resolves when command is sent
    pub fn nmt_send<'a>(
        &mut self,
        py: Python<'a>,
        cmd: NmtCmd,
        node_id: u8,
    ) -> PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        let cmd = NmtCommand::new(
            match cmd {
                NmtCmd::EnterPreOperational => NmtCommandSpecifier::EnterPreOperational,
                NmtCmd::ResetCommunication => NmtCommandSpecifier::ResetCommunication,
                NmtCmd::ResetNode => NmtCommandSpecifier::ResetNode,
                NmtCmd::StartRemoteNode => NmtCommandSpecifier::StartRemoteNode,
                NmtCmd::StopRemoteNode => NmtCommandSpecifier::StopRemoteNode,
            },
            node_id,
        );

        let interface = self.interface.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            interface.send_nmt(cmd).await.unwrap();
            Ok(())
        })
    }

    /// Send a SYNC message
    ///
    /// # Returns
    /// Awaitable that resolves when SYNC is sent
    pub fn sync_send<'a>(&mut self, py: Python<'a>) -> PyResult<pyo3::Bound<'a, pyo3::PyAny>> {
        let interface = self.interface.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            interface.send_sync().await.unwrap();
            Ok(())
        })
    }

    /// Configure periodic SYNC message transmission
    ///
    /// # Arguments
    /// * `period_ms` - Transmission period in milliseconds. None stops periodic SYNC
    #[pyo3(signature = (period_ms=None))]
    pub fn sync_set_period(&mut self, period_ms: Option<u64>) {
        self.sync_server
            .set_period(period_ms.map(Duration::from_millis));
    }

    /// Configure periodic NMT command transmission
    ///
    /// # Arguments
    /// * `period_ms` - Transmission period in milliseconds
    /// * `cmd` - Optional NMT command to send periodically. None stops periodic transmission
    /// * `node_id` - Target node ID for command (required if cmd is specified)
    #[pyo3(signature = (period_ms, cmd=None, node_id=None))]
    pub fn nmt_set_period(&mut self, period_ms: u64, cmd: Option<NmtCmd>, node_id: Option<u8>) {
        let node_id = node_id.unwrap_or(0);
        let cmd = cmd.map(|m| {
            NmtCommand::new(
                match m {
                    NmtCmd::EnterPreOperational => NmtCommandSpecifier::EnterPreOperational,
                    NmtCmd::ResetCommunication => NmtCommandSpecifier::ResetCommunication,
                    NmtCmd::ResetNode => NmtCommandSpecifier::ResetNode,
                    NmtCmd::StartRemoteNode => NmtCommandSpecifier::StartRemoteNode,
                    NmtCmd::StopRemoteNode => NmtCommandSpecifier::StopRemoteNode,
                },
                node_id,
            )
        });

        self.nmt_server
            .set_command_period(Duration::from_millis(period_ms), cmd);
    }
}