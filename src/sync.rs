use crate::{
    error::CoError,
    interface::{CanOpenInterface, SEND_TIMOUT},
    transmitter::TxPacket,
};
use std::time::Duration;
use tokio::{sync::watch, time::sleep};

pub struct Server {
    control: watch::Sender<Option<Duration>>,
}

impl Server {
    pub fn set_period(&self, period: Option<Duration>) {
        let _ = self.control.send(period);
    }

    pub fn start(interface: CanOpenInterface) -> Server {
        let (snd, rcv) = watch::channel::<Option<Duration>>(None);
        tokio::spawn(async move {
            Self::task(&interface, rcv).await;
        });

        Server { control: snd }
    }

    async fn task(interface: &CanOpenInterface, mut rcv: watch::Receiver<Option<Duration>>) {
        let mut control;
        loop {
            control = *rcv.borrow();
            if let Some(period) = control {
                _ = interface.send_sync().await;

                tokio::select! {
                    _ = sleep(period) => {},
                    _ = rcv.changed() => {},
                }
            } else {
                let _ = rcv.changed().await;
            }
        }
    }
}

impl CanOpenInterface {
    pub async fn send_sync(&self) -> Result<(), CoError> {
        self.tx
            .send_timeout(
                TxPacket {
                    cob_id: 0x080,
                    data: Vec::new(),
                },
                Duration::from_millis(SEND_TIMOUT),
            )
            .await?;
        Ok(())
    }
}
