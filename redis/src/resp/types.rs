use std::{io, num::ParseIntError};

use bytes::BytesMut;
use thiserror::Error;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum RespType {
    SimpleString(Option<BytesMut>),
    BulkString(Option<BytesMut>),
    Array(Option<Vec<RespType>>),
    Integer(Option<i64>),
    RError(String),
}

impl TryFrom<u8> for RespType {
    type Error = RespTypeError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'+' => Ok(Self::SimpleString(None)),
            b'$' => Ok(Self::BulkString(None)),
            b'*' => Ok(Self::Array(None)),
            b':' => Ok(Self::Integer(None)),
            _ => Err(RespTypeError::UnsupportedType(byte as char)),
        }
    }
}

impl TryFrom<&RespType> for u8 {
    type Error = ParseIntError;

    fn try_from(resp: &RespType) -> Result<u8, Self::Error> {
        Ok(match resp {
            RespType::SimpleString(_) => b'+',
            RespType::BulkString(_) => b'$',
            RespType::Array(_) => b'*',
            RespType::Integer(_) => b':',
            RespType::RError(_) => b'-',
        })
    }
}

impl From<io::Error> for RespType {
    fn from(value: io::Error) -> Self {
        RespType::RError(value.to_string())
    }
}

#[derive(Error, Debug)]
pub enum RespTypeError {
    #[error("unsupported resp type provided: {0}")]
    UnsupportedType(char),
}

impl From<RespTypeError> for RespType {
    fn from(value: RespTypeError) -> Self {
        RespType::RError(value.to_string())
    }
}
