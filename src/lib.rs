use chumsky::span::SimpleSpan;

pub mod parser;
pub mod validator;

#[derive(Debug)]
pub struct SpatulaError {
    pub message: String,
    pub span: SimpleSpan<usize>,
}
