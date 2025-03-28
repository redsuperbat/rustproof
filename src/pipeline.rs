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
            // We ignore tokens with a lexeme of length 3 since it's not
            // relevant for spelling mistakes
            .filter(|v| v.lexeme.len() > 3)
            .flat_map(|v| {
                if let Some(t) = v.expand() {
                    return t;
                }
                return vec![v];
            })
            .collect()
    }
}
