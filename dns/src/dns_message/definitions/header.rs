use std::{error::Error, fmt};

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum HeaderField {
    ID(u16),
    QR(u8),
    Opcode(u8),
    AA(u8),
    TC(u8),
    RD(u8),
    RA(u8),
    Z(u8),
    Rcode(u8),
    Qdcount(u16),
    Ancount(u16),
    Nscount(u16),
    Arcount(u16),
}

impl HeaderField {
    pub fn to_number(&self) -> Option<u16> {
        match *self {
            Self::ID(id) => Some(id),
            Self::Qdcount(qdcount) => Some(qdcount),
            _ => None,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Header {
    pub id: HeaderField,
    pub qr: HeaderField,
    pub opcode: HeaderField,
    pub aa: HeaderField,
    pub tc: HeaderField,
    pub rd: HeaderField,
    pub ra: HeaderField,
    pub z: HeaderField,
    pub rcode: HeaderField,
    pub qdcount: HeaderField,
    pub ancount: HeaderField,
    pub nscount: HeaderField,
    pub arcount: HeaderField,
}

#[derive(Debug)]
pub enum HeaderParseErrorKind {
    BadHeaderLen(usize),
}

impl fmt::Display for HeaderParseErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let fmt_message = match self {
            Self::BadHeaderLen(size) => {
                format!("Expected the size of {HEADER_BYTES_SIZE} bytes, received {size}")
            }
        };

        write!(f, "{}", fmt_message)
    }
}

#[derive(Debug)]
pub struct HeaderParseError {
    pub kind: HeaderParseErrorKind,
    pub source_error: Option<Box<dyn Error>>,
}

impl HeaderParseError {
    fn from_kind(kind: HeaderParseErrorKind, source_error: Option<Box<dyn Error>>) -> Self {
        Self { kind, source_error }
    }

    fn bad_header_len(size: usize, source_error: Option<Box<dyn Error>>) -> Self {
        HeaderParseError::from_kind(HeaderParseErrorKind::BadHeaderLen(size), source_error)
    }

    fn source_error_to_string(&self) -> String {
        match &self.source_error {
            Some(error) => error.to_string(),
            None => "<nil>".to_string(),
        }
    }
}

impl Error for HeaderParseError {}

impl fmt::Display for HeaderParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Unable to parse HEADER section: {}. Source error: {}",
            self.kind,
            self.source_error_to_string()
        )
    }
}

pub const HEADER_BYTES_SIZE: usize = 12;

impl Header {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, HeaderParseError> {
        if bytes.len() < HEADER_BYTES_SIZE {
            return Err(HeaderParseError::bad_header_len(bytes.len(), None));
        }

        Ok(Header {
            id: HeaderField::ID(((bytes[0] as u16) << 8) | (bytes[1] as u16)),
            qr: HeaderField::QR(bytes[2] >> 7),
            opcode: HeaderField::Opcode((bytes[2] >> 3) & 0xF),
            aa: HeaderField::AA((bytes[2] >> 2) & 1),
            tc: HeaderField::TC((bytes[2] >> 1) & 1),
            rd: HeaderField::RD(bytes[2] & 1),
            ra: HeaderField::RA(bytes[3] >> 7),
            z: HeaderField::Z((bytes[3] >> 4) & 0x7),
            rcode: HeaderField::Rcode(bytes[3] & 0xF),
            qdcount: HeaderField::Qdcount(((bytes[4] as u16) << 8) | (bytes[5] as u16)),
            ancount: HeaderField::Ancount(((bytes[6] as u16) << 8) | (bytes[7] as u16)),
            nscount: HeaderField::Nscount(((bytes[8] as u16) << 8) | (bytes[9] as u16)),
            arcount: HeaderField::Arcount(((bytes[10] as u16) << 8) | (bytes[11] as u16)),
        })
    }

    pub fn set_header(&mut self, header: HeaderField) {
        match header {
            HeaderField::ID(_) => self.id = header,
            HeaderField::QR(_) => self.qr = header,
            HeaderField::Opcode(_) => self.opcode = header,
            HeaderField::AA(_) => self.aa = header,
            HeaderField::TC(_) => self.tc = header,
            HeaderField::RD(_) => self.rd = header,
            HeaderField::RA(_) => self.ra = header,
            HeaderField::Z(_) => self.z = header,
            HeaderField::Rcode(_) => self.rcode = header,
            HeaderField::Qdcount(_) => self.qdcount = header,
            HeaderField::Ancount(_) => self.ancount = header,
            HeaderField::Nscount(_) => self.nscount = header,
            HeaderField::Arcount(_) => self.arcount = header,
        }
    }

    pub fn join(&self) -> [u8; HEADER_BYTES_SIZE] {
        let mut buf: [u8; HEADER_BYTES_SIZE] = [0; HEADER_BYTES_SIZE];

        let header_vec = [
            &self.id,
            &self.qr,
            &self.opcode,
            &self.aa,
            &self.tc,
            &self.rd,
            &self.ra,
            &self.z,
            &self.rcode,
            &self.qdcount,
            &self.ancount,
            &self.nscount,
            &self.arcount,
        ];

        for header_field in header_vec.iter() {
            match header_field {
                HeaderField::ID(id) => {
                    buf[0] = (id >> 8) as u8;
                    buf[1] = (id & 0xFF) as u8;
                }
                HeaderField::QR(qr) => {
                    buf[2] |= (qr & 1) << 7;
                }
                HeaderField::Opcode(opcode) => {
                    buf[2] |= (opcode & 0xF) << 3;
                }
                HeaderField::AA(aa) => {
                    buf[2] |= (aa & 1) << 2;
                }
                HeaderField::TC(tc) => {
                    buf[2] |= (tc & 1) << 1;
                }
                HeaderField::RD(rd) => {
                    buf[2] |= rd & 1;
                }
                HeaderField::RA(ra) => {
                    buf[3] |= (ra & 1) << 7;
                }
                HeaderField::Z(z) => {
                    buf[3] |= (z & 0x7) << 4;
                }
                HeaderField::Rcode(rcode) => {
                    buf[3] |= rcode & 0xF;
                }
                HeaderField::Qdcount(qdcount) => {
                    buf[4] = (qdcount >> 8) as u8;
                    buf[5] = (qdcount & 0xFF) as u8;
                }
                HeaderField::Ancount(ancount) => {
                    buf[6] = (ancount >> 8) as u8;
                    buf[7] = (ancount & 0xFF) as u8;
                }
                HeaderField::Nscount(nscount) => {
                    buf[8] = (nscount >> 8) as u8;
                    buf[9] = (nscount & 0xFF) as u8;
                }
                HeaderField::Arcount(arcount) => {
                    buf[10] = (arcount >> 8) as u8;
                    buf[11] = (arcount & 0xFF) as u8;
                }
            }
        }

        buf
    }
}
