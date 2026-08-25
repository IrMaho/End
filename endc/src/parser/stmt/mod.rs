use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

pub mod agent_workflow;
pub mod architecture_boundaries;
pub mod architecture_metrics;
pub mod capabilities;
pub mod capability_extensions;
pub mod compiler_extensions;
pub mod contracts_proofs;
pub mod control_flow;
pub mod governance_proposals;
pub mod loops_branches;
pub mod reactive_events;
pub mod event_streams_and_topologies;
pub mod inheritance_ops;
pub mod refactoring_ops;
pub mod syntax_composition;

impl Parser {
    pub(crate) fn parse_block(&mut self) -> Result<Block, String> {
        let span = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            if self.match_token(&TokenKind::SemiColon) {
                continue;
            }
            if self.check(&TokenKind::Fn) || (if let TokenKind::Ident(s) = self.peek_kind() { s == "fn" } else { false }) {
                match self.parse_function(false, vec![]) {
                    Ok(f) => statements.push(Statement::LocalFunction(f)),
                    Err(_) => {
                        self.synchronize();
                        if self.check(&TokenKind::RBrace) || self.check(&TokenKind::EOF) {
                            break;
                        }
                    }
                }
                continue;
            }
            match self.parse_statement() {
                Ok(stmt) => statements.push(stmt),
                Err(_) => {
                    self.synchronize();
                    if self.check(&TokenKind::RBrace) || self.check(&TokenKind::EOF) {
                        break;
                    }
                }
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Block { statements, span })
    }



    pub(crate) fn parse_statement(&mut self) -> Result<Statement, String> {
        let span = self.current_span();
        let peek_k = self.peek_kind().clone();

        if let Some(stmt) = self.parse_capability_composition_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_control_flow_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_loops_branches_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_contracts_proofs_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_inheritance_ops_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_reactive_events_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_event_streams_and_topologies_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_agent_workflow_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_refactoring_ops_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_architecture_boundaries_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_architecture_metrics_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_governance_proposals_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_syntax_composition_statement(&peek_k, &span)? {
            return Ok(stmt);
        }
        if let Some(stmt) = self.parse_compiler_extensions_statement(&peek_k, &span)? {
            return Ok(stmt);
        }

        self.parse_fallback_statement(span)
    }

    pub(crate) fn parse_fallback_statement(&mut self, span: Span) -> Result<Statement, String> {
        let expr = self.parse_expression()?;
        if self.match_token(&TokenKind::ColonEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            let pat = match expr {
                Expression::Tuple(elems, _) => {
                    let patterns = elems.into_iter().map(|e| match e {
                        Expression::Ident(n, _) => {
                            if n == "_" {
                                Pattern::Wildcard
                            } else {
                                Pattern::Binding(n)
                            }
                        }
                        Expression::Spread { expr, .. } => {
                            if let Expression::Ident(n, _) = *expr {
                                Pattern::Binding(format!("*{}", n))
                            } else {
                                Pattern::Wildcard
                            }
                        }
                        _ => Pattern::Wildcard,
                    }).collect();
                    Pattern::Tuple(patterns)
                }
                Expression::Ident(n, _) => {
                    if n == "_" {
                        Pattern::Wildcard
                    } else {
                        Pattern::Binding(n)
                    }
                }
                Expression::StructInit { name, fields, .. } => {
                    let pfields = fields.into_iter().map(|(f, _)| (f.clone(), Pattern::Binding(f))).collect();
                    Pattern::Struct { name, fields: pfields }
                }
                _ => Pattern::Wildcard,
            };
            return Ok(Statement::DestructureAssign {
                pattern: pat,
                value,
                span,
            });
        } else if self.match_token(&TokenKind::Equal) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr,
                value,
                span,
            })
        } else if self.match_token(&TokenKind::PlusEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Add,
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::MinusEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Sub,
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::StarEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Mul,
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::SlashEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Div,
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::PercentEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::Binary {
                    left: Box::new(expr),
                    op: BinaryOp::Mod,
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::QuestionQuestionEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Assignment {
                target: expr.clone(),
                value: Expression::NullCollapse {
                    left: Box::new(expr),
                    right: Box::new(value),
                    span: span.clone(),
                },
                span,
            })
        } else if self.match_token(&TokenKind::LessPlusEqual) {
            let value = self.parse_expression()?;
            self.match_token(&TokenKind::SemiColon);
            if let Expression::Ident(target_name, _) = &expr {
                Ok(Statement::AtomicOp {
                    target: target_name.clone(),
                    op: BinaryOp::Add,
                    value,
                    span,
                })
            } else {
                Ok(Statement::Assignment {
                    target: expr.clone(),
                    value: Expression::Binary {
                        left: Box::new(expr),
                        op: BinaryOp::Add,
                        right: Box::new(value),
                        span: span.clone(),
                    },
                    span,
                })
            }
        } else {
            self.match_token(&TokenKind::SemiColon);
            Ok(Statement::Expression(expr))
        }
    }
}
