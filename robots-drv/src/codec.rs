use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use robots_lib::Cmd;

use crate::error::{Error, Result};

pub struct Codec;

impl Decoder for Codec {
    type Item = Cmd;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        let endframe = src.as_ref().iter().position(|b| *b == 0);
        Ok(if let Some(n) = endframe {
            let mut cmd = src.split_to(n + 1);
            Some(Cmd::from_vec(&mut cmd)?)
        } else {
            None
        })
    }
}

impl Encoder<Cmd> for Codec {
    type Error = Error;

    fn encode(&mut self, cmd: Cmd, buf: &mut BytesMut) -> Result<()> {
        let data = cmd.to_vec()?;
        buf.reserve(data.len());
        buf.put(data.as_slice());
        Ok(())
    }
}
