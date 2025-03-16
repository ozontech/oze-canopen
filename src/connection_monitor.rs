use crate::interface::CanOpenInterface;
use futures_util::FutureExt;
use socketcan::CanInterface;
use std::{sync::Arc, time::Duration};
use tokio::sync::Notify;
use tokio::{signal::ctrl_c, time::sleep};

/// Monitors the connection status of the CANopen interface and handles reconnections.
pub(crate) struct ConnectionMonitor {
    /// The CANopen interface being monitored.
    pub co: CanOpenInterface,
    /// Notification for reconnecting the transmitter.
    pub transmitter_reconnect: Arc<Notify>,
    /// Notification for reconnecting the receiver.
    pub receiver_reconnect: Arc<Notify>,
}

impl ConnectionMonitor {
    /// Runs the connection monitor, continuously checking the status of the CAN interface
    /// and reconnecting if necessary.
    pub async fn run(&mut self) {
        loop {
            if *self.co.close.lock().await {
                self.transmitter_reconnect.notify_one();
                self.receiver_reconnect.notify_one();
                return;
            }

            if ctrl_c().now_or_never().is_some() {
                return;
            }

            let current_connection = self.co.connection.lock().await.clone();
            let Ok(interface) = CanInterface::open(&current_connection.can_name.clone()) else {
                tokio::select! {
                    _ = ctrl_c() => return,
                    _ = sleep(Duration::from_millis(100)) => {},
                };
                continue;
            };

            println!("INTERFACE connected");
            self.interface_setup(&interface).await;

            self.transmitter_reconnect.notify_one();
            self.receiver_reconnect.notify_one();
            loop {
                if *self.co.close.lock().await {
                    self.transmitter_reconnect.notify_one();
                    self.receiver_reconnect.notify_one();
                    return;
                }

                if interface.state().is_err() {
                    println!("STATE ERROR -> reconnect: {:?}", interface.state());
                    break;
                }

                if *self.co.connection.lock().await != current_connection {
                    println!("CONNECTION INTERFACE CHANGED -> reconnect");
                    break;
                }

                tokio::select! {
                    _ = ctrl_c() => return,
                    _ = sleep(Duration::from_millis(100)) => {},
                };
            }
            self.transmitter_reconnect.notify_one();
            self.receiver_reconnect.notify_one();
        }
    }

    async fn interface_setup_bitrate(&self, interface: &CanInterface, bitrate: u32) {
        let Ok(current_bitrate) = interface.bit_rate() else {
            return;
        };

        if current_bitrate.is_none() || current_bitrate.is_some_and(|b| bitrate != b) {
            if let Err(res) = interface.bring_down() {
                println!("INTERFACE DOWN {res:?}");
            }
            if let Err(res) = interface.set_bitrate(bitrate, 875) {
                println!("INTERFACE BITRATE {res:?}");
            }

            println!("INTERFACE set bitrate {bitrate}");
        }
    }

    /// Sets up the CAN interface with the appropriate bitrate and brings it up.
    ///
    /// # Arguments
    ///
    /// * `interface` - The CAN interface to set up.
    async fn interface_setup(&self, interface: &CanInterface) {
        if let Some(bitrate) = self.co.connection.lock().await.bitrate {
            self.interface_setup_bitrate(interface, bitrate).await;
        }

        if let Err(res) = interface.bring_up() {
            println!("INTERFACE UP {res:?}");
        }

        println!("INTERFACE set UP");
    }
}
