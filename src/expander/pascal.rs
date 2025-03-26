use super::expand_uppercase;
use crate::lexer::Token;

pub fn expand_pascal(token: &Token) -> Vec<Token> {
    expand_uppercase(token)
}

pub fn is_pascal(string: &str) -> bool {
    let mut char_iter = string.chars();
    let first = char_iter.next();
    let Some(first) = first else { return false };
    if first.is_lowercase() {
        return false;
    };
    return char_iter.any(|c| c.is_uppercase()) && char_iter.any(|c| c.is_lowercase());
}
