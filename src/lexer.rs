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
pub struct Lexer {
    text: Vec<char>,
    col: u32,
    line: u32,
    offset: usize,
}

impl Iterator for Lexer {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_token()
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
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

#[derive(Debug, PartialEq, Clone)]
pub enum TokenKind {
    Identifier,
}

impl Lexer {
    pub fn new<T: AsRef<str>>(text: T) -> Lexer {
        return Lexer {
            text: text.as_ref().chars().collect(),
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

    fn next_token(&mut self) -> Option<Token> {
        let next_char = self.peek()?;

        match next_char {
            'a'..='z' | 'A'..='Z' => self.identifier(),
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
        self.text.get(self.offset).map(|c| *c)
    }

    fn identifier(&mut self) -> Option<Token> {
        let start = self.pos();
        let mut lexeme = String::new();

        loop {
            let Some(char) = self.peek() else {
                break;
            };

            match char {
                'a'..='z' | 'A'..='Z' => {
                    lexeme += &self.next()?.to_string();
                }
                _ => break,
            }
        }

        let end = self.pos();

        Some(Token {
            end,
            start,
            kind: TokenKind::Identifier,
            lexeme,
        })
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
        printn(\"FizzBuzz\", i)
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
        let tokens = Lexer::new(str)
            .map(|v| v.lexeme)
            .collect::<Vec<_>>()
            .join(" ");
        println!("{:?}", tokens)
    }
}
