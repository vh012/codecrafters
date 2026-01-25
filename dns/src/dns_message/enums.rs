use std::fmt;

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum QType {
    A,
    NS,
    MX,
    Cname,
    Soa,
    MB,
    MG,
    MR,
    Null,
    Wks,
    Ptr,
    Hinfo,
    Minfo,
    Txt,
    Axfr,
    Mailb,
    Maila,
    Asterisk,
}

impl fmt::Display for QType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            QType::A => write!(f, "A"),
            QType::NS => write!(f, "NS"),
            QType::MX => write!(f, "NS"),
            QType::Cname => write!(f, "CNAME"),
            QType::Soa => write!(f, "SOA"),
            QType::MB => write!(f, "MB"),
            QType::MG => write!(f, "MG"),
            QType::MR => write!(f, "MR"),
            QType::Null => write!(f, "NULL"),
            QType::Wks => write!(f, "WKS"),
            QType::Ptr => write!(f, "PTR"),
            QType::Hinfo => write!(f, "HINFO"),
            QType::Minfo => write!(f, "MINFO"),
            QType::Txt => write!(f, "TXT"),
            QType::Axfr => write!(f, "AXFR"),
            QType::Mailb => write!(f, "MAILB"),
            QType::Maila => write!(f, "MAILA"),
            QType::Asterisk => write!(f, "ASTERISK"),
        }
    }
}

impl QType {
    pub fn from_bytes(first: &u8, second: &u8) -> Result<Self, String> {
        Ok(match ((*first as u16) << 8) | (*second as u16) {
            1 => QType::A,
            2 => QType::NS,
            15 => QType::MX,
            5 => QType::Cname,
            6 => QType::Soa,
            7 => QType::MB,
            8 => QType::MG,
            9 => QType::MR,
            10 => QType::Null,
            11 => QType::Wks,
            12 => QType::Ptr,
            13 => QType::Hinfo,
            14 => QType::Minfo,
            16 => QType::Txt,
            252 => QType::Axfr,
            253 => QType::Mailb,
            254 => QType::Maila,
            255 => QType::Asterisk,
            t => {
                return Err(format!("Unsupported QTYPE: {t}"));
            }
        })
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            QType::A => 1,
            QType::NS => 2,
            QType::MX => 15,
            QType::Cname => 5,
            QType::Soa => 6,
            QType::MB => 7,
            QType::MG => 8,
            QType::MR => 9,
            QType::Null => 10,
            QType::Wks => 11,
            QType::Ptr => 12,
            QType::Hinfo => 13,
            QType::Minfo => 14,
            QType::Txt => 16,
            QType::Axfr => 252,
            QType::Mailb => 253,
            QType::Maila => 254,
            QType::Asterisk => 255,
        }
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Class {
    IN,
    CS,
    CH,
    HS,
    Asterisk,
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Class::IN => write!(f, "IN"),
            Class::CS => write!(f, "CS"),
            Class::CH => write!(f, "CH"),
            Class::HS => write!(f, "HS"),
            Class::Asterisk => write!(f, "ASTERISK"),
        }
    }
}

impl Class {
    pub fn from_bytes(first: &u8, second: &u8) -> Result<Self, String> {
        Ok(match ((*first as u16) << 8) | (*second as u16) {
            1 => Class::IN,
            2 => Class::CS,
            15 => Class::CH,
            5 => Class::HS,
            255 => Class::Asterisk,
            c => {
                return Err(format!("Unsupported CLASS: {c}"));
            }
        })
    }

    pub fn to_byte(&self) -> u8 {
        match self {
            Class::IN => 1,
            Class::CS => 2,
            Class::CH => 15,
            Class::HS => 5,
            Class::Asterisk => 255,
        }
    }
}
