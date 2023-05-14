use tokio::time::{interval, Duration};

use robots_drv::{driver, Cmd, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let uart_port = serialport::new("/dev/ttyUSB0", 115_200);

    let (tx, mut rx) = driver(uart_port)?;

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            println!("received {cmd:?}");
        }
    });

    let mut two_hz = interval(Duration::from_millis(500));

    for _ in 0..7 {
        for cmd in [Cmd::Get, Cmd::Ping, Cmd::Pong] {
            two_hz.tick().await;
            println!("sending {cmd:?}...");
            tx.send(&cmd).await?;
        }
    }

    Ok(())
}
