mod enums;
mod util;

use std::error::Error;

pub use enums::{Class, QType};
pub mod definitions;

use definitions::{
    header::{Header, HeaderField},
    question::Question,
    request::Request,
    response::Response,
    rr::Rr,
};

pub fn reply(buf: &[u8; 512]) -> Result<[u8; 512], Box<dyn Error>> {
    let mut header = Header::from_bytes(buf)?;

    header.set_header(HeaderField::QR(1));
    header.set_header(HeaderField::AA(0));
    header.set_header(HeaderField::TC(0));
    header.set_header(HeaderField::RA(0));
    header.set_header(HeaderField::Z(0));

    if let HeaderField::Opcode(opcode) = header.opcode {
        header.set_header(HeaderField::Rcode(if opcode == 0 { 0 } else { 4 }));
    }

    header.set_header(HeaderField::Nscount(0));
    header.set_header(HeaderField::Arcount(0));

    let question = Question::from_bytes(buf, None)?;

    header.set_header(HeaderField::Qdcount(
        question.records.len().try_into().unwrap_or(0),
    ));

    let mut rrs = vec![];

    for record in &question.records {
        let rr = Rr::new(&record.name, record.qtype, record.class, 60, "8.8.8.8")?;

        rrs.push(rr);
    }

    header.set_header(HeaderField::Ancount(rrs.len().try_into().unwrap_or(0)));

    let message = Response::new(header, question, rrs);

    Ok(message.to_buf())
}

pub fn parse_into_req(buf: &[u8; 512]) -> Result<Request, Box<dyn Error>> {
    let header = Header::from_bytes(buf)?;
    let question = Question::from_bytes(buf, None)?;

    Ok(Request::new(header, question))
}

pub fn parse_into_res(buf: &[u8; 512]) -> Result<Response, Box<dyn Error>> {
    let header = Header::from_bytes(buf)?;
    let question = Question::from_bytes(buf, Some(header.qdcount.to_number().unwrap()))?;
    let rrs = Rr::from_bytes_multiple(&buf[question.offset..])?;

    Ok(Response::new(header, question, rrs))
}
