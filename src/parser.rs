use chumsky::prelude::*;

use crate::ast::*;

fn line_break<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> {
    just('\r').or_not().then(just('\n')).to_slice()
}

fn double_line_break<'a>() -> impl Parser<'a, &'a str, &'a str, extra::Err<Rich<'a, char>>> {
    line_break().then(line_break()).to_slice()
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
        .or(just("tsp").map(|_| MeasureUnit::Teaspoons))
        .or(just("teaspoon").map(|_| MeasureUnit::Teaspoons))
        .or(just("teaspoons").map(|_| MeasureUnit::Teaspoons))
        .or(just("tbsp").map(|_| MeasureUnit::Tablespoons))
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
        .then(
            measure_type()
                .padded()
                .or_not()
                .then(measure_unit().padded())
                .or_not(),
        )
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

fn serves_instruction<'a>() -> impl Parser<'a, &'a str, usize, extra::Err<Rich<'a, char>>> {
    just("Serves ")
        .ignore_then(text::int(10).map(|s: &str| s.parse().unwrap()))
}

fn instruction<'a>(
) -> impl Parser<'a, &'a str, Spanned<CookingInstruction<'a>>, extra::Err<Rich<'a, char>>> {
    let dot = just('.').ignored();
    let ingredient_name = || {
        any()
            .and_is(line_break().not())
            .and_is(dot.not())
            .and_is(just(" into ").not())
            .and_is(just(" from ").not())
            .and_is(just(" to ").not())
            .and_is(just(" until ").not())
            .repeated()
            .to_slice()
    };
    let verb = || text::ascii::ident().map(Verb);
    // Take ingredient from refrigerator.
    just("Take ")
        .ignore_then(ingredient_name())
        .then_ignore(
            just(" from ")
                .then(just("the ").or_not())
                .then(just("refrigerator")),
        )
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
        .or(
            // Fold ingredient into [nth] mixing bowl.
            just("Fold ")
                .ignore_then(ingredient_name())
                .then_ignore(just(" into "))
                .then_ignore(just("the ").or_not())
                .then(nth().or_not())
                .then_ignore(just("mixing bowl").padded())
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Fold(ingredient, mixing_bowl.unwrap_or(0))
                }),
        )
        .or(
            // Add dry ingredients [to [nth] mixing bowl].
            just("Add dry ingredients")
                .ignore_then(
                    just(" to ")
                        .then(just("the ").or_not())
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl").or_not())
                        .or_not(),
                )
                .map(|mixing_bowl| {
                    CookingInstruction::AddDryIngredients(mixing_bowl.flatten().unwrap_or(0))
                }),
        )
        .or(
            // Add ingredient [to [nth] mixing bowl].
            just("Add ")
                .ignore_then(ingredient_name())
                .then(
                    just(" to ")
                        .then(just("the ").or_not())
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl").or_not())
                        .or_not(),
                )
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Add(ingredient, mixing_bowl.flatten().unwrap_or(0))
                }),
        )
        .or(
            // Remove ingredient [from [nth] mixing bowl].
            just("Remove ")
                .ignore_then(ingredient_name())
                .then(
                    just(" from ")
                        .then(just("the ").or_not())
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl").or_not())
                        .or_not(),
                )
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Remove(ingredient, mixing_bowl.flatten().unwrap_or(0))
                }),
        )
        .or(
            // Combine ingredient [into [nth] mixing bowl].
            just("Combine ")
                .ignore_then(ingredient_name())
                .then(
                    just(" into ")
                        .then(just("the ").or_not())
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl").or_not())
                        .or_not(),
                )
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Combine(ingredient, mixing_bowl.flatten().unwrap_or(0))
                }),
        )
        .or(
            // Divide ingredient [into [nth] mixing bowl].
            just("Divide ")
                .ignore_then(ingredient_name())
                .then(
                    just(" into ")
                        .then(just("the ").or_not())
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl").or_not())
                        .or_not(),
                )
                .map(|(ingredient, mixing_bowl)| {
                    CookingInstruction::Divide(ingredient, mixing_bowl.flatten().unwrap_or(0))
                }),
        )
        .or(
            // Liquefy | Liquify contents of the [nth] mixing bowl.
            just("Liquefy ")
                .or(just("Liquify "))
                .ignore_then(just("contents of "))
                .ignore_then(just("the ").or_not())
                .ignore_then(nth().or_not())
                .then_ignore(just("mixing bowl"))
                .map(|bowl| CookingInstruction::LiquefyContents(bowl.unwrap_or(0))),
        )
        .or(
            // Liquefy | Liquify ingredient.
            just("Liquefy ")
                .or(just("Liquify "))
                .ignore_then(ingredient_name())
                .map(CookingInstruction::Liquefy),
        )
        .or(
            // Stir [the [nth] mixing bowl] for number minutes.
            just("Stir ")
                .ignore_then(just("the ").or_not())
                .ignore_then(nth().or_not())
                .then_ignore(just("mixing bowl ").or_not())
                .then_ignore(just("for "))
                .then(text::int(10).map(|s: &str| s.parse().unwrap()))
                .then_ignore(just(" minutes"))
                .map(|(bowl, minutes)| CookingInstruction::Stir(bowl.unwrap_or(0), minutes)),
        )
        .or(
            // Stir ingredient into the [nth] mixing bowl.
            just("Stir ")
                .ignore_then(ingredient_name())
                .then_ignore(just(" into "))
                .then_ignore(just("the ").or_not())
                .then(nth().or_not())
                .then_ignore(just("mixing bowl"))
                .map(|(ingredient, bowl)| {
                    CookingInstruction::StirIngredient(ingredient, bowl.unwrap_or(0))
                }),
        )
        .or(
            // Mix [the [nth] mixing bowl] well.
            just("Mix ")
                .ignore_then(
                    just("the ")
                        .or_not()
                        .ignore_then(nth().or_not())
                        .then_ignore(just("mixing bowl "))
                        .or_not(),
                )
                .then_ignore(just("well"))
                .map(|bowl| CookingInstruction::Mix(bowl.flatten().unwrap_or(0))),
        )
        .or(
            // Clean [nth] mixing bowl.
            just("Clean ")
                .ignore_then(just("the ").or_not())
                .ignore_then(nth().or_not())
                .then_ignore(just("mixing bowl"))
                .map(|bowl| CookingInstruction::Clean(bowl.unwrap_or(0))),
        )
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
        .or(
            // Set aside.
            just("Set aside").to(CookingInstruction::SetAside),
        )
        .or(
            // Serve with auxiliary-recipe.
            just("Serve with ")
                .ignore_then(ingredient_name().padded() /* misleading, we just want the rest of the instruction as name */)
                .map(CookingInstruction::ServeWith),
        )
        .or(
            // Refrigerate [for number hours].
            just("Refrigerate")
                .ignore_then(
                    just(" for ")
                    .ignore_then(text::int(10).map(|s: &str| s.parse().unwrap()))
                    .then_ignore(just(" hours"))
                    .or_not()
                )
                .map(CookingInstruction::Refrigerate)
        )
        .or(
            serves_instruction().map(CookingInstruction::Serves)
        )
        .or(
            // Verb [the ingredient] until verbed.
            verb()
                .then(just(" "))
                .ignore_then(
                    just("the ").or_not()
                        .ignore_then(ingredient_name())
                        .then_ignore(just(" "))
                        .or_not(),
                )
                .then_ignore(just("until "))
                .then(verb())
                .map(|(ingredient, until)| CookingInstruction::VerbUntil(ingredient, until)),
        )
        .or(
            // Verb the ingredient.
            verb()
                .then_ignore(just(" the").or_not())
                .then_ignore(just(" "))
                .then(ingredient_name())
                .map(|(verb, ingredient)| CookingInstruction::Verb(verb, ingredient)),
        )
        
        .map_with(Spanned::from_with_extra)
}

fn parser<'a>() -> impl Parser<'a, &'a str, Vec<ChefRecipe<'a>>, extra::Err<Rich<'a, char>>> {
    let title = any().and_is(line_break().not()).repeated().to_slice();
    let ingredients_header = || just("Ingredients.").then(line_break());

    let comments = any()
        .and_is(ingredients_header().not())
        .and_is(double_line_break().not())
        .repeated()
        .to_slice();

    let ingredients = ingredient().repeated().collect();

    let cooking_time_and_stuff = just("Cooking time: ").then(any().and_is(double_line_break().not()).repeated());

    let method_header = just("Method.").then(line_break());

    let instructions = instruction()
        .separated_by(just(".").then(just(" ").or(line_break())))
        .collect();

    let serves = double_line_break().ignore_then(serves_instruction())
        .then_ignore(just("."))
        .map_with(Spanned::from_with_extra)
        .or_not();

    title
        .then_ignore(double_line_break())
        .then(comments.then_ignore(double_line_break()).or_not())
        .then_ignore(ingredients_header())
        .then(ingredients)
        .then_ignore(line_break())
        .then_ignore(cooking_time_and_stuff.then(double_line_break()).or_not())
        .then_ignore(method_header)
        .then(instructions)
        .then_ignore(just("."))
        .then(serves)
        .map(
            |((((title, comments), ingredients), instructions), serves)| ChefRecipe {
                title,
                comments: comments.unwrap_or_default(),
                ingredients,
                instructions,
                serves,
                cooking_time: None,     // TODO
                oven_temperature: None, // TODO
            },
        )
        .separated_by(double_line_break())
        .collect()
        .padded()
}

pub fn parse<'a>(input: &'a str) -> Result<Vec<ChefRecipe>, Vec<Rich<'a, char>>> {
    parser().parse(input).into_result()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    macro_rules! parse {
        ($parser:expr, $input:expr) => {
            match $parser.parse($input).into_result() {
                Ok(ast) => ast,
                Err(errors) => {
                    use ariadne::{sources, Color, Label, Report, ReportKind};
                    // Allow stdout to calm down to avoid interleaving output
                    std::thread::sleep(std::time::Duration::from_millis(10));

                    let filename = "<inline>";
                    for e in errors {
                        Report::build(ReportKind::Error, filename.clone(), e.span().start)
                            .with_message(e.to_string())
                            .with_label(
                                Label::new((filename.clone(), e.span().into_range()))
                                    .with_message(e.reason().to_string())
                                    .with_color(Color::Red),
                            )
                            .finish()
                            .print(sources([(filename.clone(), $input.to_string())]))
                            .unwrap();
                    }
                    panic!("Failed to parse input");
                }
            }
        };
    }

    #[test]
    fn test_ingredient() {
        let input = "1 heaped tsp flour\n";
        let result = parse!(ingredient(), input);
        assert_eq!(
            result.value(),
            &Ingredient {
                initial_value: Some(1),
                measure: Some(Measure::new(
                    MeasureUnit::Teaspoons,
                    Some(MeasureType::Heaped)
                )),
                name: "flour"
            }
        );
    }

    #[test]
    fn test_take_instruction() {
        let input = "Take boiled flour from refrigerator";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Take("boiled flour"));
    }

    #[test]
    fn test_put_instruction() {
        let input = "Put galvanized steel beams into mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::Put("galvanized steel beams", 0)
        );

        let input = "Put galvanized steel beams into the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::Put("galvanized steel beams", 3)
        );
    }

    #[test]
    fn test_fold_instruction() {
        let input = "Fold flour into mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Fold("flour", 0));

        let input = "Fold flour into the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Fold("flour", 3));
    }

    #[test]
    fn test_add_instruction() {
        let input = "Add flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Add("flour", 0));

        let input = "Add flour to the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Add("flour", 3));
    }

    #[test]
    fn test_remove_instruction() {
        let input = "Remove flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Remove("flour", 0));

        let input = "Remove flour from the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Remove("flour", 3));
    }

    #[test]
    fn test_combine_instruction() {
        let input = "Combine flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Combine("flour", 0));

        let input = "Combine flour into the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Combine("flour", 3));
    }

    #[test]
    fn test_divide_instruction() {
        let input = "Divide flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Divide("flour", 0));

        let input = "Divide flour into the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Divide("flour", 3));
    }

    #[test]
    fn test_add_dry_ingredients_instruction() {
        let input = "Add dry ingredients";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::AddDryIngredients(0));

        let input = "Add dry ingredients to the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::AddDryIngredients(3));
    }

    #[test]
    fn test_liqify_instruction() {
        let input = "Liquefy contents of the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::LiquefyContents(3));

        let input = "Liquefy contents of the mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::LiquefyContents(0));

        let input = "Liquify contents of the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::LiquefyContents(3));

        let input = "Liquify contents of the mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::LiquefyContents(0));
    }

    #[test]
    fn test_liqify_ingredient_instruction() {
        let input = "Liquefy flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Liquefy("flour"));

        let input = "Liquify flour";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Liquefy("flour"));
    }

    #[test]
    fn test_stir_bowl_instruction() {
        let input = "Stir the 3rd mixing bowl for 5 minutes";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Stir(3, 5));

        let input = "Stir the mixing bowl for 5 minutes";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Stir(0, 5));

        let input = "Stir for 5 minutes";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Stir(0, 5));
    }

    #[test]
    fn test_stir_ingredient_instruction() {
        let input = "Stir flour into the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::StirIngredient("flour", 3)
        );

        let input = "Stir flour into the mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::StirIngredient("flour", 0)
        );
    }

    #[test]
    fn test_mix_instruction() {
        let input = "Mix well";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Mix(0));

        let input = "Mix the mixing bowl well";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Mix(0));

        let input = "Mix the 3rd mixing bowl well";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Mix(3));
    }

    #[test]
    fn test_clean_instruction() {
        let input = "Clean the 3rd mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Clean(3));

        let input = "Clean the mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Clean(0));

        let input = "Clean mixing bowl";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Clean(0));
    }

    #[test]
    fn test_pour_instruction() {
        let input = "Pour contents of the 3rd mixing bowl into the 5th baking dish";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Pour(3, 5));

        let input = "Pour contents of the mixing bowl into the 5th baking dish";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Pour(0, 5));

        let input = "Pour contents of the 3rd mixing bowl into the baking dish";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Pour(3, 0));

        let input = "Pour contents of the mixing bowl into the baking dish";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Pour(0, 0));
    }

    #[test]
    fn test_verb_until_instruction() {
        let input = "Whisk the flour until whisked";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::VerbUntil(Some("flour"), Verb("whisked"))
        );

        let input = "Whisk until whisked";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::VerbUntil(None, Verb("whisked"))
        );

        let input = "Heat white sugar until melted";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::VerbUntil(Some("white sugar"), Verb("melted"))
        );
    }

    #[test]
    fn test_verb_instruction() {
        let input = "Whisk the flour";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::Verb(Verb("Whisk"), "flour")
        );

        let input = "Whisk caramelized camels";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::Verb(Verb("Whisk"), "caramelized camels")
        );
    }

    #[test]
    fn test_set_aside_instruction() {
        let input = "Set aside";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::SetAside);
    }

    #[test]
    fn test_serve_with_instruction() {
        let input = "Serve with coffee";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::ServeWith("coffee"));

        let input = "Serve with Carameled Apple";
        let result = parse!(instruction(), input);
        assert_eq!(
            result.value(),
            &CookingInstruction::ServeWith("Carameled Apple")
        );
    }

    #[test]
    fn test_refrigerate_instruction() {
        let input = "Refrigerate";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Refrigerate(None));

        let input = "Refrigerate for 2 hours";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Refrigerate(Some(2)));
    }

    #[test]
    fn test_serves_instruction() {
        let input = "Serves 5";
        let result = parse!(instruction(), input);
        assert_eq!(result.value(), &CookingInstruction::Serves(5));
    }

    #[test]
    fn test_parse_minimal_recipe() {
        let input = r#"
Quine Relay Coffee.

Ingredients.
10 g caffeine 10

Method.
Serve with Quine Relay Coffee.
        "#;
        parse!(parser(), input);
    }

    #[test]
    fn test_problematic_recipe() {
        let input = r#"
Caramel Sauce.

Ingredients.
1 cup white sugar
1 cup brown sugar
1 vanilla bean

Method.
Put vanilla bean into mixing bowl. Refrigerate. Heat white sugar until melted.
        "#.trim();
        parse!(parser(), input);
    }

    #[test]
    fn test_with_cooking_time_and_stuff() {
        let input = r#"
Caramel Sauce.

Ingredients.
1 cup white sugar
1 cup brown sugar
1 vanilla bean

Cooking time: 1 hour.
Pre-heat oven to 200 degrees Celcius.

Method.
Put vanilla bean into mixing bowl. Refrigerate. Heat white sugar until melted.
        "#.trim();
        parse!(parser(), input);
    }

    #[test]
    fn test_with_serves_instruction_in_list() {
        let input = r#"
Moose gulasch.

Ingredients.
1 moose

Method.
Eat moose. Serves 1.

Serves 1.
"#.trim();
        let recipe = parse!(parser(), input);
        let recipe = recipe.first().unwrap();
        let instructions = recipe.instructions.iter().map(Spanned::value).collect::<Vec<_>>();
        assert_eq!(instructions, vec![
            &CookingInstruction::Verb(Verb("Eat"), "moose"),
            &CookingInstruction::Serves(1)
        ]);
    }
}
