use chumsky::{error::Rich, span::SimpleSpan};

pub enum ParseError<'a> {
    FirstStage(Vec<Rich<'a, char>>),
    SecondStage(String, SimpleSpan<usize>),
}
