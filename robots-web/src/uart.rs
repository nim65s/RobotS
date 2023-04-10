use crate::{
    error::Result,
    queues::{RX, TX},
};

pub async fn serve() -> Result<()> {
    let uart_port = serialport::new("/dev/ttyUSB0", 115_200);
    let (tx, rx) = robots_drv::driver(uart_port)?;

    loop {
        tokio::select! {
            cmd = rx.recv() => RX.0.send(cmd?).await?,
            cmd = TX.1.recv() => tx.send(cmd?).await?,
        }
    }
}
