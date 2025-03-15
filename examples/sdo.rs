extern crate oze_canopen;

use oze_canopen::{error::CoError, interface::CanOpenInterface};

async fn run(interface: &CanOpenInterface) -> Result<(), CoError> {
    let s = interface.get_sdo_client(4).unwrap();

    let dat = s.lock().await.upload(0x1800, 0).await?;
    println!("Register value: {:?}", dat);

    s.lock().await.download(0x1801, 2, &[1u8]).await?;

    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() {
    let (interface, mut handles) = oze_canopen::canopen::start(String::from("vcan0"), Some(100000));
    for _ in 0..10 {
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        run(&interface).await.unwrap();
    }
    handles.close_and_join().await;
}
