use crate::{lexer::Token, peekable_n::BufferedPeekable};

pub trait Expandable {
    fn expand(&self) -> Vec<Token>;
}

pub struct Expander<I: Iterator<Item = char>> {
    text: BufferedPeekable<I>,
}

impl<I: Iterator<Item = char>> Iterator for Expander<I> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_word()
    }
}

impl<I: Iterator<Item = char>> Expander<I> {
    pub fn new(text: I) -> Self {
        return Self {
            text: BufferedPeekable::new(text, 2),
        };
    }

    fn next_word(&mut self) -> Option<String> {
        let (Some(c1), Some(c2)) = (self.text.next(), self.text.peek()) else {
            return None;
        };
        match (c1.is_uppercase(), c2.is_uppercase()) {
            (true, false) => self.parse_pascal(c1),
            (true, true) => self.parse_upper(c1),
            (false, false) => self.parse_lower(c1),
            _ => None,
        }
    }

    fn parse_pascal(&mut self, first: char) -> Option<String> {
        let mut word = String::from(first);
        while let Some(next) = self.text.peek() {
            if next.is_lowercase() {
                word.push(self.text.next().unwrap());
            } else {
                break;
            }
        }
        Some(word)
    }

    fn parse_upper(&mut self, first: char) -> Option<String> {
        let mut word = String::from(first);
        loop {
            let c1 = self.text.peek().map(|c| c.to_owned());
            let c2 = self.text.peek_at(1);
            match (c1, c2) {
                (Some(_), None) => word.push(self.text.next().unwrap()),
                (Some(c1), Some(c2)) => {
                    if c1.is_uppercase() && c2.is_uppercase() {
                        word.push(self.text.next().unwrap());
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Some(word)
    }

    fn parse_lower(&mut self, first: char) -> Option<String> {
        let mut word = String::from(first);
        while let Some(next) = self.text.peek() {
            if next.is_lowercase() {
                word.push(self.text.next().unwrap());
            } else {
                break;
            }
        }
        Some(word)
    }
}

impl Expandable for Token {
    fn expand(&self) -> Vec<Token> {
        let mut start = self.start.col;
        Expander::new(self.lexeme.chars())
            .into_iter()
            .map(|lexeme| {
                let lexeme_len = lexeme.len() as u32;
                let out_token = Token {
                    start: self.start.set_col(start),
                    end: self.end.set_col(start + lexeme_len),
                    lexeme,
                };
                start += lexeme_len;
                out_token
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn expand(str: &str) -> Vec<String> {
        Expander::new(str.chars()).collect()
    }

    #[test]
    fn test_expandable() {
        assert_eq!(expand("HelloWorld"), vec!["Hello", "World"]);
        assert_eq!(expand("ABBRCase"), vec!["ABBR", "Case"]);
        assert_eq!(expand("DataJSON"), vec!["Data", "JSON"]);
        assert_eq!(expand("DataJSONGood"), vec!["Data", "JSON", "Good"]);
    }
}
