use tokio::time::{sleep, Duration};
use tokio_serial::SerialPortBuilderExt;

use robots_drv::{Driver, Result};
use robots_lib::Cmd;

#[tokio::main]
async fn main() -> Result<()> {
    let mut uart_port = serialport::new("/dev/ttyUSB0", 115_200).open_native_async()?;
    uart_port.set_exclusive(false)?;

    let mut driver = Driver::new(uart_port);
    let sender = driver.sender();
    let receiver = driver.receiver();

    tokio::spawn(async move { driver.run().await });
    tokio::spawn(async move {
        while let Ok(cmd) = receiver.recv().await {
            println!("received {cmd:?}");
        }
    });

    for _ in 0..7 {
        for cmd in [Cmd::Get, Cmd::Ping, Cmd::Pong] {
            println!("sending {cmd:?}...");
            sender.send(cmd).await?;
            sleep(Duration::from_millis(500)).await;
        }
    }

    Ok(())
}
