use bytes::{BufMut, BytesMut};
use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use tokio::time::{sleep, Duration};
use tokio_serial::SerialStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use robots_lib::Cmd;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("RobotS lib error: {0}")]
    Robots(#[from] robots_lib::Error),
}

pub struct CmdCodec;

impl Decoder for CmdCodec {
    type Item = Cmd;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        //println!("decoding {src:02x}");
        let endframe = src.as_ref().iter().position(|b| *b == 0);
        Ok(if let Some(n) = endframe {
            let mut cmd = src.split_to(n + 1);
            //println!("got {cmd:02x}");
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

pub type UartWriter = SplitSink<Framed<SerialStream, CmdCodec>, Cmd>;
pub type UartReader = SplitStream<Framed<SerialStream, CmdCodec>>;

pub async fn recv_serial(mut uart_reader: UartReader) {
    println!("receiving...");
    loop {
        match uart_reader.next().await {
            Some(Ok(cmd)) => println!("received {cmd:?}"),
            Some(Err(e)) => println!("received err {e:?}"),
            None => println!("received nothing."),
        }
    }
}

pub async fn send_serial(mut uart_writer: UartWriter) {
    for cmd in [Cmd::Get, Cmd::Ping, Cmd::Pong] {
        sleep(Duration::from_millis(8_000)).await;
        println!("sending {cmd:?}...");
        uart_writer.send(cmd).await.unwrap();
    }
}
