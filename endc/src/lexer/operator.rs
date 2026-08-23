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
            '*' => {
                if lexer.peek() == Some('*') {
                    lexer.advance();
                    TokenKind::StarStar
                } else {
                    TokenKind::Star
                }
            }
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
                if lexer.peek() == Some('-') && lexer.peek_next() == Some('>') {
                    lexer.advance(); // consume '-'
                    lexer.advance(); // consume '>'
                    TokenKind::BiArrow
                } else if lexer.peek() == Some('~') && lexer.peek_next() == Some('>') {
                    lexer.advance(); // consume '~'
                    lexer.advance(); // consume '>'
                    TokenKind::TildeBiArrow
                } else if lexer.peek() == Some('+') && lexer.peek_next() == Some('=') {
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
            ':' => {
                if lexer.peek() == Some('=') {
                    lexer.advance();
                    TokenKind::ColonEqual
                } else {
                    TokenKind::Colon
                }
            }
            ';' => TokenKind::SemiColon,
            ',' => TokenKind::Comma,
            '.' => {
                if lexer.peek() == Some('.') {
                    if lexer.peek_next() == Some('.') {
                        lexer.advance(); // consume second '.'
                        lexer.advance(); // consume third '.'
                        if lexer.peek() == Some('?') {
                            lexer.advance();
                            TokenKind::DotDotDotQuestion
                        } else {
                            TokenKind::DotDotDot
                        }
                    } else if lexer.peek_next() == Some('<') {
                        lexer.advance(); // consume second '.'
                        lexer.advance(); // consume '<'
                        TokenKind::DotDotLess
                    } else {
                        lexer.advance(); // consume second '.'
                        TokenKind::DotDot
                    }
                } else {
                    TokenKind::Dot
                }
            }
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
                    if lexer.peek_next() == Some('=') {
                        lexer.advance(); // consume second '?'
                        lexer.advance(); // consume '='
                        TokenKind::QuestionQuestionEqual
                    } else {
                        lexer.advance();
                        TokenKind::QuestionQuestion
                    }
                } else if lexer.peek() == Some('.') {
                    if lexer.peek_next() == Some('.') {
                        lexer.advance(); // consume '.'
                        lexer.advance(); // consume second '.'
                        TokenKind::QuestionDotDot
                    } else {
                        lexer.advance(); // consume '.'
                        TokenKind::QuestionDot
                    }
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

