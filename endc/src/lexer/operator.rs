use super::cursor::Lexer;
use super::tokens::{Token, TokenKind};
use crate::ast::Span;

pub fn parse_operator(lexer: &mut Lexer, ch: char, span: Span, start_line: usize, start_col: usize) -> Result<Token, String> {
        lexer.advance();
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if lexer.peek() == Some('>') {
                    lexer.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if lexer.peek() == Some('=') {
                    lexer.advance();
                    TokenKind::EqualEqual
                } else if lexer.peek() == Some('>') {
                    lexer.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if lexer.peek() == Some('=') {
                    lexer.advance();
                    TokenKind::BangEqual
                } else if lexer.peek() == Some('-') && lexer.peek_next() == Some('>') {
                    lexer.advance(); // consume '-'
                    lexer.advance(); // consume '>'
                    TokenKind::BangArrow
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if lexer.peek() == Some('+') && lexer.peek_next() == Some('=') {
                    lexer.advance(); // consume '+'
                    lexer.advance(); // consume '='
                    TokenKind::LessPlusEqual
                } else if lexer.peek() == Some('=') {
                    lexer.advance();
                    TokenKind::LessEqual
                } else if lexer.peek() == Some('<') {
                    lexer.advance();
                    TokenKind::Shl
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if lexer.peek() == Some('=') {
                    lexer.advance();
                    TokenKind::GreaterEqual
                } else if lexer.peek() == Some('>') {
                    lexer.advance();
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
                if lexer.peek() == Some('&') {
                    lexer.advance();
                    TokenKind::AmpAmp
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if lexer.peek() == Some('|') {
                    lexer.advance();
                    TokenKind::PipePipe
                } else if lexer.peek() == Some('>') {
                    lexer.advance();
                    TokenKind::PipeGreater
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => TokenKind::Caret,
            '~' => {
                if lexer.peek() == Some('>') {
                    lexer.advance();
                    TokenKind::TildeArrow
                } else {
                    TokenKind::Tilde
                }
            }
            '?' => {
                if lexer.peek() == Some('?') {
                    lexer.advance();
                    TokenKind::QuestionQuestion
                } else {
                    TokenKind::Question
                }
            }
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

