use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_primary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();

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
            let vname = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name after '.', found {:?}", other)),
            };

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

        match self.peek_kind() {
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
            TokenKind::NameOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut target_name = String::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(n) => target_name.push_str(&n),
                        TokenKind::Dot => target_name.push('.'),
                        _ => {}
                    }
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::NameOf { target: target_name, span });
            }
            TokenKind::PathOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let mut target_name = String::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    let tok = self.advance();
                    match tok.kind {
                        TokenKind::Ident(n) => target_name.push_str(&n),
                        TokenKind::Dot => target_name.push('.'),
                        _ => {}
                    }
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::PathOf { target: target_name, span });
            }
            TokenKind::TypeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::TypeOf { expr: Box::new(expr), span });
            }
            TokenKind::DocOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected identifier in docof!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::DocOf { target, span });
            }
            TokenKind::CodeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let raw_code = "expr_source".to_string();
                return Ok(Expression::CodeOf { expr: Box::new(expr), code: raw_code, span });
            }
            TokenKind::Dbg => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::Dbg { expr: Box::new(expr), code: "dbg_expr".to_string(), span });
            }
            TokenKind::AssertDebug => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let cond = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::AssertDebug { condition: Box::new(cond), code: "assert_cond".to_string(), span });
            }
            TokenKind::Translate => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let key = match self.advance().kind {
                    TokenKind::StringLit(s) => s,
                    other => return Err(format!("Expected string key in t!, found {:?}", other)),
                };
                let mut args = Vec::new();
                while self.match_token(&TokenKind::Comma) {
                    if self.check(&TokenKind::RParen) {
                        break;
                    }
                    let arg_name = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected argument name in t!, found {:?}", other)),
                    };
                    self.expect(TokenKind::Equal)?;
                    let arg_val = self.parse_expression()?;
                    args.push((arg_name, arg_val));
                }
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::Translate { key, args, span });
            }
            TokenKind::FieldsOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected struct identifier in fields_of!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::FieldsOf { target, span });
            }
            TokenKind::SqlExpr => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                return Ok(Expression::SqlExpr { expr: Box::new(expr), span });
            }
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Expression::Lit(Literal::Int(val), span))
            }
            TokenKind::FloatLit(f) => {
                let val = *f;
                self.advance();
                Ok(Expression::Lit(Literal::Float(val), span))
            }
            TokenKind::UnitLit(val, unit) => {
                let v = *val;
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
            TokenKind::Struct => {
                self.advance();
                Ok(Expression::Ident("st".to_string(), span))
            }
            TokenKind::Val => {
                self.advance();
                Ok(Expression::Ident("val".to_string(), span))
            }
            TokenKind::Mut => {
                self.advance();
                Ok(Expression::Ident("mut".to_string(), span))
            }
            TokenKind::Target => {
                self.advance();
                Ok(Expression::Ident("target".to_string(), span))
            }
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();

                // Check for Region Promotion: `promote(temp, outer_scope)`
                if id == "promote" && self.match_token(&TokenKind::LParen) {
                    let expr = self.parse_expression()?;
                    self.expect(TokenKind::Comma)?;
                    let target_region = match self.advance().kind {
                        TokenKind::Ident(r) => r,
                        other => return Err(format!("Expected target region name in promote, found {:?}", other)),
                    };
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
                        let fname = match self.advance().kind {
                            TokenKind::Ident(n) => n,
                            TokenKind::Struct => "st".to_string(),
                            TokenKind::Val => "val".to_string(),
                            TokenKind::Mut => "mut".to_string(),
                            TokenKind::Target => "target".to_string(),
                            TokenKind::Match => "match".to_string(),
                            TokenKind::Fn => "fn".to_string(),
                            TokenKind::In => "in".to_string(),
                            TokenKind::Asm => "asm".to_string(),
                            TokenKind::Region => "region".to_string(),
                            other => return Err(format!("Expected field name in struct init, found {:?}", other)),
                        };
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
                    } else if self.check(&TokenKind::Colon) {
                        self.advance();
                        self.match_token(&TokenKind::Colon)
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_enum_access {
                    let vname = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected enum variant name, found {:?}", other)),
                    };
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

                Ok(Expression::Ident(id, span))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LBrace => {
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
            other => Err(format!(
                "Unexpected token in expression: {:?} at line {}, col {}",
                other, span.line, span.col
            )),
        }
    }
}
