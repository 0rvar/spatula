use chumsky::prelude::*;

use crate::ast::*;

fn line_break<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> {
    just('\r').or_not().then(just('\n')).ignored()
}

fn double_line_break<'a>() -> impl Parser<'a, &'a str, (), extra::Err<Rich<'a, char>>> {
    line_break().then(line_break()).ignored()
}

fn measure_unit<'a>() -> impl Parser<'a, &'a str, MeasureUnit, extra::Err<Rich<'a, char>>> {
    just("g")
        .map(|_| MeasureUnit::Grams)
        .or(just("kg").map(|_| MeasureUnit::Kilograms))
        .or(just("pinch").map(|_| MeasureUnit::Pinches))
        .or(just("pinches").map(|_| MeasureUnit::Pinches))
        .or(just("ml").map(|_| MeasureUnit::Milliliters))
        .or(just("l").map(|_| MeasureUnit::Liters))
        .or(just("dash").map(|_| MeasureUnit::Dashes))
        .or(just("dashes").map(|_| MeasureUnit::Dashes))
        .or(just("cup").map(|_| MeasureUnit::Cups))
        .or(just("cups").map(|_| MeasureUnit::Cups))
        .or(just("teaspoon").map(|_| MeasureUnit::Teaspoons))
        .or(just("teaspoons").map(|_| MeasureUnit::Teaspoons))
        .or(just("tablespoon").map(|_| MeasureUnit::Tablespoons))
        .or(just("tablespoons").map(|_| MeasureUnit::Tablespoons))
}

fn measure_type<'a>() -> impl Parser<'a, &'a str, MeasureType, extra::Err<Rich<'a, char>>> {
    just("heaped")
        .map(|_| MeasureType::Heaped)
        .or(just("level").map(|_| MeasureType::Level))
}

fn ingredient<'a>() -> impl Parser<'a, &'a str, Spanned<Ingredient<'a>>, extra::Err<Rich<'a, char>>>
{
    // [initial-value] [[measure-type] measure] ingredient-name
    let initial_value = text::int(10).map(|s: &str| s.parse().unwrap());
    let ingredient_name = any()
        .and_is(line_break().not())
        .repeated()
        .at_least(2)
        .to_slice();

    initial_value
        .or_not()
        .then(measure_type().or_not().then(measure_unit()).or_not())
        .then(ingredient_name)
        .then_ignore(line_break())
        .map(|((initial_value, measure), ingredient_name)| Ingredient {
            initial_value,
            measure: measure.map(|m| Measure::new(m.1, m.0)),
            name: ingredient_name,
        })
        .map_with(Spanned::from_with_extra)
}

fn nth<'a>() -> impl Parser<'a, &'a str, usize, extra::Err<Rich<'a, char>>> {
    text::int(10)
        .map(|s: &str| s.parse().unwrap())
        .then_ignore(just("th").or(just("st")).or(just("nd")).or(just("rd")))
        .padded()
}

fn instruction<'a>(
) -> impl Parser<'a, &'a str, Spanned<CookingInstruction<'a>>, extra::Err<Rich<'a, char>>> {
    let dot = just('.').ignored();
    let ingredient_name = || {
        any()
            .and_is(line_break().not())
            .and_is(dot.not())
            .and_is(just(" into ").not())
            .repeated()
            .to_slice()
    };
    // Take ingredient from refrigerator.
    // This reads a numeric value from STDIN into the ingredient named, overwriting any previous value.
    just("Take ")
        .ignore_then(ingredient_name())
        .then_ignore(just(" from refrigerator"))
        .map(CookingInstruction::Take)
        .or(
            // Put ingredient into [nth] mixing bowl.
            just("Put ")
                .ignore_then(ingredient_name())
                .then_ignore(just(" into "))
                .then_ignore(just("the ").or_not())
                .then(nth().or_not())
                .then_ignore(just("mixing bowl").padded())
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Put(ingredient, mixing_bowl.unwrap_or(0))
                }),
        )
        .or(just("Liquefy ")
            .or(just("Liquify "))
            .ignore_then(just("contents of "))
            .ignore_then(just("the ").or_not())
            .ignore_then(nth().or_not())
            .then_ignore(just("mixing bowl"))
            .map(|bowl| CookingInstruction::LiquefyContents(bowl.unwrap_or(0))))
        .or(just("Liquefy ")
            .or(just("Liquify "))
            .ignore_then(ingredient_name())
            .map(CookingInstruction::Liquefy))
        .or(
            // Pour contents of the [nth] mixing bowl into the [pth] baking dish.
            just("Pour ")
                .ignore_then(just("contents of "))
                .ignore_then(just("the ").or_not())
                .ignore_then(nth().or_not())
                .then_ignore(just("mixing bowl into the "))
                .then(nth().or_not())
                .then_ignore(just("baking dish"))
                .map(|(from, to)| CookingInstruction::Pour(from.unwrap_or(0), to.unwrap_or(0))),
        )
        .map_with(Spanned::from_with_extra)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<ChefRecipe<'a>>, extra::Err<Rich<'a, char>>> {
    let title = any()
        .and_is(double_line_break().not())
        .repeated()
        .to_slice();
    let ingredients_header = || just("Ingredients.").then(line_break());

    let comments = any()
        .and_is(ingredients_header().not())
        .and_is(double_line_break().not())
        .repeated()
        .to_slice();

    let ingredients = ingredient().repeated().collect();

    let method_header = just("Method.").then(line_break());

    let instructions = instruction().separated_by(just(". ")).collect();

    let serves = double_line_break()
        .ignore_then(just("Serves "))
        .ignore_then(text::int(10).map(|s: &str| s.parse().unwrap()))
        .then_ignore(just("."))
        .map_with(Spanned::from_with_extra)
        .or_not();

    title
        .then_ignore(double_line_break())
        .then(comments)
        .then_ignore(double_line_break())
        .then_ignore(ingredients_header())
        .then(ingredients)
        .then_ignore(line_break())
        .then_ignore(method_header)
        .then(instructions)
        .then_ignore(just("."))
        .then(serves)
        .map(
            |((((title, comments), ingredients), instructions), serves)| ChefRecipe {
                title,
                comments,
                ingredients,
                instructions,
                serves,
                cooking_time: None,     // TODO
                oven_temperature: None, // TODO
            },
        )
        .separated_by(double_line_break())
        .collect()
}

pub fn parse<'a>(input: &'a str) -> Result<Vec<ChefRecipe>, Vec<Rich<'a, char>>> {
    parser().parse(input).into_result()
}
