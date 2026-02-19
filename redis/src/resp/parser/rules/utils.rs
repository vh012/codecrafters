use crate::resp::constants::END_SEQ;

pub(crate) fn is_end_seq(bytes: &[u8]) -> bool {
    bytes.ends_with(&END_SEQ)
}

pub(crate) fn get_end_seq_len() -> usize {
    END_SEQ.len()
}
