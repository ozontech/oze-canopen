use crate::interface::Connection;
pub use crate::message::*;
use crate::{error::CoError, interface::CanOpenInfo};
use futures_util::stream::StreamExt;
use socketcan::{tokio::CanSocket, CanFrame, EmbeddedFrame};
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;
use tokio::{
    signal::ctrl_c,
    sync::{broadcast, Mutex},
    time::sleep,
};

const RECEIVER_CHANNEL_CAPACITY: usize = 128;

/// The `Receiver` struct is responsible for receiving CAN messages and handling reconnection logic.
pub(crate) struct Receiver {
    connection: Arc<Mutex<Connection>>,
    info: Arc<Mutex<CanOpenInfo>>,
    close: Arc<Mutex<bool>>,
    pub channel: broadcast::Sender<RxMessage>,
    pub reconnect: Arc<Notify>,
}

impl Receiver {
    /// Creates a new `Receiver` instance.
    ///
    /// # Arguments
    ///
    /// * `connection` - An `Arc<Mutex<Connection>>` to the CAN connection.
    /// * `info` - An `Arc<Mutex<CanOpenInfo>>` for sharing CANopen information.
    ///
    /// # Returns
    ///
    /// A new `Receiver` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::Receiver;
    /// use std::sync::Arc;
    /// use tokio::sync::Mutex;
    ///
    /// let connection = Arc::new(Mutex::new(oze_canopen::Connection::new()));
    /// let info = Arc::new(Mutex::new(oze_canopen::CanOpenInfo::default()));
    /// let receiver = Receiver::new(connection, info);
    /// ```
    pub(crate) fn new(
        connection: Arc<Mutex<Connection>>,
        info: Arc<Mutex<CanOpenInfo>>,
        close: Arc<Mutex<bool>>,
    ) -> Self {
        Self {
            connection,
            close,
            info,
            reconnect: Arc::new(Notify::new()),
            channel: broadcast::Sender::new(RECEIVER_CHANNEL_CAPACITY),
        }
    }

    /// The main run loop for the receiver, handles reconnections and incoming messages.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::Receiver;
    ///
    /// async fn start_receiver(mut receiver: Receiver) {
    ///     receiver.run().await;
    /// }
    /// ```
    pub(crate) async fn run(&mut self) {
        loop {
            self.info.lock().await.receiver_socket = false;
            let _ = self.try_run().await;
            self.info.lock().await.receiver_socket = false;
            if *self.close.lock().await {
                return;
            }

            tokio::select! {
                _ = sleep(Duration::from_millis(100)) => {},
                _ = ctrl_c() => return,
            };
        }
    }

    async fn try_run(&mut self) -> Result<(), CoError> {
        let mut sock = CanSocket::open(&self.connection.lock().await.can_name.clone())?;
        loop {
            if *self.close.lock().await {
                return Ok(());
            }

            let rcv = tokio::select! {
                _ = ctrl_c() => return Err(CoError::Close),
                _ = self.reconnect.notified() => return Err(CoError::Close),
                Some(rcv) = sock.next() => {rcv}
            };

            let CanFrame::Data(p) = rcv? else {
                eprintln!("EEE not data");
                continue;
            };

            self.info.lock().await.rx_bits += p.dlc() * 8 + 46;
            let Some(data) = RxMessage::from(p) else {
                eprintln!("EEE RX node data");
                continue;
            };

            self.info.lock().await.receiver_socket = true;
            let _ = self.channel.send(data);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Arc;

    use tokio::sync::Mutex as TokioMutex;

    #[tokio::test]
    async fn test_receiver_initialization() {
        let connection = Arc::new(TokioMutex::new(Connection {
            can_name: "not_existing_can".to_owned(),
            bitrate: None,
        }));
        let info = Arc::new(TokioMutex::new(CanOpenInfo::default()));
        let close = Arc::new(TokioMutex::new(false));
        let mut receiver = Receiver::new(connection.clone(), info.clone(), close.clone());

        assert!(receiver.channel.is_empty());

        let rec = receiver.reconnect.clone();

        let run = receiver.run();

        close.lock().await.clone_from(&true);
        rec.notify_waiters();

        run.await;
    }
}
