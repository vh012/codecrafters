use std::io;

use bytes::BytesMut;
use tokio_util::codec::Decoder;

use crate::resp::{parser::RespCodec, parser::rules::parse_rule_factory, types::RespDataType};

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

        let Some(rule) = self.rule.as_mut() else {
            return Ok(None);
        };

        match rule.next(src)? {
            None => Ok(None),
            res @ Some(_) => {
                self.rule = None;

                Ok(res)
            }
        }
    }
}
