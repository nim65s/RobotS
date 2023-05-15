use tokio::time::{interval, Duration};

use robots_drv::{driver, Cmd, Result, RX, TX};

#[tokio::main]
async fn main() -> Result<()> {
    let uart_port = serialport::new("/dev/ttyUSB0", 115_200);

    driver(uart_port)?;
    let tx = TX.clone();
    let mut rx = RX.clone();

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            println!("received {cmd:?}");
        }
    });

    let mut ten_hz = interval(Duration::from_millis(100));

    for hue in 0..255 {
        let cmd = Cmd::Hue(hue);
        ten_hz.tick().await;
        println!("sending {cmd:?}...");
        tx.send(&cmd).await?;
    }

    Ok(())
}
