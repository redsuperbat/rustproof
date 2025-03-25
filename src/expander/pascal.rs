use crate::lexer::{Token, TokenKind};

use super::split_on_uppercase;

pub fn expand_pascal(token: &Token) -> Vec<Token> {
    let mut start: u32 = 0;
    split_on_uppercase(&token.lexeme)
        .iter()
        .map(|v| {
            let lexeme = v.to_string();
            let lexeme_len = lexeme.len() as u32;
            let token = Token {
                kind: TokenKind::Identifier,
                start: token.start.increment_col(start),
                end: token.end.increment_col(start + lexeme_len),
                lexeme,
            };
            start += lexeme_len;
            token
        })
        .collect()
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
