use tokio::time::{sleep, Duration};

use robots_drv::{serve, Result};
use robots_lib::Cmd;

#[tokio::main]
async fn main() -> Result<()> {
    let uart_port = serialport::new("/dev/ttyUSB0", 115_200);

    let (sender, receiver) = serve(uart_port)?;

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
