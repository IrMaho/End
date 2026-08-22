use crate::ast::*;
use crate::ast::expr::collections::{CollectionElement, StringPart};
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    /// Parses bracket expressions: list literals, collection control flow, and list comprehensions.
    pub(crate) fn parse_bracket_collection(&mut self) -> Result<Expression, String> {
        let span = self.current_span();
        self.expect(TokenKind::LBracket)?;

        if self.match_token(&TokenKind::RBracket) {
            return Ok(Expression::ListLiteral(vec![], span));
        }

        // Try standard list comprehension: `[expr for var in iterable if condition]`
        let checkpoint = self.cursor.clone();
        let first_expr_res = self.parse_expression();
        if let Ok(first_expr) = first_expr_res {
            if self.match_token(&TokenKind::For) {
                let var = self.parse_identifier_or_keyword()?;
                self.expect(TokenKind::In)?;
                let iterable = self.parse_pipe_expr()?;
                let mut condition = None;
                if self.match_token(&TokenKind::If) {
                    condition = Some(Box::new(self.parse_expression()?));
                }
                self.expect(TokenKind::RBracket)?;
                return Ok(Expression::ListComprehension {
                    expr: Box::new(first_expr),
                    var,
                    iterable: Box::new(iterable),
                    condition,
                    span,
                });
            }
        }
        self.cursor = checkpoint;

        let mut elements = Vec::new();
        while !self.check(&TokenKind::RBracket) && !self.check(&TokenKind::EOF) {
            elements.push(self.parse_collection_element()?);
            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RBracket)?;
        Ok(Expression::ListLiteral(elements, span))
    }

    pub(crate) fn parse_collection_element(&mut self) -> Result<CollectionElement, String> {
        if self.match_token(&TokenKind::DotDotDot) {
            let expr = self.parse_expression()?;
            return Ok(CollectionElement::Spread {
                expr,
                is_null_aware: false,
            });
        }
        if self.match_token(&TokenKind::DotDotDotQuestion) {
            let expr = self.parse_expression()?;
            return Ok(CollectionElement::Spread {
                expr,
                is_null_aware: true,
            });
        }
        if self.match_token(&TokenKind::If) {
            let condition = self.parse_pipe_expr()?;
            let element = self.parse_collection_element()?;
            let mut else_element = None;
            if self.match_token(&TokenKind::Else) {
                else_element = Some(Box::new(self.parse_collection_element()?));
            }
            return Ok(CollectionElement::If {
                condition,
                element: Box::new(element),
                else_element,
            });
        }
        if self.match_token(&TokenKind::For) {
            let item = self.parse_identifier_or_keyword()?;
            self.expect(TokenKind::In)?;
            let iterable = self.parse_pipe_expr()?;
            let element = self.parse_collection_element()?;
            return Ok(CollectionElement::For {
                item,
                iterable,
                element: Box::new(element),
            });
        }

        let expr = self.parse_expression()?;
        Ok(CollectionElement::Expr(expr))
    }

    /// Parses Dict/Set comprehensions from brace blocks `{...}`.
    pub(crate) fn try_parse_comprehension_brace(&mut self) -> Result<Option<Expression>, String> {
        let span = self.current_span();
        let checkpoint = self.cursor.clone();

        if !self.match_token(&TokenKind::LBrace) {
            return Ok(None);
        }

        let parse_inner = |parser: &mut Self| -> Result<Option<Expression>, String> {
            let key_expr = parser.parse_expression()?;
            if parser.match_token(&TokenKind::Colon) {
                let val_expr = parser.parse_expression()?;
                if parser.match_token(&TokenKind::For) {
                    let key_var = parser.parse_identifier_or_keyword()?;
                    let mut val_var = None;
                    if parser.match_token(&TokenKind::Comma) {
                        val_var = Some(parser.parse_identifier_or_keyword()?);
                    }
                    parser.expect(TokenKind::In)?;
                    let iterable = parser.parse_pipe_expr()?;
                    let mut condition = None;
                    if parser.match_token(&TokenKind::If) {
                        condition = Some(Box::new(parser.parse_expression()?));
                    }
                    parser.expect(TokenKind::RBrace)?;
                    return Ok(Some(Expression::DictComprehension {
                        key: Box::new(key_expr),
                        value: Box::new(val_expr),
                        key_var,
                        val_var,
                        iterable: Box::new(iterable),
                        condition,
                        span: span.clone(),
                    }));
                }
            } else if parser.match_token(&TokenKind::For) {
                // Set Comprehension: `{x.id for x in users if cond}`
                let var = parser.parse_identifier_or_keyword()?;
                parser.expect(TokenKind::In)?;
                let iterable = parser.parse_pipe_expr()?;
                let mut condition = None;
                if parser.match_token(&TokenKind::If) {
                    condition = Some(Box::new(parser.parse_expression()?));
                }
                parser.expect(TokenKind::RBrace)?;
                return Ok(Some(Expression::SetComprehension {
                    expr: Box::new(key_expr),
                    var,
                    iterable: Box::new(iterable),
                    condition,
                    span: span.clone(),
                }));
            }
            Ok(None)
        };

        match parse_inner(self) {
            Ok(Some(comp)) => Ok(Some(comp)),
            _ => {
                self.cursor = checkpoint;
                Ok(None)
            }
        }
    }

    /// Parses Dart-style Cascades `..` and `?..`
    pub(crate) fn parse_cascade_chain(&mut self, mut target: Expression) -> Result<Expression, String> {
        while self.check(&TokenKind::DotDot) || self.check(&TokenKind::QuestionDotDot) {
            let span = self.current_span();
            let is_null_aware = self.match_token(&TokenKind::QuestionDotDot);
            if !is_null_aware {
                self.expect(TokenKind::DotDot)?;
            }

            let member_name = self.parse_identifier_or_keyword()?;
            let op_span = self.current_span();

            let operation = if self.match_token(&TokenKind::Equal) {
                let value = self.parse_expression()?;
                Expression::Binary {
                    left: Box::new(Expression::Ident(member_name, op_span.clone())),
                    op: BinaryOp::Add, // Used as assignment placeholder
                    right: Box::new(value),
                    span: op_span,
                }
            } else if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                Expression::Call {
                    callee: Box::new(Expression::Ident(member_name, op_span.clone())),
                    args,
                    span: op_span,
                }
            } else {
                Expression::Ident(member_name, op_span)
            };

            target = Expression::Cascade {
                target: Box::new(target),
                operations: vec![operation],
                is_null_aware,
                span,
            };
        }
        Ok(target)
    }

    pub(crate) fn parse_closure_or_block(&mut self) -> Result<Expression, String> {
        let span = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let checkpoint = self.cursor.clone();

        let mut params = Vec::new();
        if self.match_token(&TokenKind::Pipe) {
            while !self.check(&TokenKind::Pipe) && !self.check(&TokenKind::EOF) {
                let p = self.parse_identifier_or_keyword()?;
                let mut pt = Type::Void;
                if self.match_token(&TokenKind::Colon) {
                    pt = self.parse_type()?;
                }
                params.push((p, pt));
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::Pipe)?;
            let mut statements = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                statements.push(self.parse_statement()?);
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Expression::Lambda {
                params: params.into_iter().map(|(name, param_type)| FunctionParam {
                    name,
                    param_type,
                    is_mut: false,
                    span: span.clone(),
                }).collect(),
                body: Box::new(Expression::Block(Block { statements, span: span.clone() })),
                is_implicit: false,
                span,
            });
        }

        if let Ok(param_name) = self.parse_identifier_or_keyword() {
            if self.match_token(&TokenKind::FatArrow) {
                let mut statements = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    statements.push(self.parse_statement()?);
                }
                self.expect(TokenKind::RBrace)?;
                return Ok(Expression::Lambda {
                    params: vec![FunctionParam {
                        name: param_name,
                        param_type: Type::Void,
                        is_mut: false,
                        span: span.clone(),
                    }],
                    body: Box::new(Expression::Block(Block { statements, span: span.clone() })),
                    is_implicit: false,
                    span,
                });
            }
        }

        self.cursor = checkpoint;
        let mut statements = Vec::new();
        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.check(&TokenKind::Fn) {
                let f = self.parse_function(false, vec![])?;
                statements.push(Statement::LocalFunction(f));
                continue;
            }
            statements.push(self.parse_statement()?);
        }
        self.expect(TokenKind::RBrace)?;
        Ok(Expression::Block(Block { statements, span }))
    }
}
