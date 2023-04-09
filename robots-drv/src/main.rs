use tokio::time::{sleep, Duration};
use tokio_serial::SerialPortBuilderExt;

use robots_drv::Drv;
use robots_lib::Cmd;

#[tokio::main]
async fn main() -> tokio_serial::Result<()> {
    let mut uart_port = serialport::new("/dev/ttyUSB0", 115_200).open_native_async()?;
    uart_port.set_exclusive(false)?;

    let mut drv = Drv::new(uart_port);
    let sender = drv.sender();
    let receiver = drv.receiver();

    tokio::spawn(async move { drv.run().await });
    tokio::spawn(async move {
        while let Ok(cmd) = receiver.recv().await {
            println!("received {cmd:?}");
        }
    });

    for _ in 0..7 {
        for cmd in [Cmd::Get, Cmd::Ping, Cmd::Pong] {
            println!("sending {cmd:?}...");
            sender.send(cmd).await.unwrap();
            sleep(Duration::from_millis(500)).await;
        }
    }

    Ok(())
}
