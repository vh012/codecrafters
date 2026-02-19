use bytes::{Buf, BytesMut};

use crate::resp::{
    parser::rules::{
        types::{ParseRule, RespParseRule, RespRuleParseError},
        utils::{get_end_seq_len, is_end_seq},
    },
    types::RespDataType,
};

#[derive(Debug)]
pub(crate) struct IntegersParseRule {}

impl IntegersParseRule {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

impl ParseRule for IntegersParseRule {
    type Output = RespDataType;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError> {
        if bytes.len() < 4 {
            return Ok(None);
        }

        for idx in 1..bytes.len() - 1 {
            if !is_end_seq(&bytes[idx..idx + get_end_seq_len()]) {
                continue;
            }

            let integer = Some(RespDataType::Integers(Some(str::parse::<i64>(
                str::from_utf8(&bytes[1..idx])?,
            )?)));

            bytes.advance(idx + get_end_seq_len());

            return Ok(integer);
        }

        Ok(None)
    }
}

impl RespParseRule for IntegersParseRule {}
