use async_channel::{unbounded, Receiver, Sender};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Framed};

use robots_lib::Cmd;

mod codec;
use codec::Codec;

mod error;
pub use error::Result;

pub struct Driver {
    writer: SplitSink<Framed<SerialStream, Codec>, Cmd>,
    reader: SplitStream<Framed<SerialStream, Codec>>,
    rx_send: Sender<Cmd>,
    rx_recv: Receiver<Cmd>,
    tx_send: Sender<Cmd>,
    tx_recv: Receiver<Cmd>,
}

impl Driver {
    #[must_use]
    pub fn new(port: SerialStream) -> Self {
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

    pub async fn run(&mut self) -> Result<()> {
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

    #[must_use]
    pub fn sender(&self) -> Sender<Cmd> {
        self.tx_send.clone()
    }

    #[must_use]
    pub fn receiver(&self) -> Receiver<Cmd> {
        self.rx_recv.clone()
    }
}
