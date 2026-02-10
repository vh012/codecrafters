use std::io;

use crate::resp::{
    parser::rules::{
        array::ArraysParseRule, bulk_string::BulkStringsParseRule,
        simple_string::SimpleStringsParseRule, types::BoxedRespParseRule,
    },
    resp_types::{RespDataType, RespTypeError},
};
use thiserror::Error;

pub(crate) mod array;
pub(crate) mod bulk_string;
pub(crate) mod simple_string;
pub(crate) mod types;
mod utils;

pub fn parse_rule_factory(byte: u8) -> Result<BoxedRespParseRule, ParseRuleFactoryError> {
    match RespDataType::try_from(byte)? {
        RespDataType::SimpleStrings(_) => Ok(Box::new(SimpleStringsParseRule::new())),
        RespDataType::BulkStrings(_) => Ok(Box::new(BulkStringsParseRule::new())),
        RespDataType::Arrays(_) => Ok(Box::new(ArraysParseRule::new())),
    }
}

#[derive(Error, Debug)]
pub enum ParseRuleFactoryError {
    #[error("unable to get parse rule for provided type: {0}")]
    UnexpectedRespType(String),
}

impl From<RespTypeError> for ParseRuleFactoryError {
    fn from(error: RespTypeError) -> Self {
        Self::UnexpectedRespType(error.to_string())
    }
}

impl From<ParseRuleFactoryError> for io::Error {
    fn from(error: ParseRuleFactoryError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}
