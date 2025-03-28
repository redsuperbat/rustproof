use crate::expander::Expandable;
use crate::lexer::{Lexer, Token};

pub struct Pipeline {}

impl Pipeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self, lexer: Lexer) -> Vec<Token> {
        lexer
            .into_iter()
            // We ignore tokens with a lexeme shorter than 4 characters
            // Those are not relevant for spelling mistakes
            .filter(|v| v.lexeme.len() > 3)
            .flat_map(|v| {
                if let Some(t) = v.expand() {
                    return t;
                }
                return vec![v];
            })
            // After expansion the tokens could be broken into smaller ones
            // therefore we filter again the first is just a performance optimization
            .filter(|v| v.lexeme.len() > 3)
            .collect()
    }
}
