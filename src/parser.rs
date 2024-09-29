use chumsky::{input::SpannedInput, prelude::*};

use crate::ast::*;

fn section_parser<'a>(
) -> impl Parser<'a, &'a str, Vec<(&'a str, SimpleSpan<usize>)>, extra::Err<Rich<'a, char>>> {
    let double_linebreak = just('\r')
        .or_not()
        .then(just('\n'))
        .then(just('\r').or_not())
        .then(just('\n'));

    let paragraphs = any()
        .and_is(double_linebreak.not())
        .repeated()
        .to_slice()
        .map_with(|section: &str, extra| (section, extra.span()))
        .separated_by(double_linebreak)
        .collect();

    paragraphs
}

type SectionsInput<'a> = SpannedInput<&'a str, SimpleSpan, &'a [(&'a str, SimpleSpan)]>;

fn parser<'a>() -> impl Parser<'a, SectionsInput<'a>, Vec<ChefRecipe>> {
    let title = any().map(|s: &str| s.to_string());

    let ingredient = any()
        .map(|s| Ingredient {
            initial_value: Some(420),
            measure_type: None,
            measure: None,
            name: s,
        })
        .separated_by(just('\n'))
        .map_with(Spanned::map_extra);
    let ingredients = just("Ingredients.").ignored().then(ingredient);
    let comments = any().and_is(ingredients.not()).repeated().at_least(0);
    let method = any().separated_by(just('\n')).map_with(Spanned::map_extra);
    let methods = just("Method.").padded().ignore_then(method);
    let recipe = title
        .then_ignore(just("Ingredients.").padded())
        .then(ingredients)
        .then(comments)
        .then(methods)
        .map(|(title, ingredients, comments, methods)| ChefRecipe {
            title: title.0 .0,
            ingredients,
            comments,
            cooking_time: None,
            oven_temperature: None,
            method: vec![],
        });
}

pub fn parse(input: &str) -> Vec<ChefRecipe> {
    let sections = section_parser().parse(input).into_result().unwrap();
    let eoi = SimpleSpan::new(input.len(), input.len());
    parser().parse(sections.spanned(eoi)).into_result().unwrap()
}
