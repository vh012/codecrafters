use std::io::{self, ErrorKind};

use bytes::{BufMut, BytesMut};
use tokio_util::codec::Encoder;

use crate::resp::{constants::END_SEQ, parser::RespCodec, types::RespDataType};

impl Encoder<RespDataType> for RespCodec {
    type Error = io::Error;

    fn encode(&mut self, item: RespDataType, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let resp_byte_type = u8::try_from(&item).map_err(|e| {
            io::Error::new(
                ErrorKind::InvalidInput,
                format!("unable to cast RespDataType into u8: {e}"),
            )
        })?;

        dst.put_u8(resp_byte_type);

        match item {
            RespDataType::Errors(err_msg) => {
                dst.extend_from_slice(err_msg.as_bytes());
                dst.extend_from_slice(&END_SEQ);
            }
            RespDataType::Integers(Some(int)) => {
                if int < 0 {
                    dst.put_u8(b'-');
                }
                dst.extend_from_slice(int.to_string().as_bytes());
                dst.extend_from_slice(&END_SEQ);
            }
            RespDataType::SimpleStrings(Some(str)) => {
                dst.extend_from_slice(str.as_bytes());
                dst.extend_from_slice(&END_SEQ);
            }
            RespDataType::BulkStrings(bulk_string) => match bulk_string {
                Some(str) => {
                    dst.extend_from_slice(str.len().to_string().as_bytes());
                    dst.extend_from_slice(&END_SEQ);
                    dst.extend_from_slice(str.as_bytes());
                    dst.extend_from_slice(&END_SEQ);
                }
                None => {
                    dst.extend_from_slice(&[b'-', b'1']);
                    dst.extend_from_slice(&END_SEQ);
                }
            },
            RespDataType::Arrays(Some(arr)) => {
                dst.extend_from_slice(arr.len().to_string().as_bytes());
                dst.extend_from_slice(&END_SEQ);

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
