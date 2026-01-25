use super::super::util::{is_label_size, is_null_byte};

use std::{
    array::TryFromSliceError, convert::Infallible, error::Error, fmt, net::Ipv4Addr, str::FromStr,
};

use super::super::{
    Class, QType,
    util::{name_to_vec8, sanitize_name},
};

pub const SUPPORTED_QTYPES: [QType; 1] = [QType::A];
pub const QTYPE_CLASS_BYTES_LEN: usize = 4;
pub const TTL_BYTES_LEN: usize = 4;
pub const RDLEN_BYTES_LEN: usize = 2;

#[derive(PartialEq, Debug, Clone)]
pub struct Rr {
    name: String,
    rrtype: QType,
    class: Class,
    ttl: u32,
    rdlength: u16,
    rdata: Vec<u8>,
}

#[derive(Debug)]
pub enum RrParseErrorKind {
    RecordTypeIsNotSupported(QType),
    UnexpectedEmptyName(usize),
    UnexpectedEof(usize),
    QtypeOrClassInvalid,
    TtlInvalid,
    RdlenInvalid,
    UnexpectedRdlen(u16),
    UnexpectedRdata,
}

impl fmt::Display for RrParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt_message = match self {
            Self::RecordTypeIsNotSupported(qtype) => {
                format!("RR has unsupported QTYPE {qtype}, supported list: {SUPPORTED_QTYPES:?}")
            }
            Self::UnexpectedEmptyName(idx) => {
                format!("RR may have empty name or incorrect align, index: {idx}")
            }
            Self::UnexpectedEof(idx) => {
                format!("RR may have incorrect align and structure, index {idx}")
            }
            Self::QtypeOrClassInvalid => {
                "QTYPE or CLASS may be missed or improperly aligned".to_string()
            }
            Self::TtlInvalid => "TTL must be valid u32".to_string(),
            Self::RdlenInvalid => "TTL must be valid u16".to_string(),
            Self::UnexpectedRdlen(value) => format!(
                "Unexpected value was provided for RDLENGTH {value}, supported list: {SUPPORTED_QTYPES:?}"
            ),
            Self::UnexpectedRdata => "Unexpected value was provided for RDATA".to_string(),
        };

        write!(f, "{}", fmt_message)
    }
}

#[derive(Debug)]
pub struct RrParseError {
    pub kind: RrParseErrorKind,
    pub source_error: Option<Box<dyn Error>>,
}

impl RrParseError {
    fn from_kind(kind: RrParseErrorKind, source_error: Option<Box<dyn Error>>) -> Self {
        Self { kind, source_error }
    }

    fn record_type_is_not_supported(qtype: QType, source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(
            RrParseErrorKind::RecordTypeIsNotSupported(qtype),
            source_error,
        )
    }

    fn unexpected_empty_name(idx: usize, source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::UnexpectedEmptyName(idx), source_error)
    }

    fn unexpected_eof(idx: usize, source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::UnexpectedEof(idx), source_error)
    }

    fn qtype_or_class_invalid(source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::QtypeOrClassInvalid, source_error)
    }

    fn ttl_invalid(source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::TtlInvalid, source_error)
    }

    fn rdlen_invalid(source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::RdlenInvalid, source_error)
    }

    fn unexpected_rdlen(value: u16, source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::UnexpectedRdlen(value), source_error)
    }

    fn unexpected_rdata(source_error: Option<Box<dyn Error>>) -> Self {
        RrParseError::from_kind(RrParseErrorKind::UnexpectedRdata, source_error)
    }

    fn source_error_to_string(&self) -> String {
        match &self.source_error {
            Some(error) => error.to_string(),
            None => "<nil>".to_string(),
        }
    }
}

impl Error for RrParseError {}

impl fmt::Display for RrParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unable to parse RR section: {}. Source error: {}",
            self.kind,
            self.source_error_to_string()
        )
    }
}

impl Rr {
    pub fn new(
        name: &str,
        rrtype: QType,
        class: Class,
        ttl: u32,
        rdata: &str,
    ) -> Result<Self, RrParseError> {
        if !SUPPORTED_QTYPES.contains(&rrtype) {
            return Err(RrParseError::record_type_is_not_supported(rrtype, None));
        }

        let rdlength = Rr::resolve_rdlength(&rrtype)?;

        let rdata = Rr::convert_rdata_to_bytes(&rrtype, rdata)?;

        Ok(Rr {
            name: sanitize_name(name).map_err(|_| RrParseError::unexpected_empty_name(0, None))?,
            rrtype,
            class,
            ttl,
            rdlength,
            rdata,
        })
    }

    pub fn from_bytes_multiple(bytes: &[u8]) -> Result<Vec<Self>, RrParseError> {
        let mut rrs = vec![];

        let mut offset = 0;

        loop {
            let maybe_rr = Rr::from_bytes(bytes, Some(offset))?;

            match maybe_rr {
                Some((rr, end)) => {
                    rrs.push(rr);

                    offset += end;
                }
                None => break,
            }
        }

        Ok(rrs)
    }

    fn from_bytes(
        bytes: &[u8],
        offset: Option<usize>,
    ) -> Result<Option<(Self, usize)>, RrParseError> {
        let mut offset = offset.unwrap_or(0);
        let mut name = String::new();

        let mut consume_label_times = 0;

        for i in offset..bytes.len() {
            if is_null_byte(&bytes[i]) {
                offset += i + 1;

                break;
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

        if name.is_empty() {
            return Ok(None);
        }

        let qtype_class_bytes: [u8; QTYPE_CLASS_BYTES_LEN] = bytes
            .get(offset..offset + QTYPE_CLASS_BYTES_LEN)
            .ok_or_else(|| RrParseError::unexpected_eof(offset, None))?
            .try_into()
            .map_err(|e: TryFromSliceError| {
                RrParseError::qtype_or_class_invalid(Some(Box::new(e)))
            })?;

        let rrtype = QType::from_bytes(&qtype_class_bytes[0], &qtype_class_bytes[1])
            .map_err(|_| {
                RrParseError::qtype_or_class_invalid(None)
            })?;
        let class = Class::from_bytes(&qtype_class_bytes[2], &qtype_class_bytes[3])
            .map_err(|_| {
                RrParseError::qtype_or_class_invalid(None)
            })?;

        offset += QTYPE_CLASS_BYTES_LEN;

        let ttl_bytes: [u8; TTL_BYTES_LEN] = bytes
            .get(offset..offset + TTL_BYTES_LEN)
            .ok_or_else(|| RrParseError::unexpected_eof(offset, None))?
            .try_into()
            .map_err(|e: TryFromSliceError| RrParseError::ttl_invalid(Some(Box::new(e))))?;

        let ttl = (ttl_bytes[0] as u32) << 24
            | (ttl_bytes[1] as u32) << 16
            | (ttl_bytes[2] as u32) << 8
            | (ttl_bytes[3] as u32);

        offset += TTL_BYTES_LEN;

        let rdlen_bytes: [u8; RDLEN_BYTES_LEN] = bytes
            .get(offset..offset + RDLEN_BYTES_LEN)
            .ok_or_else(|| RrParseError::unexpected_eof(offset, None))?
            .try_into()
            .map_err(|e: TryFromSliceError| RrParseError::rdlen_invalid(Some(Box::new(e))))?;

        let rdlength = (rdlen_bytes[0] as u16) << 8 | rdlen_bytes[1] as u16;

        if rdlength != 4 {
            return Err(RrParseError::unexpected_rdlen(rdlength, None));
        }

        offset += RDLEN_BYTES_LEN;

        let rdata_bytes: &[u8] = bytes
            .get(offset..offset + (rdlength as usize))
            .ok_or_else(|| RrParseError::unexpected_eof(offset, None))?
            .try_into()
            .map_err(|e: Infallible| RrParseError::rdlen_invalid(Some(Box::new(e))))?;

        let rdata = Rr::convert_rdata_to_bytes(
            &rrtype,
            &Ipv4Addr::from_octets([
                rdata_bytes[0],
                rdata_bytes[1],
                rdata_bytes[2],
                rdata_bytes[3],
            ])
            .to_string(),
        )?;

        offset += rdlength as usize;

        Ok(Some((
            Rr {
                name,
                rrtype,
                class,
                ttl,
                rdlength,
                rdata,
            },
            offset,
        )))
    }

    pub fn join(&self) -> Vec<u8> {
        let mut rr = name_to_vec8(&self.name);
        let mut offset = rr.len();
        let bytes_after_name = QTYPE_CLASS_BYTES_LEN + TTL_BYTES_LEN + RDLEN_BYTES_LEN;

        rr.resize(offset + bytes_after_name + self.rdlength as usize, 0);

        rr[offset] = 0;
        rr[offset + 1] = self.rrtype.to_byte();

        rr[offset + 2] = 0;
        rr[offset + 3] = self.class.to_byte();

        offset += QTYPE_CLASS_BYTES_LEN;

        rr[offset] = (self.ttl >> 24) as u8;
        rr[offset + 1] = ((self.ttl >> 16) & 0xFF) as u8;
        rr[offset + 2] = ((self.ttl >> 8) & 0xFF) as u8;
        rr[offset + 3] = (self.ttl & 0xFF) as u8;

        offset += TTL_BYTES_LEN;

        rr[offset] = (self.rdlength >> 8) as u8;
        rr[offset + 1] = (self.rdlength & 0xFF) as u8;

        offset += RDLEN_BYTES_LEN;

        let mut rdata_iterator = self.rdata.iter();

        for b in rr[offset..].iter_mut() {
            *b = match rdata_iterator.next() {
                Some(ch) => *ch,
                None => break,
            };
        }

        rr
    }

    fn resolve_rdlength(rrtype: &QType) -> Result<u16, RrParseError> {
        match *rrtype {
            QType::A => Ok(4),
            _ => Err(RrParseError::record_type_is_not_supported(*rrtype, None)),
        }
    }

    fn convert_rdata_to_bytes(rrtype: &QType, rdata: &str) -> Result<Vec<u8>, RrParseError> {
        match *rrtype {
            QType::A => {
                let mut bytes = vec![b'\0'; 4];

                let ipv4: u32 = match Ipv4Addr::from_str(rdata) {
                    Ok(ipv4) => ipv4.to_bits(),
                    Err(e) => return Err(RrParseError::unexpected_rdata(Some(Box::new(e)))),
                };

                bytes[0] = (ipv4 >> 24) as u8;
                bytes[1] = ((ipv4 >> 16) & 0xFF) as u8;
                bytes[2] = ((ipv4 >> 8) & 0xFF) as u8;
                bytes[3] = (ipv4 & 0xFF) as u8;

                Ok(bytes)
            }
            _ => Err(RrParseError::record_type_is_not_supported(*rrtype, None)),
        }
    }
}
