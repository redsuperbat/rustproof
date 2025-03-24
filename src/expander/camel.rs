use super::split_on_uppercase;
use crate::lexer::{Token, TokenKind};

pub fn expand_camel(token: &Token) -> Vec<Token> {
    split_on_uppercase(&token.lexeme)
        .iter()
        .map(|v| Token {
            kind: TokenKind::Identifier,
            start: token.start,
            end: token.end,
            lexeme: v.to_string(),
        })
        .collect()
}

pub fn is_camel(string: &str) -> bool {
    let mut char_iter = string.chars();
    let first = char_iter.next();
    let Some(first) = first else { return false };
    if first.is_uppercase() {
        return false;
    };
    return char_iter.any(|c| c.is_uppercase());
}
