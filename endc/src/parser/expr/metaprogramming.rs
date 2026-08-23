use crate::ast::*;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_metaprogramming_expr(&mut self) -> Result<Option<Expression>, String> {
        let span = self.current_span();
        let kind = self.peek_kind().clone();
        match kind {
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
                Ok(Some(Expression::NameOf { target: target_name, span }))
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
                Ok(Some(Expression::PathOf { target: target_name, span }))
            }
            TokenKind::TypeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::TypeOf { expr: Box::new(expr), span }))
            }
            TokenKind::DocOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected identifier in docof!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::DocOf { target, span }))
            }
            TokenKind::CodeOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                let raw_code = "expr_source".to_string();
                Ok(Some(Expression::CodeOf { expr: Box::new(expr), code: raw_code, span }))
            }
            TokenKind::Dbg => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::Dbg { expr: Box::new(expr), code: "dbg_expr".to_string(), span }))
            }
            TokenKind::AssertDebug => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let cond = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::AssertDebug { condition: Box::new(cond), code: "assert_cond".to_string(), span }))
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
                Ok(Some(Expression::Translate { key, args, span }))
            }
            TokenKind::FieldsOf => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let target = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected struct identifier in fields_of!, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::FieldsOf { target, span }))
            }
            TokenKind::SqlExpr => {
                self.advance();
                self.expect(TokenKind::LParen)?;
                let expr = self.parse_expression()?;
                self.expect(TokenKind::RParen)?;
                Ok(Some(Expression::SqlExpr { expr: Box::new(expr), span }))
            }
            _ => Ok(None),
        }
    }
}
