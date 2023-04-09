use async_channel::{unbounded, Receiver, Sender};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio_serial::SerialStream;
use tokio_serial::{SerialPortBuilder, SerialPortBuilderExt};
use tokio_util::codec::{Decoder, Framed};

pub use robots_lib::Cmd;

mod codec;
use codec::Codec;

mod error;
pub use error::Result;

struct Driver {
    writer: SplitSink<Framed<SerialStream, Codec>, Cmd>,
    reader: SplitStream<Framed<SerialStream, Codec>>,
    rx_send: Sender<Cmd>,
    rx_recv: Receiver<Cmd>,
    tx_send: Sender<Cmd>,
    tx_recv: Receiver<Cmd>,
}

impl Driver {
    #[must_use]
    fn new(port: SerialStream) -> Self {
        let (writer, reader) = Codec.framed(port).split();
        let (rx_send, rx_recv) = unbounded();
        let (tx_send, tx_recv) = unbounded();
        Self {
            writer,
            reader,
            rx_send,
            rx_recv,
            tx_send,
            tx_recv,
        }
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(cmd) = self.reader.next() => {
                    self.rx_send.send(cmd?).await?;
                }
                Ok(cmd) = self.tx_recv.recv() => {
                    self.writer.send(cmd).await?;
                }
            }
        }
    }
}

pub fn driver(port: SerialPortBuilder) -> Result<(Sender<Cmd>, Receiver<Cmd>)> {
    let mut port = port.open_native_async()?;
    port.set_exclusive(false)?;
    let mut drv = Driver::new(port);
    let ret = (drv.tx_send.clone(), drv.rx_recv.clone());
    tokio::spawn(async move { drv.run().await });
    Ok(ret)
}
