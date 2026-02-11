use thiserror::Error;

#[derive(Eq, PartialEq, PartialOrd, Ord, Hash, Debug, Clone)]
pub enum RespDataType {
    SimpleStrings(Option<String>),
    BulkStrings(Option<String>),
    Arrays(Option<Vec<RespDataType>>),
}

impl TryFrom<u8> for RespDataType {
    type Error = RespTypeError;

    fn try_from(byte: u8) -> Result<Self, Self::Error> {
        match byte {
            b'+' => Ok(Self::SimpleStrings(None)),
            b'$' => Ok(Self::BulkStrings(None)),
            b'*' => Ok(Self::Arrays(None)),
            _ => Err(RespTypeError::UnsupportedType(byte as char)),
        }
    }
}

#[derive(Error, Debug)]
pub enum RespTypeError {
    #[error("usupported RESP type provided: {0}")]
    UnsupportedType(char),
}
