use crate::{
    parser::{ChefProgram, Instruction, Spanned, VerbLoop},
    SpatulaError,
};

pub fn validate<'a>(program: &ChefProgram<'a>) -> Result<(), SpatulaError> {
    validate_recipe_references(program)?;

    Ok(())
}

fn validate_recipe_references<'a>(program: &ChefProgram<'_>) -> Result<(), SpatulaError> {
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
