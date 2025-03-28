use crate::lexer::{Token, TokenKind};
use camel::{expand_camel, is_camel};
use pascal::{expand_pascal, is_pascal};

pub mod camel;
pub mod pascal;

pub trait Expandable {
    fn expand(&self) -> Option<Vec<Token>>;
}

impl Expandable for Token {
    fn expand(&self) -> Option<Vec<Token>> {
        if is_camel(&self.lexeme) {
            return Some(expand_camel(&self));
        }
        if is_pascal(&self.lexeme) {
            return Some(expand_pascal(&self));
        }
        None
    }
}

fn split_on_uppercase(s: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current_word = String::new();

    for c in s.chars() {
        if c.is_uppercase() && !current_word.is_empty() {
            words.push(current_word);
            current_word = String::new();
        }
        current_word.push(c);
    }

    if !current_word.is_empty() {
        words.push(current_word);
    }

    words
}

fn expand_uppercase(token: &Token) -> Vec<Token> {
    let mut start = token.start.col;
    split_on_uppercase(&token.lexeme)
        .iter()
        .map(|v| {
            let lexeme = v.to_string();
            let lexeme_len = lexeme.len() as u32;
            let token = Token {
                kind: TokenKind::Identifier,
                start: token.start.set_col(start),
                end: token.end.set_col(start + lexeme_len),
                lexeme,
            };
            start += lexeme_len;
            token
        })
        .collect()
}
