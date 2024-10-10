use std::collections::HashSet;

use crate::{
    parser::{ChefProgram, ChefRecipe, Ingredient, Instruction, Spanned, VerbLoop},
    SpatulaError,
};

pub fn validate<'a>(program: &ChefProgram<'a>) -> Result<(), SpatulaError> {
    validate_recipe_references(program)?;
    for recipe in std::iter::once(&program.main).chain(program.auxilary.values()) {
        validate_ingredient_references(recipe)?;
    }

    Ok(())
}

fn validate_ingredient_references(
    recipe: &ChefRecipe<'_, Instruction<'_>, Ingredient<'_>>,
) -> Result<(), SpatulaError> {
    let ingredients_by_name = recipe
        .ingredients
        .iter()
        .map(|Spanned(ingredient, _)| ingredient.name.to_lowercase())
        .collect::<HashSet<_>>();
    let errors = visit(
        recipe.instructions.iter(),
        vec![],
        &|Spanned(instruction, span), errors| match instruction {
            Instruction::Take(i)
            | Instruction::Put(i, _)
            | Instruction::Fold(i, _)
            | Instruction::Add(i, _)
            | Instruction::Remove(i, _)
            | Instruction::Combine(i, _)
            | Instruction::Divide(i, _)
            | Instruction::Liquefy(i)
            | Instruction::StirIngredient(i, _) => {
                if !ingredients_by_name.contains(i.to_lowercase().as_str()) {
                    errors.push(SpatulaError {
                        message: format!("Ingredient `{}` not found", i),
                        span: span.clone(),
                    });
                }
            }
            _ => {}
        },
    );

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors.into_iter().next().unwrap())
    }
}

fn validate_recipe_references(program: &ChefProgram<'_>) -> Result<(), SpatulaError> {
    let all_instructions = program.main.instructions.iter().chain(
        program
            .auxilary
            .values()
            .flat_map(|r| r.instructions.iter()),
    );

    let references = visit(
        all_instructions,
        vec![],
        &|Spanned(instr, span), references| {
            if let Instruction::ServeWith(recipe) = instr {
                references.push((recipe.to_string(), span.clone()));
            }
        },
    );

    let available_recipes = program.auxilary.keys();
    for (reference, span) in references {
        if !program
            .auxilary
            .contains_key(reference.to_lowercase().as_str())
        {
            return Err(SpatulaError {
                message: format!(
                    "Recipe `{reference}` not found. Available recipes: {available_recipes:?}"
                ),
                span,
            });
        }
    }

    Ok(())
}

fn visit<'a, I, A, F>(instructions: I, mut arg: A, visitor: &F) -> A
where
    I: Iterator<Item = &'a Spanned<Instruction<'a>>>,
    F: Fn(&Spanned<Instruction>, &mut A),
{
    for instruction in instructions {
        visitor(instruction, &mut arg);
        if let Instruction::VerbLoop(VerbLoop { instructions, .. }) = &instruction.0 {
            arg = visit(instructions.iter(), arg, visitor);
        }
    }
    arg
}
