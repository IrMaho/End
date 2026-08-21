use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Struct,   // 'st' or 'struct'
    Enum,     // 'enum'
    Fn,       // 'fn'
    Val,      // 'val' (immutable)
    Mut,      // 'mut' (mutable)
    Return,   // 'ret' or 'return'
    If,       // 'if'
    Else,     // 'else'
    While,    // 'while'
    For,      // 'for'
    Parallel, // 'parallel'
    In,       // 'in'
    Match,    // 'match'
    Defer,    // 'defer'
    Region,   // 'region'
    Asm,      // 'asm'
    Target,   // 'target'
    Import,   // 'import'
    As,       // 'as'
    Pub,      // 'pub'
    Alloc,    // 'alloc'
    Catch,    // 'catch'
    Null,     // 'null'
    True,     // 'true'
    False,    // 'false'
    Spawn,    // 'spawn'
    Skip,     // 'skip'

    // Directives
    Directive(String), // '@agent_note', '@target', '@c', etc.

    // Literals & Identifiers
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),

    // Symbols & Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Shl,       // '<<'
    Greater,
    GreaterEqual,
    Shr,       // '>>'
    Arrow,     // '->'
    FatArrow,  // '=>'
    Colon,     // ':'
    SemiColon, // ';'
    Comma,     // ','
    Dot,       // '.'
    Ampersand, // '&'
    AmpAmp,    // '&&'
    Pipe,      // '|'
    PipePipe,  // '||'
    Caret,     // '^'
    Tilde,     // '~'
    Underscore,// '_'
    LParen,    // '('
    RParen,    // ')'
    LBrace,    // '{'
    RBrace,    // '}'
    LBracket,  // '['
    RBracket,  // ']'

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

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

    fn peek(&self) -> Option<char> {
        if self.cursor < self.chars.len() {
            Some(self.chars[self.cursor])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            Some(self.chars[self.cursor + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
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

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' => {
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        self.advance();
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == '*' && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();

        let start_line = self.line;
        let start_col = self.col;
        let span = Span::new(&self.filename, start_line, start_col);

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::EOF,
                    span,
                })
            }
        };

        // Identifiers or keywords or directive
        if ch == '@' {
            self.advance();
            let mut name = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(Token {
                kind: TokenKind::Directive(format!("@{}", name)),
                span,
            });
        }

        if ch == '_' && !self.peek_next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Underscore,
                span,
            });
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = match ident.as_str() {
                "st" | "struct" => TokenKind::Struct,
                "enum" => TokenKind::Enum,
                "fn" => TokenKind::Fn,
                "val" => TokenKind::Val,
                "mut" | "var" => TokenKind::Mut,
                "ret" | "return" => TokenKind::Return,
                "if" => TokenKind::If,
                "else" => TokenKind::Else,
                "while" => TokenKind::While,
                "for" => TokenKind::For,
                "parallel" => TokenKind::Parallel,
                "in" => TokenKind::In,
                "match" => TokenKind::Match,
                "defer" => TokenKind::Defer,
                "region" => TokenKind::Region,
                "asm" => TokenKind::Asm,
                "target" => TokenKind::Target,
                "import" => TokenKind::Import,
                "as" => TokenKind::As,
                "pub" => TokenKind::Pub,
                "alloc" => TokenKind::Alloc,
                "catch" => TokenKind::Catch,
                "null" => TokenKind::Null,
                "true" => TokenKind::True,
                "false" => TokenKind::False,
                "spawn" => TokenKind::Spawn,
                "skip" => TokenKind::Skip,
                _ => TokenKind::Ident(ident),
            };

            return Ok(Token { kind, span });
        }

        // Numbers (integers, hex, binary, or floats)
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            let mut is_float = false;

            if ch == '0' && (self.peek_next() == Some('x') || self.peek_next() == Some('X')) {
                self.advance(); // consume '0'
                self.advance(); // consume 'x'
                let mut hex_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_hexdigit() {
                        hex_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&hex_str, 16)
                    .or_else(|_| u64::from_str_radix(&hex_str, 16).map(|u| u as i64))
                    .map_err(|e| format!("Invalid hex literal 0x{}: {}", hex_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            if ch == '0' && (self.peek_next() == Some('b') || self.peek_next() == Some('B')) {
                self.advance(); // consume '0'
                self.advance(); // consume 'b'
                let mut bin_str = String::new();
                while let Some(c) = self.peek() {
                    if c == '0' || c == '1' {
                        bin_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&bin_str, 2)
                    .map_err(|e| format!("Invalid binary literal 0b{}: {}", bin_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else if c == '.' && self.peek_next().map_or(false, |next| next.is_ascii_digit()) {
                    is_float = true;
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = if is_float {
                let val: f64 = num_str.parse().map_err(|e| format!("Invalid float: {}", e))?;
                TokenKind::FloatLit(val)
            } else {
                let val: i64 = num_str.parse().map_err(|e| format!("Invalid integer: {}", e))?;
                TokenKind::IntLit(val)
            };

            return Ok(Token { kind, span });
        }

        // String literals
        if ch == '"' {
            self.advance(); // consume opening quote
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c == '"' {
                    self.advance(); // consume closing quote
                    return Ok(Token {
                        kind: TokenKind::StringLit(s),
                        span,
                    });
                } else if c == '\\' {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(other) => s.push(other),
                        None => return Err("Unexpected EOF in escape string sequence".into()),
                    }
                } else {
                    s.push(c);
                    self.advance();
                }
            }
            return Err("Unterminated string literal".into());
        }

        // Operators & Single character tokens
        self.advance();
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else if self.peek() == Some('<') {
                    self.advance();
                    TokenKind::Shl
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Shr
                } else {
                    TokenKind::Greater
                }
            }
            ':' => TokenKind::Colon,
            ';' => TokenKind::SemiColon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::PipePipe
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => TokenKind::Caret,
            '~' => TokenKind::Tilde,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            other => return Err(format!("Unexpected character: '{}' at line {}, col {}", other, start_line, start_col)),
        };

        Ok(Token { kind, span })
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::EOF;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
