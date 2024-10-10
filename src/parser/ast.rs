use chumsky::{input::MapExtra, span::SimpleSpan};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ChefRecipe<'a, T> {
    pub title: &'a str,
    pub comments: &'a str,
    pub ingredients: Vec<Spanned<Ingredient<'a>>>,
    pub cooking_time: Option<usize>,
    pub oven_temperature: Option<usize>,
    pub instructions: Vec<Spanned<T>>,
    pub serves: Option<Spanned<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Ingredient<'a> {
    pub initial_value: Option<usize>,
    pub measure: Option<Measure>,
    pub name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Verb<'a>(pub &'a str);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Measure {
    pub measure_type: Option<MeasureType>,
    pub unit: MeasureUnit,
}

impl Measure {
    pub fn new(unit: MeasureUnit, measure_type: Option<MeasureType>) -> Self {
        Self { measure_type, unit }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeasureType {
    Heaped,
    Level,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MeasureUnit {
    Grams,
    Kilograms,
    Pinches,
    Milliliters,
    Liters,
    Dashes,
    Cups,
    Teaspoons,
    Tablespoons,
}

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
