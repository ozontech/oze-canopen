extern crate oze_canopen;

#[cfg(test)]
mod tests {
    use super::*;

    use oze_canopen::{
        canopen, nmt,
        proto::nmt::{NmtCommand, NmtCommandSpecifier},
    };
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_canopen_interface_initialization() {
        let (interface, mut handles) = canopen::start(String::from("vcan0"), Some(100000));

        let res = interface
            .send_nmt(NmtCommand::new(NmtCommandSpecifier::StartRemoteNode, 0))
            .await;
        assert!(res.is_ok());

        let res = interface
            .send_nmt_command(NmtCommandSpecifier::StartRemoteNode)
            .await;
        assert!(res.is_ok());

        sleep(Duration::from_millis(300)).await;
        handles.close_and_join().await;
    }

    #[tokio::test]
    async fn test_sdo_client_initialization() {
        let (interface, mut handles) = canopen::start(String::from("vcan0"), None);
        assert!(interface.sdo_clients.contains_key(&2));
        assert!(interface.sdo_clients.contains_key(&127));
        sleep(Duration::from_millis(300)).await;
        handles.close_and_join().await;
    }

    #[tokio::test]
    async fn test_sdo_client_send_and_wait() {
        let (interface, mut handles) = canopen::start(String::from("vcan0"), None);

        sleep(Duration::from_millis(300)).await;

        let sdo_client_mtx = interface.get_sdo_client(4).unwrap();
        let mut sdo_client = sdo_client_mtx.lock().await;

        let result = sdo_client.upload(0x2110, 0x01).await.unwrap();
        assert_eq!(result.len(), 4);

        let result = sdo_client.upload(0x2110, 0x0).await.unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], 16);

        let result = sdo_client.upload(0x2120, 0x02).await.unwrap();
        assert_eq!(result.len(), 8);

        let result = sdo_client.upload(0x2121, 0x02).await.unwrap();
        assert_eq!(result.len(), 110);

        sleep(Duration::from_millis(300)).await;

        handles.close_and_join().await;
    }

    #[tokio::test]
    async fn test_sdo_client_download_and_wait() {
        let (interface, mut handles) = canopen::start(String::from("vcan0"), None);
        let sdo_client = interface.get_sdo_client(4).unwrap();

        // Mock sending and receiving SDO request/response
        // This requires setting up a mock CAN socket or simulating the behavior

        sleep(Duration::from_millis(300)).await;
        sdo_client
            .lock()
            .await
            .download(0x2110, 0x01, &[0x1, 0x2, 0x3, 0x4])
            .await
            .unwrap();

        sdo_client
            .lock()
            .await
            .download(0x2120, 0x02, &[0x1, 0x2, 0x3, 0x4, 0x1, 0x2, 0x3, 0x4])
            .await
            .unwrap();

        sleep(Duration::from_millis(1000)).await;

        handles.close_and_join().await;
    }

    #[tokio::test]
    async fn test_canopen_nmt_server() {
        sleep(Duration::from_millis(300)).await;
        let (interface, mut handles) = canopen::start(String::from("vcan0"), None);

        let n = nmt::Server::start(interface.clone());

        n.set_command_period(Duration::from_millis(100), None);
        n.set_command_period(
            Duration::from_millis(50),
            Some(NmtCommand::new(NmtCommandSpecifier::EnterPreOperational, 0)),
        );

        sleep(Duration::from_millis(300)).await;

        handles.close_and_join().await;
    }
}
