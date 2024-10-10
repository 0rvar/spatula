pub mod ast;
pub mod errors;
mod stage_one;
mod stage_one_ast;
mod stage_two;
mod stage_two_ast;

pub use ast::*;
pub use errors::ParseError;
pub use stage_two_ast::{ChefProgram, Ingredient, Instruction, VerbLoop};

pub fn parse<'a>(input: &'a str) -> Result<ChefProgram<'a>, ParseError> {
    let initial_ast = stage_one::parse(input)?;
    stage_two::parse(initial_ast)
}
