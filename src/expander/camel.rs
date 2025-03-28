use super::expand_uppercase;
use crate::lexer::Token;

pub fn expand_camel(token: &Token) -> Vec<Token> {
    expand_uppercase(token)
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

#[cfg(test)]
mod test {
    use crate::lexer::{Pos, Token, TokenKind};

    use super::expand_camel;

    #[test]
    fn camel_test() {
        let lexeme = "HelloWorld".to_string();
        let token = Token {
            kind: TokenKind::Identifier,
            start: Pos::start(),
            end: Pos {
                line: 0,
                col: (lexeme.len() as u32) - 1,
            },
            lexeme,
        };
        let result = expand_camel(&token);
        println!("{:#?}", result);
    }
}
