use bytes::{Buf, BytesMut};

use crate::resp::{
    parser::rules::{
        parse_rule_factory,
        types::{BoxedRespParseRule, ParseRule, RespParseRule, RespRuleParseError},
        utils::{get_end_seq_len, is_end_seq},
    },
    types::RespDataType,
};

#[derive(Debug)]
pub(crate) struct ArraysParseRule {
    values: Vec<RespDataType>,
    size: Option<usize>,
    current_parse_rule: Option<BoxedRespParseRule>,
}

impl ArraysParseRule {
    pub(crate) fn new() -> Self {
        Self {
            values: vec![],
            size: None,
            current_parse_rule: None,
        }
    }

    fn get_size(&self) -> usize {
        self.size.unwrap_or(0)
    }

    fn parse_next_rule(
        &mut self,
        bytes: &mut BytesMut,
    ) -> Result<Option<RespDataType>, RespRuleParseError> {
        if self.get_size() < 1 {
            return Ok(None);
        }

        if self.current_parse_rule.is_none() {
            let rule_type_byte = bytes.get(0);

            match rule_type_byte {
                Some(b) => self.current_parse_rule = Some(parse_rule_factory(*b)?),
                None => {
                    return Err(RespRuleParseError::UnexpectedSubruleParseError(
                        "unable to determine next subrule".to_string(),
                    ));
                }
            }
        }

        let rule_parse_result: Option<RespDataType> = self
            .current_parse_rule
            .as_mut()
            .ok_or(RespRuleParseError::UnexpectedSubruleParseError(
                "unexpected empty subrule".to_string(),
            ))?
            .next(bytes)?;

        let Some(rule_parse_result) = rule_parse_result else {
            return Ok(None);
        };

        self.size = Some(self.get_size() - 1);
        self.current_parse_rule = None;

        return Ok(Some(rule_parse_result));
    }
}

impl ParseRule for ArraysParseRule {
    type Output = RespDataType;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError> {
        if self.size.is_none() {
            if bytes.len() < 4 {
                return Ok(None);
            }

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

        if bytes.len() < 2 {
            return Ok(None);
        }

        while let Some(parse_result) = self.parse_next_rule(bytes)? {
            self.values.push(parse_result);
        }

        if self.get_size() <= 0 {
            return Ok(Some(RespDataType::Arrays(Some(self.values.to_vec()))));
        }

        Ok(None)
    }
}

impl RespParseRule for ArraysParseRule {}
