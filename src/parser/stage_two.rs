use std::collections::HashMap;

use chumsky::span::SimpleSpan;

use super::{
    ast::ChefRecipe,
    errors::ParseError,
    stage_one_ast::CookingInstruction,
    stage_two_ast::{ChefProgram, Instruction, VerbLoop},
    Spanned,
};

pub fn parse<'a>(
    input: Vec<ChefRecipe<'a, CookingInstruction<'a>>>,
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
        .map(|recipe| recipe.map(|recipe| (recipe.title, recipe)))
        .collect::<Result<HashMap<&'a str, ChefRecipe<'a, Instruction<'a>>>, ParseError>>()?;
    Ok(ChefProgram { main, auxilary })
}

fn parse_recipe<'a>(
    recipe: ChefRecipe<'a, CookingInstruction<'a>>,
) -> Result<ChefRecipe<'a, Instruction<'a>>, ParseError> {
    let ChefRecipe {
        title,
        comments,
        ingredients,
        cooking_time,
        oven_temperature,
        instructions,
        serves,
    } = recipe;

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
