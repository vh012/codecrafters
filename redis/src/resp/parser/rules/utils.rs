const END_SEQ: [u8; 2] = [b'\r', b'\n'];

pub fn is_end_seq(bytes: &[u8]) -> bool {
    bytes.ends_with(&END_SEQ)
}

pub fn get_end_seq_len() -> usize {
    END_SEQ.len()
}
