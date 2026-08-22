use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

pub mod primary;
pub mod unary_postfix;

impl Parser {
    pub(crate) fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_pipe_expr()
    }


    pub(crate) fn parse_pipe_expr(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_catch_expr()?;
        while self.check(&TokenKind::PipeGreater)
            || self.check(&TokenKind::TildeArrow)
            || self.check(&TokenKind::Shr)
            || self.check(&TokenKind::Fallback)
            || self.check(&TokenKind::When)
            || (self.check(&TokenKind::Question) && !self.check(&TokenKind::QuestionQuestion))
        {
            if self.match_token(&TokenKind::PipeGreater) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Pipe {
                    lhs: Box::new(expr),
                    rhs: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::TildeArrow) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::NullCollapse {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::Shr) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Compose {
                    ops: vec![expr, rhs],
                    span,
                };
            } else if self.match_token(&TokenKind::Fallback) {
                let span = self.current_span();
                let rhs = self.parse_catch_expr()?;
                expr = Expression::Alternative {
                    left: Box::new(expr),
                    right: Box::new(rhs),
                    span,
                };
            } else if self.match_token(&TokenKind::When) || self.match_token(&TokenKind::Question) {
                let span = self.current_span();
                let cond = self.parse_catch_expr()?;
                expr = Expression::ConditionalOp {
                    op: Box::new(expr),
                    condition: Box::new(cond),
                    span,
                };
            }
        }
        Ok(expr)
    }


    pub(crate) fn parse_catch_expr(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_logical_or()?;

        if self.match_token(&TokenKind::Catch) {
            let span = self.current_span();
            let mut err_name = "err".to_string();
            if let TokenKind::Ident(n) = self.peek_kind() {
                if n != "return" && n != "ret" {
                    err_name = n.clone();
                    self.advance();
                }
            }

            let handler = if self.check(&TokenKind::Return) {
                let stmt = self.parse_statement()?;
                Box::new(stmt)
            } else if self.check(&TokenKind::LBrace) {
                let blk = self.parse_block()?;
                Box::new(Statement::Expression(Expression::Block(blk)))
            } else {
                let sub_expr = self.parse_expression()?;
                Box::new(Statement::Expression(sub_expr))
            };

            expr = Expression::Catch {
                expr: Box::new(expr),
                error_name: err_name,
                handler,
                span,
            };
        }

        Ok(expr)
    }


    pub(crate) fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_logical_and()?;
        while self.match_token(&TokenKind::PipePipe) {
            let span = self.current_span();
            let right = self.parse_logical_and()?;
            let is_op = match &left {
                Expression::Ident(name, _) => name.chars().next().map_or(false, |c| c.is_uppercase()),
                Expression::Repeat { .. } | Expression::Compose { .. } | Expression::Parallel { .. } | Expression::Alternative { .. } => true,
                _ => false,
            };
            if is_op {
                left = Expression::Parallel {
                    left: Box::new(left),
                    right: Box::new(right),
                    span,
                };
            } else {
                left = Expression::Binary {
                    left: Box::new(left),
                    op: BinaryOp::Or,
                    right: Box::new(right),
                    span,
                };
            }
        }
        Ok(left)
    }


    pub(crate) fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_or()?;
        while self.match_token(&TokenKind::AmpAmp) {
            let span = self.current_span();
            let right = self.parse_bitwise_or()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_bitwise_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_xor()?;
        while self.match_token(&TokenKind::Pipe) {
            let span = self.current_span();
            let right = self.parse_bitwise_xor()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitOr,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_bitwise_xor(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_bitwise_and()?;
        while self.match_token(&TokenKind::Caret) {
            let span = self.current_span();
            let right = self.parse_bitwise_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitXor,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_bitwise_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::Ampersand) {
            let span = self.current_span();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::BitAnd,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_equality(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_comparison()?;
        while self.check(&TokenKind::EqualEqual) || self.check(&TokenKind::BangEqual) {
            let op = if self.match_token(&TokenKind::EqualEqual) {
                BinaryOp::Equal
            } else {
                self.advance();
                BinaryOp::NotEqual
            };
            let span = self.current_span();
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_shift()?;
        while self.check(&TokenKind::Less)
            || self.check(&TokenKind::LessEqual)
            || self.check(&TokenKind::Greater)
            || self.check(&TokenKind::GreaterEqual)
        {
            let op = if self.match_token(&TokenKind::Less) {
                BinaryOp::LessThan
            } else if self.match_token(&TokenKind::LessEqual) {
                BinaryOp::LessEqual
            } else if self.match_token(&TokenKind::Greater) {
                BinaryOp::GreaterThan
            } else {
                self.advance();
                BinaryOp::GreaterEqual
            };
            let span = self.current_span();
            let right = self.parse_shift()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_shift(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;
        while self.check(&TokenKind::Shl) || self.check(&TokenKind::Shr) {
            let span = self.current_span();
            if self.match_token(&TokenKind::Shl) {
                let right = self.parse_addition()?;
                left = Expression::Binary {
                    left: Box::new(left),
                    op: BinaryOp::Shl,
                    right: Box::new(right),
                    span,
                };
            } else {
                self.advance();
                let right = self.parse_addition()?;
                match &mut left {
                    Expression::Compose { ops, .. } => {
                        ops.push(right);
                    }
                    _ => {
                        left = Expression::Compose {
                            ops: vec![left, right],
                            span,
                        };
                    }
                }
            }
        }
        Ok(left)
    }


    pub(crate) fn parse_addition(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_multiplication()?;
        while self.check(&TokenKind::Plus) || self.check(&TokenKind::Minus) {
            let op = if self.match_token(&TokenKind::Plus) {
                BinaryOp::Add
            } else {
                self.advance();
                BinaryOp::Sub
            };
            let span = self.current_span();
            let right = self.parse_multiplication()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }


    pub(crate) fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;
        while self.check(&TokenKind::Star) || self.check(&TokenKind::Slash) || self.check(&TokenKind::Percent) {
            let span = self.current_span();
            if self.match_token(&TokenKind::Star) {
                let right = self.parse_unary()?;
                left = Expression::Binary {
                    left: Box::new(left),
                    op: BinaryOp::Mul,
                    right: Box::new(right),
                    span,
                };
            } else if self.match_token(&TokenKind::Slash) {
                let right = self.parse_unary()?;
                let is_op = match &left {
                    Expression::Ident(name, _) => name.chars().next().map_or(false, |c| c.is_uppercase()),
                    Expression::Repeat { .. } | Expression::Compose { .. } | Expression::Parallel { .. } | Expression::Alternative { .. } => true,
                    _ => false,
                };
                if is_op {
                    left = Expression::Alternative {
                        left: Box::new(left),
                        right: Box::new(right),
                        span,
                    };
                } else {
                    left = Expression::Binary {
                        left: Box::new(left),
                        op: BinaryOp::Div,
                        right: Box::new(right),
                        span,
                    };
                }
            } else {
                self.advance();
                let right = self.parse_unary()?;
                left = Expression::Binary {
                    left: Box::new(left),
                    op: BinaryOp::Mod,
                    right: Box::new(right),
                    span,
                };
            }
        }
        Ok(left)
    }

}
