use std::fmt::Display;
use std::io;

use crate::rdb::opcodes::OpCode;
use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RdbType {
    String,
}

impl RdbType {
    pub fn is_valid_type(value: &u8) -> bool {
        matches!(*value, 0)
    }
}

impl TryFrom<u8> for RdbType {
    type Error = RdbTypeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(RdbType::String),
            _ => Err(RdbTypeParseError::UnexpectedRdbType(value)),
        }
    }
}

impl From<RdbType> for u8 {
    fn from(value: RdbType) -> Self {
        match value {
            RdbType::String => 0,
        }
    }
}

impl Display for RdbType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RdbType::String => write!(f, "RdbType::String"),
        }
    }
}

#[derive(Error, Debug)]
pub enum RdbTypeParseError {
    #[error("unexpected type: {0}")]
    UnexpectedRdbType(u8),
}

impl From<RdbTypeParseError> for io::Error {
    fn from(error: RdbTypeParseError) -> io::Error {
        io::Error::other(error)
    }
}
