use async_channel::{unbounded, Receiver, Sender};
use bytes::{BufMut, BytesMut};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use robots_lib::Cmd;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RobotS lib error: {0}")]
    Robots(#[from] robots_lib::Error),

    #[error("Async channel SendError: {0}")]
    SendError(#[from] async_channel::SendError<Cmd>),

    #[error("Async channel RecvError: {0}")]
    RecvError(#[from] async_channel::RecvError),
}

pub struct CmdCodec;

impl Decoder for CmdCodec {
    type Item = Cmd;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let endframe = src.as_ref().iter().position(|b| *b == 0);
        Ok(if let Some(n) = endframe {
            let mut cmd = src.split_to(n + 1);
            Some(Cmd::from_vec(&mut cmd)?)
        } else {
            None
        })
    }
}

impl Encoder<Cmd> for CmdCodec {
    type Error = Error;

    fn encode(&mut self, cmd: Cmd, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let data = cmd.to_vec()?;
        buf.reserve(data.len());
        buf.put(data.as_slice());
        Ok(())
    }
}

pub struct Drv {
    writer: SplitSink<Framed<SerialStream, CmdCodec>, Cmd>,
    reader: SplitStream<Framed<SerialStream, CmdCodec>>,
    rx_send: Sender<Cmd>,
    rx_recv: Receiver<Cmd>,
    tx_send: Sender<Cmd>,
    tx_recv: Receiver<Cmd>,
}

impl Drv {
    pub fn new(port: SerialStream) -> Self {
        let (writer, reader) = CmdCodec.framed(port).split();
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

    pub async fn run(&mut self) -> Result<(), Error> {
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

    pub fn sender(&self) -> Sender<Cmd> {
        self.tx_send.clone()
    }

    pub fn receiver(&self) -> Receiver<Cmd> {
        self.rx_recv.clone()
    }
}
