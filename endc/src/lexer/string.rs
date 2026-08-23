use super::cursor::Lexer;
use super::tokens::{Token, TokenKind};
use crate::ast::Span;

pub fn parse_string(lexer: &mut Lexer, span: Span) -> Result<Token, String> {
    lexer.advance(); // consume opening quote
    let is_triple = if lexer.peek() == Some('"') && lexer.peek_next() == Some('"') {
        lexer.advance(); // consume 2nd quote
        lexer.advance(); // consume 3rd quote
        true
    } else {
        false
    };

    let mut s = String::new();
    while let Some(c) = lexer.peek() {
        if is_triple {
            if c == '"' && lexer.peek_next() == Some('"') && lexer.peek_offset(2) == Some('"') {
                lexer.advance();
                lexer.advance();
                lexer.advance();
                return Ok(Token {
                    kind: TokenKind::StringLit(s),
                    span,
                });
            }
        } else if c == '"' {
            lexer.advance(); // consume closing quote
            return Ok(Token {
                kind: TokenKind::StringLit(s),
                span,
            });
        }

        if c == '\\' && !is_triple {
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
