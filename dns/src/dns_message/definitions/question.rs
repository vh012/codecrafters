use std::array::TryFromSliceError;
use std::error::Error;
use std::{fmt, u16};

use super::super::util::{get_pointer_offset, is_label_size, is_null_byte, is_pointer};

use super::super::{Class, QType, util::name_to_vec8};

#[derive(Debug, Clone)]
pub struct QuestionRecord {
    pub name: String,
    pub qtype: QType,
    pub class: Class,
}

pub struct Question {
    pub records: Vec<QuestionRecord>,
    pub offset: usize,
}

#[derive(Debug)]
pub enum QuestionParseErrorKind {
    QuestionNameEmpty(usize),
    UnexpectedEof(usize),
    QtypeOrClassInvalid,
}

impl fmt::Display for QuestionParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt_message = match self {
            Self::QtypeOrClassInvalid => {
                "QTYPE or CLASS may be missed or improperly aligned".to_string()
            }
            Self::UnexpectedEof(idx) => {
                format!("QUESTION may have incorrect align and structure, index {idx}")
            }
            Self::QuestionNameEmpty(idx) => {
                format!("QUESTION may have incorrect align and structure, index {idx}")
            }
        };

        write!(f, "{}", fmt_message)
    }
}

#[derive(Debug)]
pub struct QuestionParseError {
    pub kind: QuestionParseErrorKind,
    pub source_error: Option<Box<dyn Error>>,
}

impl QuestionParseError {
    fn from_kind(kind: QuestionParseErrorKind, source_error: Option<Box<dyn Error>>) -> Self {
        Self { kind, source_error }
    }

    fn unexpected_eof(idx: usize, source_error: Option<Box<dyn Error>>) -> Self {
        QuestionParseError::from_kind(QuestionParseErrorKind::UnexpectedEof(idx), source_error)
    }

    fn question_name_empty(idx: usize, source_error: Option<Box<dyn Error>>) -> Self {
        QuestionParseError::from_kind(QuestionParseErrorKind::QuestionNameEmpty(idx), source_error)
    }

    fn qtype_or_class_invalid(source_error: Option<Box<dyn Error>>) -> Self {
        QuestionParseError::from_kind(QuestionParseErrorKind::QtypeOrClassInvalid, source_error)
    }

    fn source_error_to_string(&self) -> String {
        match &self.source_error {
            Some(error) => error.to_string(),
            None => "<nil>".to_string(),
        }
    }
}

impl Error for QuestionParseError {}

impl fmt::Display for QuestionParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unable to parse QUESTION section: {}. Source error: {}",
            self.kind,
            self.source_error_to_string()
        )
    }
}

pub const BASE_OFFSET: usize = 12;
pub const QTYPE_CLASS_BYTES_LEN: usize = 4;

impl Question {
    pub fn from_records(records: Vec<QuestionRecord>) -> Self {
        Question { records, offset: 0 }
    }

    pub fn from_record(record: QuestionRecord) -> Self {
        Question {
            records: vec![record],
            offset: 0,
        }
    }

    pub fn from_bytes(
        bytes: &[u8],
        expected_number: Option<u16>,
    ) -> Result<Self, QuestionParseError> {
        if bytes.len() <= BASE_OFFSET {
            return Err(QuestionParseError::question_name_empty(BASE_OFFSET, None));
        }

        let (records, offset) =
            Question::parse_records_from_bytes(bytes.get(BASE_OFFSET..).unwrap(), expected_number)?;

        Ok(Self {
            records,
            offset: offset + BASE_OFFSET,
        })
    }

    fn parse_records_from_bytes(
        bytes: &[u8],
        expected_number: Option<u16>,
    ) -> Result<(Vec<QuestionRecord>, usize), QuestionParseError> {
        let mut records = vec![];
        let expected_number = expected_number.unwrap_or(u16::MAX);

        let mut offset = 0;

        for _ in 0..expected_number {
            let (name, end) = Question::parse_name_until_terminator(bytes, Some(offset));

            match name.is_empty() {
                true if records.is_empty() => {
                    return Err(QuestionParseError::question_name_empty(end, None));
                }
                true => break,
                false => {
                    let qtype_class_bytes: [u8; QTYPE_CLASS_BYTES_LEN] = bytes
                        .get(end + 1..end + 1 + QTYPE_CLASS_BYTES_LEN)
                        .ok_or_else(|| QuestionParseError::unexpected_eof(end, None))?
                        .try_into()
                        .map_err(|e: TryFromSliceError| {
                            QuestionParseError::qtype_or_class_invalid(Some(Box::new(e)))
                        })?;

                    records.push(QuestionRecord {
                        name,
                        qtype: QType::from_bytes(&qtype_class_bytes[0], &qtype_class_bytes[1])
                            .map_err(|_| {
                                QuestionParseError::qtype_or_class_invalid(None)
                            })?,
                        class: Class::from_bytes(&qtype_class_bytes[2], &qtype_class_bytes[3])
                            .map_err(|_| {
                                QuestionParseError::qtype_or_class_invalid(None)
                            })?,
                    });

                    offset = end + 1 + QTYPE_CLASS_BYTES_LEN;
                }
            }
        }

        Ok((records, offset))
    }

    fn parse_name_until_terminator(bytes: &[u8], offset: Option<usize>) -> (String, usize) {
        let offset = offset.unwrap_or(0);
        let mut name = String::new();

        let mut consume_label_times = 0;

        for i in offset..bytes.len() {
            if is_null_byte(&bytes[i]) {
                return (name, i);
            }

            if bytes.get(i + 1).is_some() {
                let maybe_pointer = ((bytes[i] as u16) << 8_u16) | (bytes[i + 1] as u16);

                if is_pointer(&maybe_pointer) {
                    let (name_from_pointer, _) = Question::parse_name_until_terminator(
                        bytes,
                        Some(get_pointer_offset(&maybe_pointer)),
                    );

                    if name.len() > 0 {
                        name.push('.');
                    }

                    name.push_str(&name_from_pointer);

                    return (name, i + 1);
                }
            }

            if is_label_size(&bytes[i]) && consume_label_times == 0 {
                consume_label_times = bytes[i];

                if name.len() > 0 {
                    name.push('.');
                }

                continue;
            }

            if consume_label_times > 0 {
                name.push(bytes[i] as char);

                consume_label_times -= 1;
            }
        }

        (name, bytes.len() - 1)
    }

    pub fn join(&self) -> Vec<u8> {
        let mut bytes = vec![];

        for name in &self.records {
            bytes.append(&mut Question::name_to_bytes(name));
        }

        bytes
    }

    fn name_to_bytes(name: &QuestionRecord) -> Vec<u8> {
        let mut name_vec8 = name_to_vec8(&name.name);
        let name_len = name_vec8.len();

        name_vec8.resize(name_len + 4, 0);

        name_vec8[name_len] = 0;
        name_vec8[name_len + 1] = name.qtype.to_byte();

        name_vec8[name_len + 2] = 0;
        name_vec8[name_len + 3] = name.class.to_byte();

        name_vec8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_valid_name() {
        let question = Question::from_bytes(
            &[3, b'f', b'o', b'o', 2, b'b', b'a', b'\0', 0, 1, 0, 1],
            None,
        )
        .unwrap();

        assert_eq!(question.records[0].name, "foo.ba");
        assert_eq!(question.records[0].qtype, QType::A);
        assert_eq!(question.records[0].class, Class::IN);
    }

    #[test]
    fn two_valid_names() {
        let question = Question::from_bytes(
            &[
                3, b'f', b'o', b'o', 2, b'b', b'a', b'\0', 0, 1, 0, 1, 1, b'r', 3, b'b', b'a',
                b'z', b'\0', 0, 1, 0, 1,
            ],
            None,
        )
        .unwrap();

        assert_eq!(question.records[0].name, "foo.ba");
        assert_eq!(question.records[0].qtype, QType::A);
        assert_eq!(question.records[0].class, Class::IN);
        assert_eq!(question.records[1].name, "r.baz");
        assert_eq!(question.records[1].qtype, QType::A);
        assert_eq!(question.records[1].class, Class::IN);
    }

    #[test]
    fn one_long_valid_name() {
        let question = Question::from_bytes(
            &[
                3, b'f', b'o', b'o', 3, b'b', b'a', b'r', 3, b'b', b'a', b'z', 1, b'f', 1, b'o', 1,
                b'o', b'\0', 0, 1, 0, 1,
            ],
            None,
        )
        .unwrap();

        assert_eq!(question.records[0].name, "foo.bar.baz.f.o.o");
        assert_eq!(question.records[0].qtype, QType::A);
        assert_eq!(question.records[0].class, Class::IN);
    }

    #[test]
    fn two_valid_names_with_pointer() {
        let question = Question::from_bytes(
            &[
                3, b'f', b'o', b'o', 2, b'b', b'a', b'\0', 0, 1, 0, 1, 3, b'b', b'a', b'z',
                0b11000000, 12, 0, 1, 0, 1,
            ],
            None,
        )
        .unwrap();

        assert_eq!(question.records[0].name, "foo.ba");
        assert_eq!(question.records[0].qtype, QType::A);
        assert_eq!(question.records[0].class, Class::IN);
        assert_eq!(question.records[1].name, "baz.foo.ba");
        assert_eq!(question.records[1].qtype, QType::A);
        assert_eq!(question.records[1].class, Class::IN);
    }

    #[test]
    fn three_valid_names_with_pointer_as_name() {
        let question = Question::from_bytes(
            &[
                3, b'f', b'o', b'o', 2, b'b', b'a', b'\0', 0, 1, 0, 1, 3, b'b', b'a', b'z',
                0b11000000, 12, 0, 1, 0, 1, 0b11000000, 12, 0, 1, 0, 1,
            ],
            None,
        )
        .unwrap();

        assert_eq!(question.records[0].name, "foo.ba");
        assert_eq!(question.records[0].qtype, QType::A);
        assert_eq!(question.records[0].class, Class::IN);
        assert_eq!(question.records[1].name, "baz.foo.ba");
        assert_eq!(question.records[1].qtype, QType::A);
        assert_eq!(question.records[1].class, Class::IN);
        assert_eq!(question.records[2].name, "foo.ba");
        assert_eq!(question.records[2].qtype, QType::A);
        assert_eq!(question.records[2].class, Class::IN);
    }
}
