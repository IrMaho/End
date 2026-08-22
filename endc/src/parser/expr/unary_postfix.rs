use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_unary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();
        if self.match_token(&TokenKind::Minus) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Negate,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Bang) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Not,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Tilde) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::BitNot,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Ampersand) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::AddressOf,
                expr: Box::new(expr),
                span,
            });
        }
        if self.match_token(&TokenKind::Star) {
            let expr = self.parse_unary()?;
            return Ok(Expression::Unary {
                op: UnaryOp::Deref,
                expr: Box::new(expr),
                span,
            });
        }

        let mut expr = self.parse_postfix()?;
        while self.match_token(&TokenKind::As) {
            let span = self.current_span();
            let target_type = self.parse_type()?;
            expr = Expression::Cast {
                expr: Box::new(expr),
                target_type,
                span,
            };
        }
        Ok(expr)
    }


    pub(crate) fn parse_postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            let span = self.current_span();
            if self.match_token(&TokenKind::Dot) {
                let member_name = self.parse_identifier_or_keyword()?;
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field: member_name,
                    span,
                };
            } else if self.match_token(&TokenKind::QuestionDot) {
                let member_name = self.parse_identifier_or_keyword()?;
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field: member_name,
                    span,
                };
            } else if self.check(&TokenKind::QuestionDotDot) || (self.check(&TokenKind::DotDot) && self.peek_next_kind().map_or(false, |k| matches!(k, TokenKind::Ident(_)))) {
                expr = self.parse_cascade_chain(expr)?;
            } else if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let arg_span = self.current_span();
                    let checkpoint = self.cursor.clone();
                    if let Ok(arg_name) = self.parse_identifier_or_keyword() {
                        if self.match_token(&TokenKind::Colon) {
                            let arg_val = self.parse_expression()?;
                            args.push(Expression::NamedArg {
                                name: arg_name,
                                value: Box::new(arg_val),
                                span: arg_span,
                            });
                            self.match_token(&TokenKind::Comma);
                            continue;
                        }
                    }
                    self.cursor = checkpoint;
                    args.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                if self.check(&TokenKind::LBrace) {
                    let closure = self.parse_closure_or_block()?;
                    args.push(closure);
                }
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else if self.check(&TokenKind::LBracket) && !self.peek_next_kind().map_or(false, |k| matches!(k, TokenKind::For | TokenKind::If)) {
                let checkpoint = self.cursor;
                self.advance();
                let index_res = self.parse_expression();
                if let Ok(index) = index_res {
                    if self.match_token(&TokenKind::RBracket) {
                        expr = Expression::Index {
                            array: Box::new(expr),
                            index: Box::new(index),
                            span,
                        };
                        continue;
                    }
                }
                self.cursor = checkpoint;
                break;
            } else if self.match_token(&TokenKind::Retry) {
                let count = self.parse_primary()?;
                expr = Expression::Repeat {
                    op: Box::new(expr),
                    count: Box::new(count),
                    is_retry: true,
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

}
