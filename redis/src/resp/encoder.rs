use std::io::{self, ErrorKind};

use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

use crate::resp::{parser::RespCodec, resp_types::RespDataType};

impl Encoder<RespDataType> for RespCodec {
    type Error = io::Error;

    fn encode(&mut self, item: RespDataType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        match item {
            RespDataType::SimpleStrings(Some(str)) => {
                dst.put_u8(b'+');
                dst.extend_from_slice(str.as_bytes());
                dst.extend_from_slice(&[b'\r', b'\n']);
            }
            RespDataType::BulkStrings(str) => match str {
                Some(str) => {
                    dst.put_u8(b'$');
                    dst.extend_from_slice(str.len().to_string().as_bytes());
                    dst.extend_from_slice(&[b'\r', b'\n']);
                    dst.extend_from_slice(str.as_bytes());
                    dst.extend_from_slice(&[b'\r', b'\n']);
                }
                None => {
                    dst.put_u8(b'$');
                    dst.extend_from_slice(&[b'-', b'1']);
                    dst.extend_from_slice(&[b'\r', b'\n']);
                }
            },
            RespDataType::Arrays(Some(arr)) => {
                dst.put_u8(b'*');
                dst.extend_from_slice(arr.len().to_string().as_bytes());
                dst.extend_from_slice(&[b'\r', b'\n']);

                for resp_data in arr {
                    self.encode(resp_data, dst)?;
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
