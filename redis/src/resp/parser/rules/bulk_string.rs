use bytes::{Buf, BytesMut};

use crate::resp::{
    parser::rules::{
        types::{ParseRule, RespParseRule, RespRuleParseError},
        utils::{get_end_seq_len, is_end_seq},
    },
    types::RespType,
};

#[derive(Debug)]
pub(crate) struct BulkStringsParseRule {
    size: Option<usize>,
}

impl BulkStringsParseRule {
    pub(crate) fn new() -> Self {
        Self { size: None }
    }

    fn get_size(&self) -> usize {
        self.size.unwrap_or(0)
    }
}

impl ParseRule for BulkStringsParseRule {
    type Output = RespType;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError> {
        if self.size.is_none() {
            let mut end_seq_idx = None;

            for idx in 0..bytes.len() - 1 {
                if !is_end_seq(&bytes[idx..idx + get_end_seq_len()]) {
                    continue;
                }

                end_seq_idx = Some(idx);

                break;
            }

            match end_seq_idx {
                Some(idx) => {
                    self.size = Some(str::parse::<usize>(str::from_utf8(&bytes[1..idx])?)?);
                    bytes.advance(idx + get_end_seq_len());
                }
                None => return Ok(None),
            }
        }

        if self.get_size() == 0 {
            return Ok(Some(RespType::BulkString(None)));
        }

        if bytes.len() < 2 {
            return Ok(None);
        }

        for idx in 0..bytes.len() - 1 {
            if !is_end_seq(&bytes[idx..idx + get_end_seq_len()]) {
                continue;
            }

            let bulk_string = Some(RespType::BulkString(Some(bytes[..idx].into())));

            bytes.advance(idx + get_end_seq_len());

            return Ok(bulk_string);
        }

        Ok(None)
    }
}

impl RespParseRule for BulkStringsParseRule {}
