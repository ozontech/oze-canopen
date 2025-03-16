use oze_canopen::{
    self,
    error::CoError,
    interface::{CanOpenInterface, SEND_TIMOUT},
    nmt,
    proto::nmt::{NmtCommand, NmtCommandSpecifier},
    sync,
    transmitter::TxPacket,
};
use std::time::Duration;
use tokio::{signal::ctrl_c, time::timeout};

async fn send_pdo(interface: &CanOpenInterface) -> Result<(), CoError> {
    interface
        .tx
        .send_timeout(
            TxPacket {
                cob_id: 0x180,
                data: vec![1, 2, 3, 4, 5, 6, 7, 8],
            },
            Duration::from_millis(SEND_TIMOUT),
        )
        .await?;
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let (interface, _) = oze_canopen::canopen::start(String::from("vcan0"), Some(100000));

    let nmt_server = nmt::Server::start(interface.clone());
    let sync_server = sync::Server::start(interface.clone());
    sync_server.set_period(Some(Duration::from_micros(1)));
    nmt_server.set_command_period(
        Duration::from_micros(1),
        Some(NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 0)),
    );
    loop {
        timeout(Duration::from_millis(500), ctrl_c())
            .await
            .unwrap()
            .unwrap();
        send_pdo(&interface).await.unwrap();
    }
}
