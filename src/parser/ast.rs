use chumsky::{input::MapExtra, span::SimpleSpan};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChefRecipe<'a, T, I> {
    pub title: &'a str,
    pub comments: &'a str,
    pub ingredients: Vec<Spanned<I>>,
    pub cooking_time: Option<usize>,
    pub oven_temperature: Option<usize>,
    pub instructions: Vec<Spanned<T>>,
    pub serves: Option<Spanned<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Verb<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Spanned<T>(pub T, pub SimpleSpan<usize>);
impl<T> Spanned<T> {
    pub fn new(value: T, span: SimpleSpan<usize>) -> Self {
        Self(value, span)
    }
    pub fn from_with_extra<'a, E, I>(value: T, extra: &mut MapExtra<'a, '_, I, E>) -> Self
    where
        I: chumsky::input::Input<'a, Span = SimpleSpan<usize>>,
        E: chumsky::extra::ParserExtra<'a, I>,
    {
        Self(value, extra.span())
    }

    pub fn value(&self) -> &T {
        &self.0
    }

    pub fn into_value(self) -> T {
        self.0
    }

    pub fn span(&self) -> SimpleSpan<usize> {
        self.1
    }
}
