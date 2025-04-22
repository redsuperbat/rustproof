use super::expand_uppercase;
use crate::lexer::Token;

pub fn expand_pascal(token: &Token) -> Vec<Token> {
    expand_uppercase(token)
}

pub fn is_pascal(string: &str) -> bool {
    let mut char_iter = string.chars().peekable();
    let first = char_iter.peek();
    let Some(first) = first else { return false };
    if first.is_lowercase() {
        return false;
    };
    return char_iter.any(|c| c.is_uppercase()) && char_iter.any(|c| c.is_lowercase());
}

#[cfg(test)]
mod test {
    use crate::expander::pascal::is_pascal;

    #[test]
    fn is_pascal_test() {
        assert!(is_pascal("HelloWorld"));
        assert!(is_pascal("HelloWorldHello"));
        assert!(is_pascal("HelloW"));
    }
}
