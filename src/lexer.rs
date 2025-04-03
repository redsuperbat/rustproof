use ropey::Rope;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pos {
    pub line: u32,
    pub col: u32,
}

impl Pos {
    pub fn start() -> Pos {
        Pos { line: 0, col: 0 }
    }

    pub fn set_col(&self, col: u32) -> Pos {
        Pos {
            line: self.line,
            col,
        }
    }
}

impl Into<Pos> for &Pos {
    fn into(self) -> Pos {
        *self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub start: Pos,
    pub end: Pos,
}

impl Into<Location> for &Location {
    fn into(self) -> Location {
        *self
    }
}

impl Location {
    pub fn new<T: Into<Pos>>(start: T, end: T) -> Location {
        Location {
            start: start.into(),
            end: end.into(),
        }
    }

    pub fn beginning() -> Location {
        Location {
            start: Pos::start(),
            end: Pos::start(),
        }
    }
}

#[derive(Debug)]
pub struct Lexer<'a> {
    text: &'a Rope,
    col: u32,
    line: u32,
    offset: usize,
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub start: Pos,
    pub end: Pos,
    pub lexeme: String,
}

impl Into<Location> for &Token {
    fn into(self) -> Location {
        Location::new(self.start, self.end)
    }
}

impl Into<Location> for Token {
    fn into(self) -> Location {
        (&self).into()
    }
}

impl Lexer<'_> {
    pub fn new<'a>(text: &'a Rope) -> Lexer<'a> {
        return Lexer {
            text,
            col: 0,
            line: 0,
            offset: 0,
        };
    }

    fn pos(&self) -> Pos {
        Pos {
            line: self.line,
            col: self.col,
        }
    }

    fn is_accepted_char(&self, char: char) -> bool {
        match char {
            'a'..='z'
            | 'A'..='Z'
            | 'å'
            | 'Å'
            | 'ä'
            | 'Ä'
            | 'ö'
            | 'Ö'
            | 'ø'
            | 'í'
            | 'ü'
            | 'ą'
            | 'ô'
            | 'č'
            | 'ę'
            | 'ė'
            | 'į'
            | 'š'
            | 'ų'
            | 'ž' => true,
            _ => false,
        }
    }

    fn next_token(&mut self) -> Option<Token> {
        let next_char = self.peek()?;

        match next_char {
            c if self.is_accepted_char(c) => self.identifier(),
            _ => {
                self.next()?;
                return self.next_token();
            }
        }
    }

    fn next(&mut self) -> Option<char> {
        let char = self.peek()?;

        self.col += 1;

        if char == '\n' {
            self.col = 0;
            self.line += 1;
        }
        self.offset += 1;

        Some(char)
    }

    fn peek(&self) -> Option<char> {
        self.text.get_char(self.offset)
    }

    fn identifier(&mut self) -> Option<Token> {
        let start = self.pos();
        let mut lexeme = String::new();
        let mut maybe_quote: Option<char> = None;

        loop {
            let Some(char) = self.peek() else {
                break;
            };

            match char {
                c if self.is_accepted_char(c) => {
                    // we only care about single quotes if they occur
                    // in the middle of a word
                    if let Some(quote) = maybe_quote {
                        lexeme += &quote.to_string();
                        maybe_quote = None
                    }
                    lexeme += &self.next()?.to_string();
                }
                '\'' => maybe_quote = self.next(),
                _ => break,
            }
        }

        let end = self.pos();

        Some(Token { end, start, lexeme })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn lex_source() {
        let str = "
fn print(s: string, i: int) {}

fn fizz_buzz(n: int) {
  fn helper(i: int) {
    if i < n + 1 {
      if 0 == d {
        print(\"FizzBuzz\", i)
      } else if 0 == i % 3 {
        print(\"Fizz\", i)
      } else if 0 == i % 5 {
        print(\"Buzz\", i)
      }
      helper(i + 1)
    }
  }
  helper(1)
}

fizz_buzz(15)
";
        let text = Rope::from_str(str);
        let tokens = Lexer::new(&text)
            .map(|v| v.lexeme)
            .collect::<Vec<_>>()
            .join(" ");
        println!("{:?}", tokens)
    }
}
