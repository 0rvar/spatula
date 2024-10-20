use std::collections::HashMap;

use chumsky::span::SimpleSpan;

use crate::{
    parser::{ChefProgram, ChefRecipe, Ingredient, IngredientKind, Instruction, Spanned},
    SpatulaError,
};

pub fn run(program: &ChefProgram) -> Result<(), SpatulaError> {
    eval_recipe(&program.main, &program.auxilary)
}

#[derive(Clone)]
struct IngredientAmount {
    amount: usize,
    kind: IngredientKind,
}
impl IngredientAmount {
    pub fn new(amount: usize, kind: IngredientKind) -> IngredientAmount {
        Self { amount, kind }
    }
    pub fn set_amount(&mut self, amount: usize) {
        self.amount = amount;
    }
    pub fn amount(&self) -> usize {
        self.amount
    }
}

struct EvalContext {
    kinds: HashMap<String, IngredientKind>,
    values: HashMap<String, IngredientAmount>,
    bowls: HashMap<usize, Vec<IngredientAmount>>,
}
fn eval_recipe(
    function: &ChefRecipe<'_, Instruction, Ingredient>,
    scope: &HashMap<String, ChefRecipe<'_, Instruction, Ingredient>>,
) -> Result<(), SpatulaError> {
    let mut values = HashMap::new();
    let mut kinds = HashMap::new();
    for Spanned(ingredient, _) in &function.ingredients {
        kinds.insert(ingredient.name.to_string(), ingredient.kind);
        let Some(initial_value) = ingredient.initial_value else {
            // Silently ignore ingredients without initial values
            // This is according to spec. Later, when trying to use the ingredient and it has no valuie,
            // we will raise a runtime error
            continue;
        };
        values.insert(
            ingredient.name.to_string(),
            IngredientAmount::new(initial_value, ingredient.kind),
        );
    }
    let bowls: HashMap<usize, Vec<IngredientAmount>> = HashMap::new();
    let mut context = EvalContext {
        values,
        bowls,
        kinds,
    };

    eval_instructions(&function.instructions, &mut context, scope)?;

    todo!("Do something with Serves or whatever");
}

fn eval_instructions<'a>(
    instructions: &[Spanned<Instruction<'a>>],
    ctx: &mut EvalContext,
    scope: &HashMap<String, ChefRecipe<'_, Instruction, Ingredient>>,
) -> Result<(), SpatulaError> {
    for instruction in instructions {
        eval_instruction(instruction, ctx, scope)?;
    }

    Ok(())
}

fn eval_instruction<'a>(
    instruction: &Spanned<Instruction<'a>>,
    ctx: &mut EvalContext,
    scope: &HashMap<String, ChefRecipe<'_, Instruction, Ingredient>>,
) -> Result<(), SpatulaError> {
    let Spanned(instruction, span) = instruction;
    match instruction {
        Instruction::Take(ingredient_name) => {
            let value = read_input();
            let Some(kind) = ctx.kinds.get(*ingredient_name) else {
                return Err(SpatulaError::new("Ingredient does not exist", *span));
            };

            ctx.values.insert(
                ingredient_name.to_string(),
                IngredientAmount::new(value, *kind),
            );
        }
        Instruction::Put(ingredient_name, bowl) => {
            modify_bowl(ctx, span, ingredient_name, *bowl, |bowl, value| {
                bowl.push(value.clone());
                Ok(())
            })?;
        }
        Instruction::Fold(ingredient_name, bowl) => {
            modify_bowl(ctx, span, ingredient_name, *bowl, |bowl, value| {
                let Some(value_from_bowl) = bowl.pop() else {
                    return Err(SpatulaError::new("Bowl is empty".to_string(), *span));
                };
                value.set_amount(value_from_bowl.amount());
                Ok(())
            });
        }
        Instruction::Add(ingredient_name, bowl) => {
            binary_op(ctx, span, ingredient_name, *bowl, |a, b| a + b)?;
        }
        Instruction::Remove(ingredient_name, bowl) => {
            binary_op(ctx, span, ingredient_name, *bowl, |a, b| a - b)?;
        }
        Instruction::Combine(ingredient_name, bowl) => {
            binary_op(ctx, span, ingredient_name, *bowl, |a, b| a * b)?;
        }
        Instruction::Divide(ingredient_name, bowl) => {
            binary_op(ctx, span, ingredient_name, *bowl, |a, b| a / b)?;
        }
        Instruction::AddDryIngredients(bowl) => {
            let dry_ingredients = ctx
                .values
                .values()
                .filter(|v| v.kind == IngredientKind::Dry)
                .map(|v| v.amount)
                .sum();
            ctx.bowls.insert(
                *bowl,
                vec![IngredientAmount::new(dry_ingredients, IngredientKind::Dry)],
            );
        }
        Instruction::Liquefy(ingredient_name) => {
            let Some(ingredient) = ctx.values.get_mut(*ingredient_name) else {
                return Err(SpatulaError::new(
                    format!("Ingredient `{ingredient_name}` has no value"),
                    *span,
                ));
            };
            ingredient.kind = IngredientKind::Wet;
        }
        Instruction::LiquefyContents(bowl) => {
            let bowl = ctx.bowls.entry(*bowl).or_default();
            for ingredient in bowl {
                ingredient.kind = IngredientKind::Wet;
            }
        }
        Instruction::Stir(bowl, minutes) => {
            let bowl = ctx.bowls.entry(*bowl).or_default();
            // This "rolls" the top number ingredients in the nth mixing bowl,
            // such that the top ingredient goes down that number of ingredients
            // and all ingredients above it rise one place.
            // If there are not that many ingredients in the bowl,
            // the top ingredient goes to tbe bottom of the bowl and
            // all the others rise one place.
            let Some(top) = bowl.pop() else {
                return Err(SpatulaError::new("Bowl is empty".to_string(), *span));
            };
            let len = bowl.len();
            let new_position = len.saturating_sub(*minutes);
            bowl.insert(new_position, top);
        }
        _ => todo!(),
    };
    Ok(())
}

fn binary_op<F>(
    ctx: &mut EvalContext,
    span: &SimpleSpan,
    ingredient_name: &str,
    bowl: usize,
    op: F,
) -> Result<(), SpatulaError>
where
    F: Fn(usize, usize) -> usize,
{
    modify_bowl(ctx, span, ingredient_name, bowl, |bowl, value| {
        let Some(amount) = bowl.last().map(|value| value.amount) else {
            return Err(SpatulaError::new("Bowl is empty".to_string(), *span));
        };
        bowl.push(IngredientAmount::new(
            op(amount, value.amount()),
            value.kind,
        ));
        Ok(())
    })
}

fn modify_bowl<F>(
    ctx: &mut EvalContext,
    span: &SimpleSpan,
    ingredient_name: &str,
    bowl: usize,
    op: F,
) -> Result<(), SpatulaError>
where
    F: Fn(&mut Vec<IngredientAmount>, &mut IngredientAmount) -> Result<(), SpatulaError>,
{
    let Some(ingredient_value) = ctx.values.get_mut(ingredient_name) else {
        return Err(SpatulaError::new(
            format!("Ingredient `{ingredient_name}` has no value"),
            *span,
        ));
    };

    let bowl = ctx.bowls.entry(bowl).or_default();
    op(bowl, ingredient_value)
}

fn read_input() -> usize {
    loop {
        eprint!("> ");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf);
        let num = buf.parse();
        match num {
            Ok(num) => return num,
            Err(e) => {
                println!("Invalid number: {e:?}")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stir_contents() {
        fn apply_stir(ingredients: &[usize], minutes: usize) -> Vec<usize> {
            let mut ctx = EvalContext {
                values: HashMap::new(),
                bowls: HashMap::new(),
                kinds: HashMap::new(),
            };
            let bowl_index = 0;
            let bowl = ctx.bowls.entry(bowl_index).or_default();
            for ingredient in ingredients {
                bowl.push(IngredientAmount::new(*ingredient, IngredientKind::Dry));
            }
            eval_instruction(
                &Spanned(
                    Instruction::Stir(bowl_index, minutes),
                    SimpleSpan::new(0, 0),
                ),
                &mut ctx,
                &HashMap::new(),
            )
            .unwrap();

            ctx.bowls
                .remove(&bowl_index)
                .unwrap()
                .into_iter()
                .map(|v| v.amount)
                .collect()
        }
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 0), vec![1, 2, 3, 4, 5]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 1), vec![1, 2, 3, 5, 4]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 2), vec![1, 2, 5, 3, 4]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 3), vec![1, 5, 2, 3, 4]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 4), vec![5, 1, 2, 3, 4]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 5), vec![5, 1, 2, 3, 4]);
        assert_eq!(apply_stir(&[1, 2, 3, 4, 5], 6), vec![5, 1, 2, 3, 4]);
    }
}
