use super::cursor::Lexer;
use super::tokens::{Token, TokenKind};
use crate::ast::Span;

pub fn parse_number(lexer: &mut Lexer, ch: char, span: Span) -> Result<Token, String> {
            let mut num_str = String::new();
            let mut is_float = false;

            if ch == '0' && (lexer.peek_next() == Some('x') || lexer.peek_next() == Some('X')) {
                lexer.advance(); // consume '0'
                lexer.advance(); // consume 'x'
                let mut hex_str = String::new();
                while let Some(c) = lexer.peek() {
                    if c.is_ascii_hexdigit() {
                        hex_str.push(c);
                        lexer.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&hex_str, 16)
                    .or_else(|_| u64::from_str_radix(&hex_str, 16).map(|u| u as i64))
                    .map_err(|e| format!("Invalid hex literal 0x{}: {}", hex_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            if ch == '0' && (lexer.peek_next() == Some('b') || lexer.peek_next() == Some('B')) {
                lexer.advance(); // consume '0'
                lexer.advance(); // consume 'b'
                let mut bin_str = String::new();
                while let Some(c) = lexer.peek() {
                    if c == '0' || c == '1' {
                        bin_str.push(c);
                        lexer.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&bin_str, 2)
                    .map_err(|e| format!("Invalid binary literal 0b{}: {}", bin_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            while let Some(c) = lexer.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    lexer.advance();
                } else if c == '.' && lexer.peek_next().map_or(false, |next| next.is_ascii_digit()) {
                    is_float = true;
                    num_str.push(c);
                    lexer.advance();
                } else {
                    break;
                }
            }

            if lexer.peek() == Some('[') {
                lexer.advance(); // consume '['
                let mut unit_str = String::new();
                while let Some(c) = lexer.peek() {
                    if c == ']' {
                        lexer.advance(); // consume ']'
                        break;
                    } else {
                        unit_str.push(c);
                        lexer.advance();
                    }
                }
                let val_f: f64 = num_str.parse().unwrap_or(0.0);
                return Ok(Token {
                    kind: TokenKind::UnitLit(val_f, unit_str),
                    span,
                });
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
