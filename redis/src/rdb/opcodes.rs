use std::fmt::Display;
use std::io;

use thiserror::Error;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpCode {
    Eof,
    SelectDb,
    ExpireTime,
    ExpireTimeMs,
    ResizeDb,
    Aux,
}

impl OpCode {
    pub fn is_valid_opcode(value: &u8) -> bool {
        matches!(*value, 0xFA..=0xFF)
    }
}

impl TryFrom<u8> for OpCode {
    type Error = OpCodeParseError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0xFF => Ok(OpCode::Eof),
            0xFE => Ok(OpCode::SelectDb),
            0xFD => Ok(OpCode::ExpireTime),
            0xFC => Ok(OpCode::ExpireTimeMs),
            0xFB => Ok(OpCode::ResizeDb),
            0xFA => Ok(OpCode::Aux),
            _ => Err(OpCodeParseError::UnexpectedOpCode(value)),
        }
    }
}

impl From<OpCode> for u8 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Aux => 0xFA,
            OpCode::ResizeDb => 0xFB,
            OpCode::ExpireTimeMs => 0xFC,
            OpCode::ExpireTime => 0xFD,
            OpCode::SelectDb => 0xFE,
            OpCode::Eof => 0xFF,
        }
    }
}

impl Display for OpCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpCode::Aux => write!(f, "OpCode::Aux"),
            OpCode::ResizeDb => write!(f, "OpCode::ResizeDb"),
            OpCode::ExpireTimeMs => write!(f, "OpCode::ExpireTimeMs"),
            OpCode::ExpireTime => write!(f, "OpCode::ExpireTime"),
            OpCode::SelectDb => write!(f, "OpCode::SelectDb "),
            OpCode::Eof => write!(f, "OpCode::Eof"),
        }
    }
}

#[derive(Error, Debug)]
pub enum OpCodeParseError {
    #[error("unexpected opcode: {0}")]
    UnexpectedOpCode(u8),
}

impl From<OpCodeParseError> for io::Error {
    fn from(error: OpCodeParseError) -> io::Error {
        io::Error::other(error)
    }
}
