use std::collections::HashMap;

use chumsky::span::SimpleSpan;

use crate::parser::stage_two_ast::IngredientType;

use super::{
    stage_one_ast::{CookingIngredient, CookingInstruction},
    ChefProgram, ChefRecipe, Ingredient, Instruction, ParseError, Spanned, VerbLoop,
};

pub fn parse<'a>(
    input: Vec<ChefRecipe<'a, CookingInstruction<'a>, CookingIngredient<'a>>>,
) -> Result<ChefProgram<'a>, ParseError> {
    let mut functions = input.into_iter();
    let Some(main) = functions.next() else {
        return Err(ParseError::SecondStage(
            "No main recipe found".to_string(),
            SimpleSpan::new(0, 1),
        ));
    };
    let main = parse_recipe(main)?;
    let auxilary = functions
        .map(parse_recipe)
        .map(|recipe| recipe.map(|recipe| (recipe.title.to_lowercase(), recipe)))
        .collect::<Result<HashMap<String, _>, ParseError>>()?;
    Ok(ChefProgram { main, auxilary })
}

fn parse_recipe<'a>(
    recipe: ChefRecipe<'a, CookingInstruction<'a>, CookingIngredient<'a>>,
) -> Result<ChefRecipe<'a, Instruction<'a>, Ingredient<'a>>, ParseError> {
    let ChefRecipe {
        title,
        comments,
        ingredients,
        cooking_time,
        oven_temperature,
        instructions,
        serves,
    } = recipe;

    let instructions = parse_instructions(instructions)?;
    let ingredients = parse_ingredients(ingredients)?;

    Ok(ChefRecipe {
        title,
        comments,
        ingredients,
        cooking_time,
        oven_temperature,
        instructions,
        serves,
    })
}

fn parse_ingredients<'a>(
    ingredients: Vec<Spanned<CookingIngredient<'a>>>,
) -> Result<Vec<Spanned<Ingredient<'a>>>, ParseError> {
    ingredients
        .into_iter()
        .map(|Spanned(ingredient, span)| {
            let CookingIngredient {
                name,
                measure,
                initial_value,
            } = ingredient;
            let ingredient_type = IngredientType::parse(measure, &span)?;

            Ok(Spanned::new(
                Ingredient {
                    name,
                    initial_value,
                    r#type: ingredient_type,
                },
                span,
            ))
        })
        .collect::<Result<Vec<_>, ParseError>>()
}

fn parse_instructions<'a>(
    instructions: Vec<Spanned<CookingInstruction<'a>>>,
) -> Result<Vec<Spanned<Instruction<'a>>>, ParseError> {
    let mut instructions_iter = instructions.into_iter();
    let mut loop_stack: Vec<(VerbLoop, SimpleSpan<usize>)> = vec![];
    let mut instructions = vec![];

    loop {
        let Some(instruction) = instructions_iter.next() else {
            if let Some((current_loop, loop_span)) = loop_stack.last() {
                return Err(ParseError::SecondStage(
                    format!(
                        "Recipe ends during `{}` - matching `until` not found",
                        current_loop.verb.0
                    ),
                    loop_span.clone(),
                ));
            }
            break;
        };

        let Spanned(instruction, span) = instruction;
        macro_rules! spanned {
            ($instr:expr) => {
                Spanned::new($instr, span)
            };
        }

        let instruction = match instruction {
            CookingInstruction::Take(ingredient) => spanned!(Instruction::Take(ingredient)),
            CookingInstruction::Put(ingredient, bowl) => {
                spanned!(Instruction::Put(ingredient, bowl))
            }
            CookingInstruction::Fold(ingredient, bowl) => {
                spanned!(Instruction::Fold(ingredient, bowl))
            }
            CookingInstruction::Add(ingredient, bowl) => {
                spanned!(Instruction::Add(ingredient, bowl))
            }
            CookingInstruction::Remove(ingredient, bowl) => {
                spanned!(Instruction::Remove(ingredient, bowl))
            }
            CookingInstruction::Combine(ingredient, bowl) => {
                spanned!(Instruction::Combine(ingredient, bowl))
            }
            CookingInstruction::Divide(ingredient, bowl) => {
                spanned!(Instruction::Divide(ingredient, bowl))
            }
            CookingInstruction::AddDryIngredients(bowl) => {
                spanned!(Instruction::AddDryIngredients(bowl))
            }
            CookingInstruction::Liquefy(instruction) => spanned!(Instruction::Liquefy(instruction)),
            CookingInstruction::LiquefyContents(bowl) => {
                spanned!(Instruction::LiquefyContents(bowl))
            }
            CookingInstruction::Stir(bowl, minutes) => spanned!(Instruction::Stir(bowl, minutes)),
            CookingInstruction::StirIngredient(ingredient, bowl) => {
                spanned!(Instruction::StirIngredient(ingredient, bowl))
            }
            CookingInstruction::Mix(bowl) => spanned!(Instruction::Mix(bowl)),
            CookingInstruction::Clean(bowl) => spanned!(Instruction::Clean(bowl)),
            CookingInstruction::Pour(bowl, baking_dish) => {
                spanned!(Instruction::Pour(bowl, baking_dish))
            }
            CookingInstruction::SetAside => spanned!(Instruction::SetAside),
            CookingInstruction::ServeWith(recipe) => spanned!(Instruction::ServeWith(recipe)),
            CookingInstruction::Refrigerate(hours) => spanned!(Instruction::Refrigerate(hours)),
            CookingInstruction::Serves(serves) => spanned!(Instruction::Serves(serves)),
            CookingInstruction::Verb(verb, ingredient) => {
                loop_stack.push((
                    VerbLoop {
                        verb: verb.clone(),
                        ingredient,
                        instructions: vec![],
                    },
                    span.clone(),
                ));
                continue;
            }
            CookingInstruction::VerbUntil(_, verb) => {
                let Some((current_loop, mut loop_span)) = loop_stack.pop() else {
                    return Err(ParseError::SecondStage(
                        format!("`until` {} with no matching initial {}", verb.0, verb.0),
                        span.clone(),
                    ));
                };

                loop_span.end = span.end;
                Spanned::new(Instruction::VerbLoop(current_loop), loop_span)
            }
        };
        if let Some(active_loop) = loop_stack.last_mut() {
            let (verb_loop, span) = active_loop;
            span.end = instruction.1.end;
            verb_loop.instructions.push(instruction);
        } else {
            instructions.push(instruction);
        }
    }

    Ok(instructions)
}
