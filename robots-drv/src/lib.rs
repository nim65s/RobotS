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

struct Driver {
    writer: SplitSink<Framed<SerialStream, Codec>, Cmd>,
    reader: SplitStream<Framed<SerialStream, Codec>>,
    rx: Channel,
    tx: Channel,
}

impl Driver {
    #[must_use]
    fn new(port: SerialStream) -> Self {
        let (writer, reader) = Codec.framed(port).split();
        Self {
            writer,
            reader,
            rx: Channel::new(),
            tx: Channel::new(),
        }
    }

    async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                Some(cmd) = self.reader.next() => {
                    self.rx.send(&cmd?).await?;
                }
                Some(cmd) = self.tx.next() => {
                    self.writer.send(cmd).await?;
                }
            }
        }
    }
}

pub fn driver(port: SerialPortBuilder) -> Result<(Channel, Channel)> {
    let mut port = port.open_native_async()?;
    port.set_exclusive(false)?;
    let mut drv = Driver::new(port);
    let ret = (drv.tx.clone(), drv.rx.clone());
    tokio::spawn(async move { drv.run().await });
    Ok(ret)
}
