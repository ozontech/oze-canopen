use crate::connection_monitor::ConnectionMonitor;
use crate::interface::Connection;
use crate::interface::{CanOpenInfo, CanOpenInterface};
use crate::receiver::Receiver;
use crate::sdo_client;
use crate::transmitter::Transmitter;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;

pub use crate::close::JoinHandles;
pub use crate::message::*;
pub use crate::message_parsed::*;

/// Starts a CANopen interface with the specified CAN interface name and optional bitrate.
///
/// This function sets up the connection, receiver, and transmitter, and spawns asynchronous
/// tasks to monitor the connection, and handle incoming and outgoing messages. It also
/// initializes all SDO clients for the interface.
///
/// # Arguments
///
/// * `can_name` - The name of the CAN interface (e.g., "can0").
/// * `bitrate` - An optional bitrate for the CAN interface. If bitrate will be Some, then on connect interface will go down, then bitrate setm then interface will go up.
///
/// # Returns
///
/// A `CanOpenInterface` instance that can be used to interact with the CANopen network.
///
/// # Examples
///
/// ```
/// use oze_canopen::canopen;
///
/// let (interface, mut handles) = canopen::start(String::from("can0"), Some(100000));
/// // Do something
/// handles.close_and_join().await;
/// ```
pub fn start(can_name: String, bitrate: Option<u32>) -> (CanOpenInterface, JoinHandles) {
    let connection = Arc::new(Mutex::new(Connection { can_name, bitrate }));
    let close = Arc::new(Mutex::new(false));
    let mut join_handles = JoinHandles {
        close_flag: close.clone(),
        handles: Vec::new(),
    };
    let info = Arc::new(Mutex::new(CanOpenInfo::default()));
    let mut receiver = Receiver::new(connection.clone(), info.clone(), close.clone());
    let mut transmitter = Transmitter::new(connection.clone(), info.clone(), close.clone());

    let mut interface = CanOpenInterface {
        connection: connection.clone(),
        tx: transmitter.channel_sender.clone(),
        rx: receiver.channel.subscribe(),
        sdo_clients: HashMap::new(),
        info,
        close,
    };

    let mut monitor = ConnectionMonitor {
        co: interface.clone(),
        transmitter_reconnect: transmitter.reconnect.clone(),
        receiver_reconnect: receiver.reconnect.clone(),
    };

    join_handles
        .handles
        .push(tokio::spawn(async move { receiver.run().await }));
    join_handles
        .handles
        .push(tokio::spawn(async move { transmitter.run().await }));
    join_handles
        .handles
        .push(tokio::spawn(async move { monitor.run().await }));

    sdo_client::create_all_clients(&mut interface);

    (interface, join_handles)
}

#[cfg(test)]
mod tests {
    use super::start;

    #[tokio::test]
    async fn test_canopen_interface_initialization() {
        let (interface, mut handles) = start(String::from("not existing can"), Some(125000));

        {
            let connection = interface.connection.lock().await;
            assert_eq!(connection.can_name, "not existing can");
            assert_eq!(connection.bitrate, Some(125000));
        }

        handles.close_and_join().await;
    }
}
