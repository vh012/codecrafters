use std::{io, num::ParseIntError};

use thiserror::Error;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum RespDataType {
    SimpleStrings(Option<String>),
    BulkStrings(Option<String>),
    Arrays(Option<Vec<RespDataType>>),
    Integers(Option<i64>),
    Errors(String),
}

impl TryFrom<u8> for RespDataType {
    type Error = RespTypeError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'+' => Ok(Self::SimpleStrings(None)),
            b'$' => Ok(Self::BulkStrings(None)),
            b'*' => Ok(Self::Arrays(None)),
            b':' => Ok(Self::Integers(None)),
            _ => Err(RespTypeError::UnsupportedType(byte as char)),
        }
    }
}

impl TryFrom<&RespDataType> for u8 {
    type Error = ParseIntError;

    fn try_from(resp: &RespDataType) -> Result<u8, Self::Error> {
        Ok(match resp {
            RespDataType::SimpleStrings(_) => b'+',
            RespDataType::BulkStrings(_) => b'$',
            RespDataType::Arrays(_) => b'*',
            RespDataType::Integers(_) => b':',
            RespDataType::Errors(_) => b'-',
        })
    }
}

impl From<io::Error> for RespDataType {
    fn from(value: io::Error) -> Self {
        RespDataType::Errors(value.to_string())
    }
}

#[derive(Error, Debug)]
pub enum RespTypeError {
    #[error("usupported RESP type provided: {0}")]
    UnsupportedType(char),
}

impl From<RespTypeError> for RespDataType {
    fn from(value: RespTypeError) -> Self {
        RespDataType::Errors(value.to_string())
    }
}
