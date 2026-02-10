use std::io;

use bytes::BytesMut;
use tokio_util::codec::Decoder;

use crate::resp::{parser::RespCodec, parser::rules::parse_rule_factory, resp_types::RespDataType};

impl Decoder for RespCodec {
    type Item = RespDataType;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }

        if self.rule.is_none() {
            self.rule = Some(parse_rule_factory(src[0])?)
        }

        let data = self.rule.as_mut().unwrap().next(src)?;

        if data.is_none() {
            return Ok(None);
        }

        self.rule = None;

        Ok(data)
    }
}
