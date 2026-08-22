use super::cursor::Lexer;
use super::tokens::{Token, TokenKind};
use crate::ast::Span;

pub fn parse_string(lexer: &mut Lexer, span: Span) -> Result<Token, String> {
            lexer.advance(); // consume opening quote
            let mut s = String::new();
            while let Some(c) = lexer.peek() {
                if c == '"' {
                    lexer.advance(); // consume closing quote
                    return Ok(Token {
                        kind: TokenKind::StringLit(s),
                        span,
                    });
                } else if c == '\\' {
                    lexer.advance();
                    match lexer.advance() {
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
                    lexer.advance();
                }
            }
            return Err("Unterminated string literal".into());

}
