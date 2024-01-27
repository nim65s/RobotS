use tokio::time::{interval, Duration};

use robots_drv::{driver, get_port, Cmd, RX, TX, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let rgb = option_env!("ROBOTS_RGB").unwrap_or("false") == "true";
    let uart_port = serialport::new(get_port()?, 115_200);

    driver(uart_port)?;
    let tx = TX.clone();
    let mut rx = RX.clone();

    tokio::spawn(async move {
        while let Some(cmd) = rx.recv().await {
            println!("received {cmd:?}");
        }
    });

    let mut cmd = Cmd::Ping;
    let mut led = true;
    println!("sending {cmd:?}...");
    tx.send(&cmd).await?;

    if rgb {
        let mut ten_hz = interval(Duration::from_millis(100));
        for hue in 0..=255 {
            ten_hz.tick().await;
            println!("sending {cmd:?}...");
            tx.send(&cmd).await?;
            cmd = Cmd::Hue(hue);
        }
    } else {
        let mut hz = interval(Duration::from_secs(1));
        for _ in 0..30 {
            hz.tick().await;
            println!("sending {cmd:?}...");
            tx.send(&cmd).await?;
            led = !led;
            cmd = Cmd::Led(led);
        }
    }

    Ok(())
}
