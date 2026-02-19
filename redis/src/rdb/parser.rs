use crate::rdb::opcodes::OpCode;

pub struct RdbCodec {
    pub current_opcode: Option<OpCode>,
    pub is_header_read: bool,
}

impl RdbCodec {
    pub fn new() -> Self {
        Self {
            current_opcode: None,
            is_header_read: false,
        }
    }
}
