use crate::{
    error::CoError,
    interface::{CanOpenInfo, Connection},
    proto::CobId,
};
use socketcan::{tokio::CanSocket, CanDataFrame, CanFrame, EmbeddedFrame, StandardId};
use std::{sync::Arc, time::Duration};
use tokio::{
    signal::ctrl_c,
    sync::{mpsc, Mutex, Notify},
    time::sleep,
};

const TRANSMITTER_CHANNEL_CAPACITY: usize = 128;

/// Represents a CANopen Transmit Packet
#[derive(Debug, Clone)]
pub struct TxPacket {
    pub cob_id: CobId,
    pub data: Vec<u8>,
}

impl TxPacket {
    /// Creates a new TxPacket with given cob_id and data
    pub fn new(cob_id: u16, data: &[u8]) -> Result<Self, CoError> {
        Ok(Self {
            cob_id,
            data: Vec::from(data),
        })
    }
}

/// CANopen Transmitter for handling CAN communication
pub(crate) struct Transmitter {
    channel: mpsc::Receiver<TxPacket>,
    connection: Arc<Mutex<Connection>>,
    info: Arc<Mutex<CanOpenInfo>>,
    close: Arc<Mutex<bool>>,
    pub reconnect: Arc<Notify>,
    pub channel_sender: mpsc::Sender<TxPacket>,
}

impl Transmitter {
    /// Creates a new Transmitter
    pub(crate) fn new(
        connection: Arc<Mutex<Connection>>,
        info: Arc<Mutex<CanOpenInfo>>,
        close: Arc<Mutex<bool>>,
    ) -> Self {
        let (snd, rcv) = mpsc::channel(TRANSMITTER_CHANNEL_CAPACITY);
        Self {
            info,
            close,
            connection,
            channel: rcv,
            channel_sender: snd,
            reconnect: Arc::new(Notify::new()),
        }
    }

    /// Attempts to run the transmitter loop
    async fn try_run(&mut self) -> Result<(), CoError> {
        let sock = CanSocket::open(&self.connection.lock().await.can_name.clone())?;
        self.info.lock().await.transmitter_socket = true;
        loop {
            if *self.close.lock().await {
                return Ok(());
            }

            let rx = tokio::select! {
                Some(rx) = self.channel.recv() => rx,
                _ = self.reconnect.notified() => return Err(CoError::Close),
                _ = ctrl_c() => return Err(CoError::Close),
            };

            let Some(id) = StandardId::new(rx.cob_id) else {
                continue;
            };

            let id = socketcan::Id::Standard(id);
            let frame = CanFrame::Data(CanDataFrame::new(id, &rx.data).ok_or(
                CoError::FrameError(format!("Transmit frame create error: {rx:?}")),
            )?);
            sock.write_frame(frame).await?;
        }
    }

    /// Runs the transmitter loop
    pub async fn run(&mut self) {
        loop {
            self.info.lock().await.transmitter_socket = false;
            let _ = self.try_run().await;
            self.info.lock().await.transmitter_socket = false;
            if *self.close.lock().await {
                return;
            }
            tokio::select! {
                _ = sleep(Duration::from_millis(100)) => {},
                _ = ctrl_c() => return,
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use tokio::sync::Mutex as TokioMutex;

    #[tokio::test]
    async fn test_transmitter_initialization() {
        let connection = Arc::new(TokioMutex::new(Connection {
            can_name: "not_existing_can".to_owned(),
            bitrate: None,
        }));
        let info = Arc::new(TokioMutex::new(CanOpenInfo::default()));
        let close = Arc::new(TokioMutex::new(false));
        let mut transmitter = Transmitter::new(connection.clone(), info.clone(), close.clone());

        assert_eq!(transmitter.channel.capacity(), TRANSMITTER_CHANNEL_CAPACITY);

        let rec = transmitter.reconnect.clone();
        let run = transmitter.run();

        close.lock().await.clone_from(&true);
        rec.notify_waiters();

        run.await;
    }
}
