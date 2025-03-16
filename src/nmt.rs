use crate::{
    error::CoError,
    interface::{CanOpenInterface, SEND_TIMOUT},
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
    transmitter::TxPacket,
};
use binrw::BinWrite;
use std::time::Duration;
use tokio::{sync::watch, time::sleep};

/// A control struct that holds the period duration and an optional NMT command.
#[derive(Clone)]
struct Control {
    period: Duration,
    command: Option<NmtCommand>,
}

/// A server struct responsible for managing and sending NMT commands.
pub struct Server {
    control: watch::Sender<Control>,
}

impl Server {
    /// Sets the period and command for sending NMT commands.
    ///
    /// # Arguments
    ///
    /// * `period` - The period duration for sending the command.
    /// * `command` - The optional NMT command to be sent.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::Server;
    /// use std::time::Duration;
    /// use oze_canopen::proto::nmt::{NmtCommand, NmtCommandSpecifier};
    ///
    /// let server = Server::start(interface);
    /// server.set_command_period(Duration::from_secs(1), Some(NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 0)));
    /// ```
    pub fn set_command_period(&self, period: Duration, command: Option<NmtCommand>) {
        let _ = self.control.send(Control { period, command });
    }

    /// Starts the server with the given CANopen interface.
    ///
    /// # Arguments
    ///
    /// * `interface` - The CANopen interface to be used by the server.
    ///
    /// # Returns
    ///
    /// A new `Server` instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::Server;
    ///
    /// let interface = oze_canopen::start_interface();
    /// let server = Server::start(interface);
    /// ```
    pub fn start(interface: CanOpenInterface) -> Server {
        let (snd, rcv) = watch::channel::<Control>(Control {
            period: Duration::MAX,
            command: None,
        });
        tokio::spawn(async move {
            Self::task(&interface, rcv).await;
        });

        Server { control: snd }
    }

    /// The main task loop for the server, responsible for sending NMT commands based on the control settings.
    ///
    /// # Arguments
    ///
    /// * `interface` - The CANopen interface to be used.
    /// * `rcv` - The watch receiver for control settings.
    async fn task(interface: &CanOpenInterface, mut rcv: watch::Receiver<Control>) {
        let mut control;
        loop {
            control = rcv.borrow().clone();
            if let Some(cmd) = control.command {
                tokio::select! {
                    _ = sleep(control.period) => {},
                    _ = rcv.changed() => {},
                }

                _ = interface.send_nmt(cmd).await;
            } else {
                let _ = rcv.changed().await;
            }
        }
    }
}

impl CanOpenInterface {
    /// Sends an NMT command through the CANopen interface.
    ///
    /// # Arguments
    ///
    /// * `nmt` - The NMT command to be sent.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::CanOpenInterface;
    /// use oze_canopen::proto::nmt::{NmtCommand, NmtCommandSpecifier};
    ///
    /// let interface = oze_canopen::start_interface();
    /// let result = interface.send_nmt(NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 0)).await;
    /// ```
    pub async fn send_nmt(&self, nmt: NmtCommand) -> Result<(), CoError> {
        let mut writer = binrw::io::Cursor::new(Vec::new());
        nmt.write(&mut writer)?;
        self.tx
            .send_timeout(
                TxPacket {
                    cob_id: 0x000,
                    data: writer.into_inner(),
                },
                Duration::from_millis(SEND_TIMOUT),
            )
            .await?;
        Ok(())
    }

    /// Sends an NMT command through the CANopen interface using a command specifier.
    ///
    /// # Arguments
    ///
    /// * `nmt_command` - The NMT command specifier to be used.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    ///
    /// # Examples
    ///
    /// ```
    /// use oze_canopen::CanOpenInterface;
    /// use oze_canopen::proto::nmt::NmtCommandSpecifier;
    ///
    /// let interface = oze_canopen::start_interface();
    /// let result = interface.send_nmt_command(NmtCommandSpecifier::StartRemoteNode).await;
    /// ```
    pub async fn send_nmt_command(&self, nmt_command: NmtCommandSpecifier) -> Result<(), CoError> {
        self.send_nmt(NmtCommand::new(nmt_command, 0)).await
    }
}
