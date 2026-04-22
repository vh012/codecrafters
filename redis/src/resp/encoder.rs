use std::io::{self, ErrorKind};

use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

use crate::resp::{constants::END_SEQ, parser::RespCodec, types::RespType};

impl Encoder<RespType> for RespCodec {
    type Error = io::Error;

    fn encode(&mut self, item: RespType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let resp_byte_type = u8::try_from(&item).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("unable to cast RespType into u8: {e}"),
            )
        })?;

        dst.put_u8(resp_byte_type);

        match item {
            RespType::RError(err_msg) => {
                dst.extend_from_slice(err_msg.as_ref());
                dst.extend_from_slice(&END_SEQ);
            }
            RespType::Integer(Some(int)) => {
                if int < 0 {
                    dst.put_u8(b'-');
                }
                dst.extend_from_slice(int.to_string().as_ref());
                dst.extend_from_slice(&END_SEQ);
            }
            RespType::SimpleString(Some(simple_str)) => {
                dst.extend_from_slice(simple_str.as_ref());
                dst.extend_from_slice(&END_SEQ);
            }
            RespType::BulkString(bulk_str) => match bulk_str {
                Some(str) => {
                    dst.extend_from_slice(str.len().to_string().as_ref());
                    dst.extend_from_slice(&END_SEQ);
                    dst.extend_from_slice(str.as_ref());
                    dst.extend_from_slice(&END_SEQ);
                }
                None => {
                    dst.extend_from_slice(b"-1");
                    dst.extend_from_slice(&END_SEQ);
                }
            },
            RespType::Array(Some(arr)) => {
                dst.extend_from_slice(arr.len().to_string().as_ref());
                dst.extend_from_slice(&END_SEQ);

                for nested_resp in arr {
                    self.encode(nested_resp, dst)?;
                }
            }
            _ => {
                return Err(io::Error::new(
                    ErrorKind::InvalidInput,
                    "encoder does not support provided type or data might be malformed",
                ));
            }
        }

        Ok(())
    }
}
