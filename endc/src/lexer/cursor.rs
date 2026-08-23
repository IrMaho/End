use crate::ast::Span;

#[allow(dead_code)]
pub struct Lexer<'a> {
    pub source: &'a str,
    pub chars: Vec<char>,
    pub cursor: usize,
    pub line: usize,
    pub col: usize,
    pub filename: String,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: impl Into<String>, source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            cursor: 0,
            line: 1,
            col: 1,
            filename: filename.into(),
        }
    }

    pub fn peek(&self) -> Option<char> {
        if self.cursor < self.chars.len() {
            Some(self.chars[self.cursor])
        } else {
            None
        }
    }

    pub fn peek_next(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            Some(self.chars[self.cursor + 1])
        } else {
            None
        }
    }

    pub fn peek_offset(&self, offset: usize) -> Option<char> {
        if self.cursor + offset < self.chars.len() {
            Some(self.chars[self.cursor + offset])
        } else {
            None
        }
    }

    pub fn advance(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor];
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    pub fn skip_whitespace_and_comments(&mut self) {
        while let Some(c) = self.peek() {
            if c.is_whitespace() {
                self.advance();
            } else if c == '/' && self.peek_next() == Some('/') {
                // Line comment
                while let Some(c) = self.peek() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
            } else if c == '/' && self.peek_next() == Some('*') {
                // Block comment
                self.advance(); // consume '/'
                self.advance(); // consume '*'
                while let Some(c) = self.peek() {
                    if c == '*' && self.peek_next() == Some('/') {
                        self.advance(); // consume '*'
                        self.advance(); // consume '/'
                        break;
                    }
                    self.advance();
                }
            } else {
                break;
            }
        }
    }
}
