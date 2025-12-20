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
pub struct Lexer<I: Iterator<Item = char>> {
    text: I,
    col: u32,
    line: u32,
    offset: usize,
}

impl<I: Iterator<Item = char>> Iterator for Lexer<I> {
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

impl<I: Iterator<Item = char>> Lexer<I> {
    pub fn new(text: I) -> Self {
        Self {
            text,
            col: 0,
            line: 0,
            offset: 0,
        }
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
        let start = self.pos();
        let mut lexeme = String::new();
        let mut maybe_quote: Option<char> = None;

        let mut end;
        loop {
            end = self.pos();

            let Some(char) = self.next() else {
                // We are at the end of the file
                if lexeme.is_empty() {
                    return None;
                }
                return Some(Token { lexeme, start, end });
            };

            match char {
                c if self.is_accepted_char(c) => {
                    // we only care about single quotes if they occur
                    // in the middle of a word
                    if let Some(quote) = maybe_quote {
                        lexeme += &quote.to_string();
                        maybe_quote = None
                    }
                    lexeme += &char.to_string();
                }
                '\'' => {
                    if lexeme.is_empty() {
                        break;
                    }
                    maybe_quote = Some(char);
                }
                _ => break,
            }
        }

        if lexeme.is_empty() {
            return self.next_token();
        }

        Some(Token { end, start, lexeme })
    }

    fn next(&mut self) -> Option<char> {
        let char = self.text.next()?;

        if char == '\n' {
            self.col = 0;
            self.line += 1;
        } else {
            // Use UTF-16 code units for LSP compatibility
            // BMP characters (U+0000 to U+FFFF) = 1 code unit
            // Characters outside BMP (emoji, etc.) = 2 code units (surrogate pair)
            self.col += char.len_utf16() as u32;
        }

        self.offset += 1;

        Some(char)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(str: &str) -> String {
        Lexer::new(str.chars())
            .map(|v| v.lexeme)
            .collect::<Vec<_>>()
            .join(" ")
    }

    #[test]
    fn it_creates_tokens_from_snake_case() {
        let str = "fn fizz_buzz(n: int){}";
        let tokens = tokenize(str);
        assert_eq!(tokens, "fn fizz buzz n int");
    }

    #[test]
    fn it_creates_tokens_from_kebab_case() {
        let str = "fn-fizz-buzz(n: int){}";
        let tokens = tokenize(str);
        assert_eq!(tokens, "fn fizz buzz n int");
    }

    #[test]
    fn it_handles_unicode_characters() {
        let str = "🤖 Hellooo";
        let tokens = Lexer::new(str.chars()).collect::<Vec<_>>();
        let token = tokens.get(0).unwrap();
        assert_eq!(token.start.col, 3);
    }

    #[test]
    fn it_creates_tokens_with_single_quotes() {
        let str = "fn fizz_buzz(n: string){ return \"what's up dude\"}";
        let tokens = tokenize(str);
        assert_eq!(tokens, "fn fizz buzz n string return what's up dude");

        let str = "fn fizz_buzz(n: string){ return 'hello #{n} world'}";
        let tokens = tokenize(str);
        assert_eq!(tokens, "fn fizz buzz n string return hello n world");

        let str = "fn fizz_buzz(n: string){ return 'hello #{n}'}";
        let tokens = tokenize(str);
        assert_eq!(tokens, "fn fizz buzz n string return hello n");
    }

    #[test]
    fn it_handles_tabs_as_single_utf16_unit() {
        // Tab is 1 UTF-16 code unit, not visual width
        let str = "\tHellooo";
        let tokens = Lexer::new(str.chars()).collect::<Vec<_>>();
        let token = tokens.get(0).unwrap();
        assert_eq!(token.start.col, 1); // 1 tab = 1 UTF-16 code unit
    }

    #[test]
    fn it_handles_surrogate_pairs() {
        // Characters outside BMP use 2 UTF-16 code units (surrogate pair)
        // 𐐀 (U+10400) is outside BMP
        let str = "a𐐀b";
        let tokens = Lexer::new(str.chars()).collect::<Vec<_>>();
        // 'a' at col 0, '𐐀' at col 1 (2 code units), 'b' at col 3
        let token = tokens.get(0).unwrap(); // "a"
        assert_eq!(token.start.col, 0);
        let token = tokens.get(1).unwrap(); // "b"
        assert_eq!(token.start.col, 3); // 1 + 2 = 3
    }
}
