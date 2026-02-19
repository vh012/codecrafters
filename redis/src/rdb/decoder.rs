use std::io;

use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

use crate::rdb::{constants::HEADER_STR, opcodes::OpCode, parser::RdbCodec};

impl Decoder for RdbCodec {
    type Item = ();
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if !self.is_header_read {
            let Some(first_opcode_idx) = src.iter().position(|b| OpCode::is_valid_opcode(b)) else {
                return Ok(None);
            };

            if !src[..first_opcode_idx].ends_with(HEADER_STR.as_bytes()) {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "unexpected header value",
                ));
            }

            self.is_header_read = true;

            src.advance(first_opcode_idx);
        }

        Ok(Some(()))
    }
}
