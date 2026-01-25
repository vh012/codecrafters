use super::header::HeaderField;
use super::header::{HEADER_BYTES_SIZE, Header};
use super::question::Question;

pub struct Request {
    pub header: Header,
    pub question: Question,
}

pub const REQUEST_BYTES_SIZE: usize = 512;

impl Request {
    pub fn new(header: Header, question: Question) -> Self {
        Request { header, question }
    }

    pub fn to_buf(&self) -> [u8; REQUEST_BYTES_SIZE] {
        let mut buf: [u8; REQUEST_BYTES_SIZE] = [0; REQUEST_BYTES_SIZE];

        let header_buf = self.header.join();

        for (i, e) in buf[..HEADER_BYTES_SIZE].iter_mut().enumerate() {
            *e = header_buf[i];
        }

        let mut question_vec = self.question.join();
        let mut q_iter = question_vec.iter_mut();

        for b in buf[HEADER_BYTES_SIZE..].iter_mut() {
            if let Some(qb) = q_iter.next() {
                *b = *qb;

                continue;
            }
        }

        buf
    }

    pub fn split_into_multiple(&self) -> Vec<Self> {
        let mut requests = vec![];

        for name in &self.question.records {
            let mut header = self.header.clone();

            header.set_header(HeaderField::Qdcount(1));

            let question = Question::from_record(name.clone());

            requests.push(Request::new(header, question))
        }

        requests
    }
}
