use crate::resp::parser::rules::types::BoxedRespParseRule;

pub(crate) mod rules;

pub struct RespCodec {
    pub rule: Option<BoxedRespParseRule>,
}

impl RespCodec {
    pub fn new() -> Self {
        Self { rule: None }
    }
}
