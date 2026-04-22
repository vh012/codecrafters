use crate::commands::hash_map::Value;
use crate::rdb::types::RdbType;
use crate::rdb::{constants::HEADER_STR, opcodes::OpCode, parser::RdbCodec};
use crate::resp::types::RespType;
use bytes::{Buf, BytesMut};
use std::io::{self, Read};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio_util::codec::Decoder;

#[derive(Debug)]
enum LenEncodingType {
    String(usize),
    StringInteger(usize),
}

#[derive(Debug)]
enum LenEncodingValue {
    String(BytesMut),
    StringInteger(String),
}

impl From<LenEncodingValue> for RespType {
    fn from(value: LenEncodingValue) -> Self {
        match value {
            LenEncodingValue::String(v) => RespType::BulkString(Some(v)),
            LenEncodingValue::StringInteger(v) => RespType::BulkString(Some(v.as_bytes().into())),
        }
    }
}

fn parse_len_encoding(bytes: &mut BytesMut) -> Option<LenEncodingType> {
    match bytes.first() {
        Some(first_byte) => {
            let significant_bytes = first_byte >> 6;

            if significant_bytes == 0 {
                let len = Some(LenEncodingType::String((first_byte & 0x3f) as usize));

                bytes.advance(1);

                len
            } else if significant_bytes == 0b01 {
                let len = bytes.get(1).map(|second_byte| {
                    LenEncodingType::String(
                        (((first_byte & 0x3f) as usize) << 8) | *second_byte as usize,
                    )
                });

                bytes.advance(2);

                len
            } else if significant_bytes == 0b10 {
                let len = bytes.get(1).and_then(|second_byte| {
                    bytes.get(2).and_then(|third_byte| {
                        bytes.get(3).and_then(|fourth_byte| {
                            bytes.get(4).map(|fifth_byte| {
                                LenEncodingType::String(
                                    (*second_byte as usize) << 24
                                        | ((*third_byte as usize) << 16)
                                        | ((*fourth_byte as usize) << 8)
                                        | *fifth_byte as usize,
                                )
                            })
                        })
                    })
                });

                bytes.advance(5);

                len
            } else if significant_bytes == 0b11 {
                let special_format = (first_byte & 0x3f) as usize;

                let len = match special_format {
                    0..=1 => Some(LenEncodingType::StringInteger(special_format + 1)),
                    2 => Some(LenEncodingType::StringInteger(special_format.pow(2))),
                    _ => None,
                };

                bytes.advance(1);

                len
            } else {
                None
            }
        }
        _ => None,
    }
}

fn decode_single_value(bytes: &mut BytesMut) -> Option<LenEncodingValue> {
    let len_encoding_type = parse_len_encoding(bytes)?;

    match len_encoding_type {
        LenEncodingType::String(str_len) => {
            let mut buf = BytesMut::with_capacity(str_len);
            buf.fill_with(|| 0);

            bytes.reader().read_exact(buf.as_mut()).ok()?;

            Some(LenEncodingValue::String(buf))
        }
        LenEncodingType::StringInteger(int_len) => match int_len {
            1 => Some(LenEncodingValue::StringInteger(bytes.get_i8().to_string())),
            2 => Some(LenEncodingValue::StringInteger(
                bytes.get_i16_le().to_string(),
            )),
            4 => Some(LenEncodingValue::StringInteger(
                bytes.get_i32_le().to_string(),
            )),
            _ => None,
        },
    }
}

impl<'a> Decoder for RdbCodec<'a> {
    type Item = ();
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < HEADER_STR.len() {
            return Ok(None);
        }

        if !self.is_header_read && !src.starts_with(HEADER_STR.as_ref()) {
            return Err(io::Error::other("unexpected header value"));
        } else if !self.is_header_read {
            src.advance(HEADER_STR.len());
            self.is_header_read = true;
        };

        loop {
            match src.first() {
                Some(byte) if OpCode::is_valid_opcode(byte) => {
                    match OpCode::try_from(*byte) {
                        Ok(opcode) => match opcode {
                            OpCode::Aux => {
                                src.advance(1);

                                if let (Some(key), Some(value)) =
                                    (decode_single_value(src), decode_single_value(src))
                                {
                                    println!("OpCode::Aux: key {key:?} - value {value:?}");
                                } else {
                                    return Err(io::Error::other(format!(
                                        "unable to parse {opcode}"
                                    )));
                                }
                            }
                            OpCode::SelectDb => {
                                src.advance(1);
                                parse_len_encoding(src);
                            }
                            OpCode::ResizeDb => {
                                src.advance(1);
                                parse_len_encoding(src);
                                parse_len_encoding(src);
                            }
                            op @ OpCode::ExpireTime | op @ OpCode::ExpireTimeMs => {
                                src.advance(1);

                                let is_ms = op == OpCode::ExpireTimeMs;

                                let expire_time_ms: u64 = if is_ms {
                                    src.get_u64_le()
                                } else {
                                    src.get_u32_le() as u64 * 1000
                                };
                                let value_type =
                                    src.first().and_then(|b| RdbType::try_from(*b).ok());

                                match value_type {
                                    Some(RdbType::String) => {
                                        src.advance(1);

                                        if let (Some(key), Some(value)) =
                                            (decode_single_value(src), decode_single_value(src))
                                        {
                                            let now = SystemTime::now()
                                                .duration_since(UNIX_EPOCH)
                                                .map_err(|e| io::Error::other(e.to_string()))?;

                                            if Duration::from_millis(expire_time_ms).gt(&now) {
                                                self.map.insert(
                                                    key.into(),
                                                    Value::new(
                                                        value.into(),
                                                        Some(
                                                            Duration::from_millis(expire_time_ms)
                                                                - now,
                                                        ),
                                                    ),
                                                );
                                            }
                                        } else {
                                            return Err(io::Error::other(format!(
                                                "unable to parse {op}"
                                            )));
                                        }
                                    }
                                    _ => {
                                        return Err(io::Error::other(format!(
                                            "cannot read value type for {op}"
                                        )));
                                    }
                                }
                            }
                            OpCode::Eof => {
                                return Ok(Some(()));
                            }
                        },
                        Err(e) => return Err(e.into()),
                    };
                }
                Some(byte) if RdbType::is_valid_type(byte) => match RdbType::try_from(*byte) {
                    Ok(rdb_type) => match rdb_type {
                        RdbType::String => {
                            src.advance(1);

                            if let (Some(key), Some(value)) =
                                (decode_single_value(src), decode_single_value(src))
                            {
                                self.map.insert(key.into(), Value::new(value.into(), None));
                            } else {
                                return Err(io::Error::other(format!(
                                    "unable to parse {rdb_type}"
                                )));
                            }
                        }
                    },
                    Err(e) => return Err(e.into()),
                },
                None => {
                    return Ok(None);
                }
                _ => {
                    src.advance(1);
                }
            }
        }
    }
}
