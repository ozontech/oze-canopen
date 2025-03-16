use crate::{canopen::RxMessage, sdo_client::SdoClient, transmitter::TxPacket};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::{broadcast, mpsc, Mutex};

/// Timeout duration for sending messages.
pub const SEND_TIMOUT: u64 = 20;

/// Struct representing CANopen information.
#[derive(Debug, Default, Clone)]
pub struct CanOpenInfo {
    /// Number of received bits.
    pub rx_bits: usize,
    /// Status of the transmitter socket.
    pub transmitter_socket: bool,
    /// Status of the receiver socket.
    pub receiver_socket: bool,
}

/// Struct representing a CAN connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// Name of the CAN interface.
    pub can_name: String,
    /// Bitrate of the CAN interface.
    pub bitrate: Option<u32>,
}

/// Struct representing a CANopen interface.
pub struct CanOpenInterface {
    /// Connection details.
    pub connection: Arc<Mutex<Connection>>,
    /// Transmitter channel.
    pub tx: mpsc::Sender<TxPacket>,
    /// Receiver channel.
    pub rx: broadcast::Receiver<RxMessage>,
    /// Map of SDO clients.
    pub sdo_clients: HashMap<u8, Arc<Mutex<SdoClient>>>,
    /// CANopen information.
    pub info: Arc<Mutex<CanOpenInfo>>,
    /// CANopen information.
    pub(crate) close: Arc<Mutex<bool>>,
}

impl CanOpenInterface {
    /// Retrieves an SDO client for a given node ID.
    ///
    /// # Arguments
    ///
    /// * `node_id` - The ID of the node.
    ///
    /// # Returns
    ///
    /// An `Option` containing an `Arc<Mutex<SdoClient>>` if found, otherwise `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = can_interface.get_sdo_client(2).unwrap();
    /// client.lock().await.download(0x1801, 2, &[1u8]).await;
    /// ```
    pub fn get_sdo_client(&self, node_id: u8) -> Option<Arc<Mutex<SdoClient>>> {
        let v = self.sdo_clients.get(&node_id)?;
        Some(v.clone())
    }

    pub async fn close() {}
}

impl Clone for CanOpenInterface {
    /// Clones the `CanOpenInterface`.
    fn clone(&self) -> Self {
        Self {
            connection: self.connection.clone(),
            tx: self.tx.clone(),
            rx: self.rx.resubscribe(),
            sdo_clients: self.sdo_clients.clone(),
            info: self.info.clone(),
            close: self.close.clone(),
        }
    }
}
