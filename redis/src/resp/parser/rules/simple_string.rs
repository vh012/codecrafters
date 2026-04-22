use bytes::{Buf, BytesMut};

use crate::resp::{
    parser::rules::{
        types::{ParseRule, RespParseRule, RespRuleParseError},
        utils::{get_end_seq_len, is_end_seq},
    },
    types::RespType,
};

#[derive(Debug)]
pub(crate) struct SimpleStringsParseRule {}

impl SimpleStringsParseRule {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl ParseRule for SimpleStringsParseRule {
    type Output = RespType;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError> {
        if bytes.len() < 4 {
            return Ok(None);
        }

        for idx in 1..bytes.len() - 1 {
            if !is_end_seq(&bytes[idx..idx + get_end_seq_len()]) {
                continue;
            }

            let simple_string = Some(RespType::SimpleString(Some(bytes[1..idx].into())));

            bytes.advance(idx + get_end_seq_len());

            return Ok(simple_string);
        }

        Ok(None)
    }
}

impl RespParseRule for SimpleStringsParseRule {}
