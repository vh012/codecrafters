use thiserror::Error;

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
        match *value {
            0xFA..=0xFF => true,
            _ => false,
        }
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

#[derive(Error, Debug)]
pub enum OpCodeParseError {
    #[error("unexpected opcode: {0}")]
    UnexpectedOpCode(u8),
}
