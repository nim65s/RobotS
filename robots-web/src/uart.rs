use crate::{
    error::Result,
    queues::{RX, TX},
};

use robots_drv::{Receiver, Sender};

async fn forward_rx(rx: Receiver) -> Result<()> {
    println!("forward tx");
    loop {
        let cmd = rx.recv().await?;
        println!("hey rx {cmd:?}");
        RX.0.send(cmd).await?;
    }
}

async fn forward_tx(tx: Sender) -> Result<()> {
    println!("forward tx");
    loop {
        let cmd = TX.1.recv().await?;
        println!("hey tx {cmd:?}");
        tx.send(cmd).await?;
    }
}

pub async fn serve() -> Result<()> {
    println!("uart start");
    let uart_port = serialport::new("/dev/ttyUSB0", 115_200);
    let (tx, rx) = robots_drv::driver(uart_port)?;

    let tx_task = forward_tx(tx);
    let rx_task = forward_rx(rx);
    tokio::pin!(rx_task);
    tokio::pin!(tx_task);
    println!("uart serve");
    tokio::select! {
        ret = rx_task => eprintln!("rx task ended with {ret:?}"),
        ret = tx_task => eprintln!("tx task ended with {ret:?}"),
    }
    Ok(())
}
