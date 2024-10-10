use std::collections::HashMap;

use super::{
    ast::{ChefRecipe, Verb},
    Spanned,
};

#[derive(Debug, Clone)]
pub struct ChefProgram<'a> {
    pub main: ChefRecipe<'a, Instruction<'a>>,
    pub auxilary: HashMap<&'a str, ChefRecipe<'a, Instruction<'a>>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Instruction<'a> {
    /// Take ingredient from refrigerator.
    /// This reads a numeric value from STDIN into the ingredient named, overwriting any previous value.
    Take(&'a str),
    /// Put ingredient into [nth] mixing bowl.
    /// This puts the ingredient into the nth mixing bowl.
    Put(&'a str, usize),
    /// Fold ingredient into [nth] mixing bowl.
    /// This removes the top value from the nth mixing bowl and places it in the ingredient.
    Fold(&'a str, usize),
    /// Add ingredient [to [nth] mixing bowl].
    /// This adds the value of ingredient to the value of the ingredient on top of the nth mixing bowl and stores the result in the nth mixing bowl.
    Add(&'a str, usize),
    /// Remove ingredient [from [nth] mixing bowl].
    /// This subtracts the value of ingredient from the value of the ingredient on top of the nth mixing bowl and stores the result in the nth mixing bowl.
    Remove(&'a str, usize),
    /// Combine ingredient [into [nth] mixing bowl].
    /// This multiplies the value of ingredient by the value of the ingredient on top of the nth mixing bowl and stores the result in the nth mixing bowl.
    Combine(&'a str, usize),
    /// Divide ingredient [into [nth] mixing bowl].
    /// This divides the value of ingredient into the value of the ingredient on top of the nth mixing bowl and stores the result in the nth mixing bowl.
    Divide(&'a str, usize),
    /// Add dry ingredients [to [nth] mixing bowl].
    /// This adds the values of all the dry ingredients together and places the result into the nth mixing bowl.
    AddDryIngredients(usize),
    /// Liquefy | Liquify ingredient.
    /// This turns the ingredient into a liquid, i.e. a Unicode character for output purposes. (Note: The original specification used the word "Liquify", which is a spelling error. "Liquify" is deprecated. Use "Liquefy" in all new code.)
    Liquefy(&'a str),
    /// Liquefy | Liquify contents of the [nth] mixing bowl.
    /// This turns all the ingredients in the nth mixing bowl into a liquid, i.e. a Unicode characters for output purposes.
    LiquefyContents(usize),
    /// Stir [the [nth] mixing bowl] for number minutes.
    /// This "rolls" the top number ingredients in the nth mixing bowl, such that the top ingredient goes down that number of ingredients and all ingredients above it rise one place. If there are not that many ingredients in the bowl, the top ingredient goes to tbe bottom of the bowl and all the others rise one place.
    Stir(usize, usize),
    /// Stir ingredient into the [nth] mixing bowl.
    /// This rolls the number of ingredients in the nth mixing bowl equal to the value of ingredient, such that the top ingredient goes down that number of ingredients and all ingredients above it rise one place. If there are not that many ingredients in the bowl, the top ingredient goes to the bottom of the bowl and all the others rise one place.
    StirIngredient(&'a str, usize),
    /// Mix [the [nth] mixing bowl] well.
    //// This randomises the order of the ingredients in the nth mixing bowl.
    Mix(usize),
    /// Clean [nth] mixing bowl.
    /// This removes all the ingredients from the nth mixing bowl.
    Clean(usize),
    /// Pour contents of the [nth] mixing bowl into the [pth] baking dish.
    /// This copies all the ingredients from the nth mixing bowl to the pth baking dish, retaining the order and putting them on top of anything already in the baking dish.
    Pour(usize, usize),
    /// Loop.
    VerbLoop(VerbLoop<'a>),
    /// Set aside.
    /// This causes execution of the innermost loop in which it occurs to end immediately and execution to continue at the statement after the "until".
    SetAside,
    /// Serve with auxiliary-recipe.
    /// This invokes a sous-chef to immediately prepare the named auxiliary-recipe. The calling chef waits until the sous-chef is finished before continuing. See the section on auxiliary recipes below.
    ServeWith(&'a str),
    /// Refrigerate [for number hours].
    /// This causes execution of the recipe in which it appears to end immediately. If in an auxiliary recipe, the auxiliary recipe ends and the sous-chef's first mixing bowl is passed back to the calling chef as normal. If a number of hours is specified, the recipe will print out its first number baking dishes (see the Serves statement below) before ending.
    Refrigerate(Option<usize>),
    /// Serves number-of-diners.
    /// This statement writes to STDOUT the contents of the first number-of-diners baking dishes. It begins with the 1st baking dish, removing values from the top one by one and printing them until the dish is empty, then progresses to the next dish, until all the dishes have been printed. The serves statement is optional, but is required if the recipe is to output anything!
    Serves(usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VerbLoop<'a> {
    pub verb: Verb<'a>,
    pub ingredient: &'a str,
    pub instructions: Vec<Spanned<Instruction<'a>>>,
}
