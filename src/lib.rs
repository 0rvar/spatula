use chumsky::span::SimpleSpan;

pub mod interpreter;
pub mod parser;
pub mod validator;

#[derive(Debug)]
pub struct SpatulaError {
    pub message: String,
    pub span: SimpleSpan,
}
impl SpatulaError {
    pub fn new<S>(message: S, span: SimpleSpan) -> Self
    where
        S: Into<String>,
    {
        Self {
            message: message.into(),
            span,
        }
    }
}
