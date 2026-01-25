use super::header::{HEADER_BYTES_SIZE, Header};
use super::question::Question;
use super::rr::Rr;

pub struct Response {
    pub header: Header,
    pub question: Question,
    pub answer: Vec<Rr>,
}

pub const RESPONSE_BYTES_SIZE: usize = 512;

impl Response {
    pub fn new(header: Header, question: Question, answer: Vec<Rr>) -> Self {
        Response {
            header,
            question,
            answer,
        }
    }

    pub fn to_buf(&self) -> [u8; RESPONSE_BYTES_SIZE] {
        let mut buf: [u8; RESPONSE_BYTES_SIZE] = [0; RESPONSE_BYTES_SIZE];

        let header_buf = self.header.join();

        for (i, e) in buf[..HEADER_BYTES_SIZE].iter_mut().enumerate() {
            *e = header_buf[i];
        }

        let mut question_vec = self.question.join();
        let mut q_iter = question_vec.iter_mut();

        let mut answer_iter = self.answer.iter().flat_map(|rr| rr.join());

        for b in buf[HEADER_BYTES_SIZE..].iter_mut() {
            if let Some(qb) = q_iter.next() {
                *b = *qb;

                continue;
            }

            if let Some(rrb) = answer_iter.next() {
                *b = rrb;
            }
        }

        buf
    }
}
