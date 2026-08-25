use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_primary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();

        if self.match_token(&TokenKind::If) {
            let cond = self.parse_expression()?;
            let then_block = self.parse_block()?;
            let mut else_branch = None;
            if self.match_token(&TokenKind::Else) {
                if self.check(&TokenKind::LBrace) {
                    let eb = self.parse_block()?;
                    else_branch = Some(Expression::Block(eb));
                } else if self.check(&TokenKind::If) {
                    let nested_if = self.parse_primary()?;
                    else_branch = Some(nested_if);
                } else {
                    let ee = self.parse_expression()?;
                    else_branch = Some(ee);
                }
            }
            return Ok(Expression::Conditional {
                condition: Box::new(cond),
                then_branch: Box::new(Expression::Block(then_block)),
                else_branch: Box::new(else_branch.unwrap_or_else(|| Expression::Lit(Literal::Null, span.clone()))),
                span,
            });
        }

        if self.match_token(&TokenKind::Fn) {
            let span = self.current_span();
            self.expect(TokenKind::LParen)?;
            let mut params = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                let p_span = self.current_span();
                let is_mut = self.match_token(&TokenKind::Mut);
                let p_name = self.parse_identifier_or_keyword()?;
                let mut p_ty = Type::Void;
                if self.match_token(&TokenKind::Colon) {
                    p_ty = self.parse_type()?;
                }
                params.push(FunctionParam {
                    name: p_name,
                    param_type: p_ty,
                    is_mut,
                    span: p_span,
                });
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            if self.match_token(&TokenKind::Arrow) {
                let _ret_ty = self.parse_type()?;
            }
            let body = self.parse_block()?;
            return Ok(Expression::Lambda {
                params,
                body: Box::new(Expression::Block(body)),
                is_implicit: false,
                span,
            });
        }

        if self.match_token(&TokenKind::Match) {
            let expr = self.parse_expression()?;
            self.expect(TokenKind::LBrace)?;
            let mut arms = Vec::new();
            while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                let arm_span = self.current_span();
                let pattern = self.parse_pattern()?;
                let mut guard = None;
                if self.match_token(&TokenKind::If) {
                    guard = Some(self.parse_expression()?);
                }
                self.expect(TokenKind::FatArrow)?;
                let body = if self.check(&TokenKind::LBrace) {
                    self.parse_block()?
                } else {
                    let expr = self.parse_expression()?;
                    self.match_token(&TokenKind::Comma);
                    Block {
                        statements: vec![Statement::Expression(expr)],
                        span: arm_span.clone(),
                    }
                };
                self.match_token(&TokenKind::Comma);
                arms.push(MatchArm {
                    pattern,
                    guard,
                    body,
                    span: arm_span,
                });
            }
            self.expect(TokenKind::RBrace)?;
            return Ok(Expression::Match {
                expr: Box::new(expr),
                arms,
                span,
            });
        }

        if self.match_token(&TokenKind::Dot) {
            let vname = self.parse_identifier_or_keyword()?;

            let mut payload = None;
            if self.match_token(&TokenKind::LParen) {
                payload = Some(Box::new(self.parse_expression()?));
                self.expect(TokenKind::RParen)?;
            }

            return Ok(Expression::EnumInit {
                enum_name: None,
                variant_name: vname,
                payload,
                span,
            });
        }

        if let Some(meta_expr) = self.parse_metaprogramming_expr()? {
            return Ok(meta_expr);
        }

        let kind = self.peek_kind().clone();
        match kind {
            TokenKind::Operation => {
                let op = self.parse_operation(false)?;
                return Ok(Expression::OperationLiteral {
                    name: if op.name.is_empty() { None } else { Some(op.name) },
                    params: op.params,
                    return_type: op.return_type,
                    requires: op.requires,
                    guarantees: op.guarantees,
                    effects: op.effects,
                    emits: op.emits,
                    body: op.body,
                    span,
                });
            }
            TokenKind::Compose => {
                self.advance();
                self.expect(TokenKind::LBrace)?;
                let mut ops = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    if self.match_token(&TokenKind::Then) {
                        continue;
                    }
                    ops.push(self.parse_expression()?);
                    self.match_token(&TokenKind::Then);
                    self.match_token(&TokenKind::Comma);
                    self.match_token(&TokenKind::SemiColon);
                }
                self.expect(TokenKind::RBrace)?;
                return Ok(Expression::Compose { ops, span });
            }
            TokenKind::Retry => {
                self.advance();
                let op = self.parse_primary()?;
                let mut count = Expression::Lit(Literal::Int(3), span.clone());
                if self.peek_kind() == &TokenKind::Ident("up".to_string()) {
                    self.advance();
                    if self.peek_kind() == &TokenKind::To {
                        self.advance();
                    }
                }
                if let TokenKind::IntLit(_) = self.peek_kind() {
                    count = self.parse_primary()?;
                }
                return Ok(Expression::Repeat {
                    op: Box::new(op),
                    count: Box::new(count),
                    is_retry: true,
                    span,
                });
            }
            TokenKind::Repeat => {
                self.advance();
                let op = self.parse_primary()?;
                let mut count = Expression::Lit(Literal::Int(1), span.clone());
                if let TokenKind::IntLit(_) = self.peek_kind() {
                    count = self.parse_primary()?;
                }
                return Ok(Expression::Repeat {
                    op: Box::new(op),
                    count: Box::new(count),
                    is_retry: false,
                    span,
                });
            }
            TokenKind::Memoize => {
                self.advance();
                let op = self.parse_primary()?;
                return Ok(Expression::Memoize {
                    op: Box::new(op),
                    span,
                });
            }
            TokenKind::IntLit(n) => {
                let val = n;
                self.advance();
                Ok(Expression::Lit(Literal::Int(val), span))
            }
            TokenKind::FloatLit(f) => {
                let val = f;
                self.advance();
                Ok(Expression::Lit(Literal::Float(val), span))
            }
            TokenKind::UnitLit(val, unit) => {
                let v = val;
                let u = unit.clone();
                self.advance();
                Ok(Expression::UnitLit { value: v, unit: u, span })
            }
            TokenKind::StringLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Expression::Lit(Literal::String(val), span))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Lit(Literal::Bool(true), span))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Lit(Literal::Bool(false), span))
            }
            TokenKind::Null => {
                self.advance();
                Ok(Expression::Lit(Literal::Null, span))
            }
            TokenKind::Underscore => {
                self.advance();
                Ok(Expression::Ident("_".to_string(), span))
            }
            TokenKind::DotDotDot => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Expression::Spread {
                    expr: Box::new(expr),
                    is_null_aware: false,
                    span,
                })
            }
            TokenKind::DotDotDotQuestion => {
                self.advance();
                let expr = self.parse_expression()?;
                Ok(Expression::Spread {
                    expr: Box::new(expr),
                    is_null_aware: true,
                    span,
                })
            }
            TokenKind::LBracket => {
                self.parse_bracket_collection()
            }
            TokenKind::LParen => {
                self.advance();
                if self.match_token(&TokenKind::RParen) {
                    return Ok(Expression::Tuple(Vec::new(), span));
                }

                // Check for Walrus Assignment: `(n := get_number())`
                let checkpoint = self.checkpoint();
                if let Ok(var_name) = self.parse_identifier_or_keyword() {
                    if self.match_token(&TokenKind::ColonEqual) {
                        let val_expr = self.parse_expression()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expression::Walrus {
                            name: var_name,
                            expr: Box::new(val_expr),
                            span,
                        });
                    }
                }
                self.restore_checkpoint(checkpoint);

                let first_expr = self.parse_expression()?;
                if self.match_token(&TokenKind::Comma) {
                    let mut elements = vec![first_expr];
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        elements.push(self.parse_expression()?);
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                    Ok(Expression::Tuple(elements, span))
                } else {
                    self.expect(TokenKind::RParen)?;
                    Ok(first_expr)
                }
            }
            TokenKind::LBrace => {
                if let Ok(Some(comp)) = self.try_parse_comprehension_brace() {
                    return Ok(comp);
                }
                let blk = self.parse_block()?;
                Ok(Expression::Block(blk))
            }
            TokenKind::Alloc => {
                self.advance();
                let has_paren = self.match_token(&TokenKind::LParen);
                let target_type = self.parse_type()?;
                if has_paren {
                    self.match_token(&TokenKind::RParen);
                }
                Ok(Expression::Alloc {
                    allocator: Box::new(Expression::Ident("default_allocator".into(), span.clone())),
                    target_type,
                    span,
                })
            }
            _ => {
                let checkpoint = self.checkpoint();
                if let Ok(id) = self.parse_identifier_or_keyword() {
                    // Check for Region Promotion: `promote(temp, outer_scope)`
                    if id == "promote" && self.match_token(&TokenKind::LParen) {
                        let expr = self.parse_expression()?;
                        self.expect(TokenKind::Comma)?;
                        let target_region = self.parse_identifier_or_keyword()?;
                        self.expect(TokenKind::RParen)?;
                        return Ok(Expression::Promote {
                            expr: Box::new(expr),
                            target_region,
                            span,
                        });
                    }

                    // Check for Struct Initialization: `User { id: 1, name: "Ali" }`
                    if self.check(&TokenKind::LBrace) && id.chars().next().map_or(false, |c| c.is_uppercase()) {
                        self.advance();
                        let mut fields = Vec::new();
                        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                            let fname = self.parse_identifier_or_keyword()?;
                            let mut fvalue = Expression::Ident(fname.clone(), self.current_span());
                            if self.match_token(&TokenKind::Colon) {
                                fvalue = self.parse_expression()?;
                            }
                            fields.push((fname, fvalue));
                            if !self.match_token(&TokenKind::Comma) {
                                break;
                            }
                        }
                        self.expect(TokenKind::RBrace)?;
                        return Ok(Expression::StructInit {
                            name: id,
                            fields,
                            span,
                        });
                    }

                    // Check for Enum Qualified Init: `Status.Pending` or `Status::Ok`
                    let is_enum_access = if self.enum_names.contains(&id) {
                        if self.match_token(&TokenKind::Dot) {
                            true
                        } else if self.check(&TokenKind::Colon) && self.peek_next_kind().map_or(false, |k| matches!(k, TokenKind::Colon)) {
                            self.advance();
                            self.advance();
                            true
                        } else {
                            false
                        }
                    } else if self.check(&TokenKind::Colon) && self.peek_next_kind().map_or(false, |k| matches!(k, TokenKind::Colon)) {
                        self.advance();
                        self.advance();
                        true
                    } else {
                        false
                    };

                    if is_enum_access {
                        let vname = self.parse_identifier_or_keyword()?;
                        let mut payload = None;
                        if self.match_token(&TokenKind::LParen) {
                            payload = Some(Box::new(self.parse_expression()?));
                            self.expect(TokenKind::RParen)?;
                        }
                        return Ok(Expression::EnumInit {
                            enum_name: Some(id),
                            variant_name: vname,
                            payload,
                            span,
                        });
                    }

                    return Ok(Expression::Ident(id, span));
                }
                self.restore_checkpoint(checkpoint);

                let other = self.peek_kind().clone();
                let actual = format!("{:?}", other);
                let expected = "expression".to_string();
                let raw_msg = format!(
                    "Unexpected token in expression: {:?} at line {}, col {}",
                    other, span.line, span.col
                );
                let formatted = self.emit_e005(&span, &expected, &actual, &raw_msg);
                Err(formatted)
            }
        }
    }
}
