use bytes::{Buf, BytesMut};

use crate::resp::{
    parser::rules::{
        types::{ParseRule, RespParseRule, RespRuleParseError},
        utils::{get_end_seq_len, is_end_seq},
    },
    resp_types::RespDataType,
};

#[derive(Debug)]
pub struct SimpleStringsParseRule {}

impl SimpleStringsParseRule {
    pub fn new() -> Self {
        Self {}
    }
}

impl ParseRule for SimpleStringsParseRule {
    type Output = RespDataType;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError> {
        if bytes.len() < 4 {
            return Ok(None);
        }

        for idx in 1..bytes.len() - 1 {
            if !is_end_seq(&bytes[idx..idx + get_end_seq_len()]) {
                continue;
            }

            let utf8_str = Some(RespDataType::SimpleStrings(Some(
                String::from_utf8_lossy(&bytes[1..idx]).into(),
            )));

            bytes.advance(idx + get_end_seq_len());

            return Ok(utf8_str);
        }

        Ok(None)
    }
}

impl RespParseRule for SimpleStringsParseRule {}
