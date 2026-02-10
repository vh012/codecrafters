use bytes::BytesMut;
use std::{fmt::Debug, io, num::ParseIntError, str::Utf8Error};
use thiserror::Error;

use crate::resp::{parser::rules::ParseRuleFactoryError, resp_types::RespDataType};

pub trait ParseRule {
    type Output;

    fn next(&mut self, bytes: &mut BytesMut) -> Result<Option<Self::Output>, RespRuleParseError>;
}

pub trait RespParseRule: ParseRule<Output = RespDataType> + Send + Debug {}

pub type BoxedRespParseRule = Box<dyn RespParseRule>;

#[derive(Error, Debug)]
pub enum RespRuleParseError {
    #[error("unable to parse numeric value: {0}")]
    NumericParseError(String),
    #[error("unable to parse utf8 value: {0}")]
    Utf8ParseError(String),
    #[error("unable to parse with subrule: {0}")]
    UnexpectedSubruleParseError(String),
}

impl From<ParseIntError> for RespRuleParseError {
    fn from(error: ParseIntError) -> Self {
        Self::NumericParseError(error.to_string())
    }
}

impl From<Utf8Error> for RespRuleParseError {
    fn from(error: Utf8Error) -> Self {
        Self::Utf8ParseError(error.to_string())
    }
}

impl From<ParseRuleFactoryError> for RespRuleParseError {
    fn from(error: ParseRuleFactoryError) -> Self {
        Self::UnexpectedSubruleParseError(error.to_string())
    }
}

impl From<RespRuleParseError> for io::Error {
    fn from(error: RespRuleParseError) -> io::Error {
        io::Error::new(io::ErrorKind::InvalidData, error.to_string())
    }
}
