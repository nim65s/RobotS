use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio_serial::{SerialPortBuilder, SerialPortBuilderExt, SerialStream};
use tokio_util::codec::{Decoder, Framed};

pub use robots_lib::Cmd;

mod codec;
use codec::Codec;

pub mod error;
pub use error::{Error, Result};

pub type Channel = broadcaster::BroadcastChannel<Cmd>;

lazy_static::lazy_static! {
    pub static ref TX: Channel = Channel::new();
    pub static ref RX: Channel = Channel::new();
}

struct Driver {
    writer: SplitSink<Framed<SerialStream, Codec>, Cmd>,
    reader: SplitStream<Framed<SerialStream, Codec>>,
}

impl Driver {
    #[must_use]
    fn new(port: SerialStream) -> Self {
        let (writer, reader) = Codec.framed(port).split();
        Self { writer, reader }
    }

    async fn run(&mut self) -> Result<()> {
        let mut tx = TX.clone();
        let rx = RX.clone();
        loop {
            tokio::select! {
                Some(cmd) = self.reader.next() => {
                    let cmd = cmd?;
                    rx.send(&cmd).await?;
                    if cmd == Cmd::Ping {
                        self.writer.send(Cmd::Pong).await?;
                    }
                }
                Some(cmd) = tx.next() => {
                    self.writer.send(cmd).await?;
                }
            }
        }
    }
}

pub fn driver(port: SerialPortBuilder) -> Result<()> {
    let mut port = port.open_native_async()?;
    port.set_exclusive(false)?;
    let mut drv = Driver::new(port);
    tokio::spawn(async move { drv.run().await });
    Ok(())
}
