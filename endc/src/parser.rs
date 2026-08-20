use crate::ast::*;
use crate::lexer::{Token, TokenKind};

pub struct Parser {
    tokens: Vec<Token>,
    cursor: usize,
    pub filename: String,
}

impl Parser {
    pub fn new(filename: impl Into<String>, tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            cursor: 0,
            filename: filename.into(),
        }
    }

    fn peek(&self) -> &Token {
        if self.cursor < self.tokens.len() {
            &self.tokens[self.cursor]
        } else {
            &self.tokens[self.tokens.len() - 1]
        }
    }

    fn peek_kind(&self) -> &TokenKind {
        &self.peek().kind
    }

    fn advance(&mut self) -> Token {
        if self.cursor < self.tokens.len() {
            let tok = self.tokens[self.cursor].clone();
            self.cursor += 1;
            tok
        } else {
            self.tokens[self.tokens.len() - 1].clone()
        }
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek_kind() == kind
    }

    fn match_token(&mut self, kind: &TokenKind) -> bool {
        if self.check(kind) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, String> {
        let current = self.peek();
        if std::mem::discriminant(&current.kind) == std::mem::discriminant(&kind) {
            Ok(self.advance())
        } else {
            Err(format!(
                "Expected token {:?}, found {:?} at line {}, col {}",
                kind, current.kind, current.span.line, current.span.col
            ))
        }
    }

    fn current_span(&self) -> Span {
        self.peek().span.clone()
    }

    pub fn parse_module(&mut self, module_name: &str) -> Result<Module, String> {
        let mut imports = Vec::new();
        let mut enums = Vec::new();
        let mut structs = Vec::new();
        let mut functions = Vec::new();
        let start_span = self.current_span();

        while !self.check(&TokenKind::EOF) {
            let mut pending_directives = Vec::new();
            while let TokenKind::Directive(d) = self.peek_kind() {
                let dir_name = d.clone();
                let dir_span = self.current_span();
                self.advance();
                let mut args = Vec::new();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        match self.peek_kind() {
                            TokenKind::StringLit(s) => {
                                args.push(s.clone());
                                self.advance();
                            }
                            TokenKind::Ident(i) => {
                                args.push(i.clone());
                                self.advance();
                            }
                            TokenKind::IntLit(n) => {
                                args.push(n.to_string());
                                self.advance();
                            }
                            _ => {
                                self.advance();
                            }
                        }
                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                pending_directives.push(Directive {
                    name: dir_name,
                    args,
                    span: dir_span,
                });
            }

            match self.peek_kind() {
                TokenKind::Import => {
                    imports.push(self.parse_import()?);
                }
                TokenKind::Enum => {
                    enums.push(self.parse_enum(false, pending_directives)?);
                }
                TokenKind::Struct => {
                    structs.push(self.parse_struct(false, pending_directives)?);
                }
                TokenKind::Fn => {
                    functions.push(self.parse_function(false, pending_directives)?);
                }
                TokenKind::Pub => {
                    self.advance();
                    match self.peek_kind() {
                        TokenKind::Enum => {
                            enums.push(self.parse_enum(true, pending_directives)?);
                        }
                        TokenKind::Struct => {
                            structs.push(self.parse_struct(true, pending_directives)?);
                        }
                        TokenKind::Fn => {
                            functions.push(self.parse_function(true, pending_directives)?);
                        }
                        other => {
                            return Err(format!(
                                "Expected enum, struct or fn after 'pub', found {:?} at line {}",
                                other,
                                self.current_span().line
                            ))
                        }
                    }
                }
                TokenKind::SemiColon => {
                    self.advance();
                }
                TokenKind::EOF => break,
                other => {
                    return Err(format!(
                        "Unexpected top-level token: {:?} at line {}, col {}",
                        other,
                        self.current_span().line,
                        self.current_span().col
                    ));
                }
            }
        }

        Ok(Module {
            name: module_name.to_string(),
            imports,
            enums,
            structs,
            functions,
            span: start_span,
        })
    }

    fn parse_import(&mut self) -> Result<ImportStmt, String> {
        let span = self.current_span();
        self.expect(TokenKind::Import)?;

        let (kind, path) = match self.peek_kind() {
            TokenKind::Directive(d) => {
                let dir = d.clone();
                self.advance();
                self.expect(TokenKind::LParen)?;
                let p = match self.advance().kind {
                    TokenKind::StringLit(s) => s,
                    other => return Err(format!("Expected string path in import directive, found {:?}", other)),
                };
                self.expect(TokenKind::RParen)?;

                match dir.as_str() {
                    "@c" => (ImportKind::C(p.clone()), p),
                    "@zig" => (ImportKind::Zig(p.clone()), p),
                    "@rust" => (ImportKind::Rust(p.clone()), p),
                    "@go" => (ImportKind::Go(p.clone()), p),
                    _ => (ImportKind::Standard, p),
                }
            }
            TokenKind::Ident(_) => {
                let mut full_path = String::new();
                while let TokenKind::Ident(id) = self.peek_kind() {
                    full_path.push_str(id);
                    self.advance();
                    if self.match_token(&TokenKind::Dot) {
                        full_path.push('.');
                    } else {
                        break;
                    }
                }
                (ImportKind::Standard, full_path.clone())
            }
            other => return Err(format!("Invalid import syntax: {:?} at line {}", other, span.line)),
        };

        let mut alias = None;
        if self.match_token(&TokenKind::As) {
            match self.advance().kind {
                TokenKind::Ident(a) => alias = Some(a),
                other => return Err(format!("Expected alias identifier after 'as', found {:?}", other)),
            }
        }

        self.match_token(&TokenKind::SemiColon);

        Ok(ImportStmt {
            kind,
            path,
            alias,
            span,
        })
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        let _span = self.current_span();
        if self.match_token(&TokenKind::Bang) {
            let inner = self.parse_type()?;
            return Ok(Type::Result(Box::new(inner), None));
        }

        if self.match_token(&TokenKind::Star) {
            let inner = self.parse_type()?;
            return Ok(Type::Pointer(Box::new(inner)));
        }

        if self.match_token(&TokenKind::LBracket) {
            if self.match_token(&TokenKind::RBracket) {
                let inner = self.parse_type()?;
                return Ok(Type::Slice(Box::new(inner)));
            } else if let TokenKind::IntLit(n) = self.peek_kind() {
                let size = *n as usize;
                self.advance();
                self.expect(TokenKind::RBracket)?;
                let inner = self.parse_type()?;
                return Ok(Type::Array(Box::new(inner), size));
            }
        }

        match self.peek_kind() {
            TokenKind::Ident(name) => {
                let type_name = name.clone();
                self.advance();
                let ty = match type_name.as_str() {
                    "void" => Type::Void,
                    "bool" => Type::Bool,
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" | "int" => Type::I32,
                    "i64" => Type::I64,
                    "u8" | "byte" => Type::U8,
                    "u16" => Type::U16,
                    "u32" => Type::U32,
                    "u64" => Type::U64,
                    "f32" | "float" => Type::F32,
                    "f64" => Type::F64,
                    "str" | "string" => Type::Str,
                    "Allocator" => Type::Allocator,
                    "region" => {
                        if self.match_token(&TokenKind::Less) {
                            let reg_name = match self.advance().kind {
                                TokenKind::Ident(s) => s,
                                other => return Err(format!("Expected region name, found {:?}", other)),
                            };
                            self.expect(TokenKind::Greater)?;
                            Type::Region(reg_name)
                        } else {
                            Type::Region("default".into())
                        }
                    }
                    other => {
                        // Check for Generic arguments: `List<User>`
                        if self.match_token(&TokenKind::Less) {
                            let mut params = Vec::new();
                            while !self.check(&TokenKind::Greater) && !self.check(&TokenKind::EOF) {
                                params.push(self.parse_type()?);
                                if !self.match_token(&TokenKind::Comma) {
                                    break;
                                }
                            }
                            self.expect(TokenKind::Greater)?;
                            Type::Generic(other.to_string(), params)
                        } else {
                            Type::Custom(other.to_string())
                        }
                    }
                };
                Ok(ty)
            }
            other => Err(format!("Expected type, found {:?} at line {}", other, self.current_span().line)),
        }
    }

    fn parse_enum(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<EnumDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Enum)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected enum name, found {:?} at line {}", other, span.line)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut variants = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let vspan = self.current_span();
            let vname = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name, found {:?}", other)),
            };

            let mut payload = None;
            if self.match_token(&TokenKind::LParen) {
                payload = Some(self.parse_type()?);
                self.expect(TokenKind::RParen)?;
            }

            self.match_token(&TokenKind::Comma);

            variants.push(EnumVariant {
                name: vname,
                payload,
                span: vspan,
            });
        }

        self.expect(TokenKind::RBrace)?;

        Ok(EnumDef {
            name,
            is_pub,
            variants,
            directives,
            span,
        })
    }

    fn parse_struct(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<StructDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Struct)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected struct name, found {:?} at line {}", other, span.line)),
        };

        self.expect(TokenKind::LBrace)?;
        let mut fields = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            let field_span = self.current_span();
            let is_field_pub = self.match_token(&TokenKind::Pub);
            let field_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected field name, found {:?}", other)),
            };

            self.expect(TokenKind::Colon)?;
            let field_type = self.parse_type()?;
            self.match_token(&TokenKind::Comma);
            self.match_token(&TokenKind::SemiColon);

            fields.push(StructField {
                name: field_name,
                field_type,
                is_pub: is_field_pub,
                span: field_span,
            });
        }

        self.expect(TokenKind::RBrace)?;

        Ok(StructDef {
            name,
            is_pub,
            fields,
            directives,
            span,
        })
    }

    fn parse_function(&mut self, is_pub: bool, directives: Vec<Directive>) -> Result<FunctionDef, String> {
        let span = self.current_span();
        self.expect(TokenKind::Fn)?;

        let name = match self.advance().kind {
            TokenKind::Ident(n) => n,
            other => return Err(format!("Expected function name, found {:?} at line {}", other, span.line)),
        };

        self.expect(TokenKind::LParen)?;
        let mut params = Vec::new();

        while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
            let p_span = self.current_span();
            let is_mut = self.match_token(&TokenKind::Mut);
            let param_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected parameter name, found {:?}", other)),
            };

            let mut param_type = Type::Void;
            if self.match_token(&TokenKind::Colon) {
                param_type = self.parse_type()?;
            }

            params.push(FunctionParam {
                name: param_name,
                param_type,
                is_mut,
                span: p_span,
            });

            if !self.match_token(&TokenKind::Comma) {
                break;
            }
        }
        self.expect(TokenKind::RParen)?;

        // Return type
        let return_type = if self.match_token(&TokenKind::Arrow) {
            self.parse_type()?
        } else if self.check(&TokenKind::Bang)
            || matches!(self.peek_kind(), TokenKind::Ident(_))
            || self.check(&TokenKind::Star)
            || self.check(&TokenKind::LBracket)
        {
            self.parse_type()?
        } else {
            Type::Void
        };

        let body = self.parse_block()?;

        Ok(FunctionDef {
            name,
            is_pub,
            params,
            return_type,
            body,
            directives,
            span,
        })
    }

    fn parse_block(&mut self) -> Result<Block, String> {
        let span = self.current_span();
        self.expect(TokenKind::LBrace)?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
            statements.push(self.parse_statement()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Block { statements, span })
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        let span = self.current_span();

        match self.peek_kind() {
            TokenKind::Val | TokenKind::Mut => {
                let is_mut = self.peek_kind() == &TokenKind::Mut;
                self.advance();

                let name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected variable name, found {:?}", other)),
                };

                let mut var_type = None;
                if self.match_token(&TokenKind::Colon) {
                    var_type = Some(self.parse_type()?);
                }

                let mut initializer = None;
                if self.match_token(&TokenKind::Equal) {
                    initializer = Some(self.parse_expression()?);
                }

                self.match_token(&TokenKind::SemiColon);

                Ok(Statement::VarDecl {
                    name,
                    var_type,
                    is_mut,
                    initializer,
                    span,
                })
            }
            TokenKind::Return => {
                self.advance();
                let value = if !self.check(&TokenKind::SemiColon) && !self.check(&TokenKind::RBrace) {
                    Some(self.parse_expression()?)
                } else {
                    None
                };
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Return { value, span })
            }
            TokenKind::If => {
                self.advance();
                let condition = self.parse_expression()?;
                let then_block = self.parse_block()?;
                let mut else_block = None;
                if self.match_token(&TokenKind::Else) {
                    if self.check(&TokenKind::If) {
                        let if_stmt = self.parse_statement()?;
                        else_block = Some(Block {
                            statements: vec![if_stmt],
                            span: self.current_span(),
                        });
                    } else {
                        else_block = Some(self.parse_block()?);
                    }
                }
                Ok(Statement::If {
                    condition,
                    then_block,
                    else_block,
                    span,
                })
            }
            TokenKind::While => {
                self.advance();
                let condition = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::While {
                    condition,
                    body,
                    span,
                })
            }
            TokenKind::For => {
                self.advance();
                let item_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected item name after 'for', found {:?}", other)),
                };
                self.expect(TokenKind::In)?;
                let iterable = self.parse_expression()?;
                let body = self.parse_block()?;
                Ok(Statement::ForIn {
                    item_name,
                    iterable,
                    body,
                    span,
                })
            }
            TokenKind::Match => {
                self.advance();
                let expr = self.parse_expression()?;
                self.expect(TokenKind::LBrace)?;
                let mut arms = Vec::new();
                while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                    arms.push(self.parse_match_arm()?);
                    self.match_token(&TokenKind::Comma);
                }
                self.expect(TokenKind::RBrace)?;
                Ok(Statement::Match { expr, arms, span })
            }
            TokenKind::Region => {
                self.advance();
                let reg_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected region name, found {:?}", other)),
                };
                let body = self.parse_block()?;
                Ok(Statement::RegionBlock {
                    name: reg_name,
                    body,
                    span,
                })
            }
            TokenKind::Defer => {
                self.advance();
                let expr = self.parse_expression()?;
                self.match_token(&TokenKind::SemiColon);
                Ok(Statement::Defer { expr, span })
            }
            _ => {
                let expr = self.parse_expression()?;
                if self.match_token(&TokenKind::Equal) {
                    let value = self.parse_expression()?;
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::Assignment {
                        target: expr,
                        value,
                        span,
                    })
                } else {
                    self.match_token(&TokenKind::SemiColon);
                    Ok(Statement::Expression(expr))
                }
            }
        }
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, String> {
        let span = self.current_span();
        let pattern = self.parse_pattern()?;

        let mut guard = None;
        if self.match_token(&TokenKind::If) {
            guard = Some(self.parse_expression()?);
        }

        self.expect(TokenKind::FatArrow)?;

        let body = if self.check(&TokenKind::LBrace) {
            self.parse_block()?
        } else {
            let stmt = self.parse_statement()?;
            Block {
                statements: vec![stmt],
                span: span.clone(),
            }
        };

        Ok(MatchArm {
            pattern,
            guard,
            body,
            span,
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, String> {
        if self.match_token(&TokenKind::Underscore) {
            return Ok(Pattern::Wildcard);
        }

        if self.match_token(&TokenKind::Dot) {
            let variant_name = match self.advance().kind {
                TokenKind::Ident(n) => n,
                other => return Err(format!("Expected variant name after '.', found {:?}", other)),
            };

            let mut binding = None;
            if self.match_token(&TokenKind::LParen) {
                if let TokenKind::Ident(b) = self.advance().kind {
                    binding = Some(b);
                }
                self.expect(TokenKind::RParen)?;
            }

            return Ok(Pattern::Variant {
                enum_name: None,
                variant_name,
                binding,
            });
        }

        match self.peek_kind() {
            TokenKind::IntLit(n) => {
                let val = *n;
                self.advance();
                Ok(Pattern::Literal(Literal::Int(val)))
            }
            TokenKind::StringLit(s) => {
                let val = s.clone();
                self.advance();
                Ok(Pattern::Literal(Literal::String(val)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(Literal::Bool(false)))
            }
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();
                if self.match_token(&TokenKind::Dot) {
                    let vname = match self.advance().kind {
                        TokenKind::Ident(n) => n,
                        other => return Err(format!("Expected variant name, found {:?}", other)),
                    };
                    let mut binding = None;
                    if self.match_token(&TokenKind::LParen) {
                        if let TokenKind::Ident(b) = self.advance().kind {
                            binding = Some(b);
                        }
                        self.expect(TokenKind::RParen)?;
                    }
                    Ok(Pattern::Variant {
                        enum_name: Some(id),
                        variant_name: vname,
                        binding,
                    })
                } else {
                    Ok(Pattern::Ident(id))
                }
            }
            other => Err(format!("Invalid pattern syntax: {:?} at line {}", other, self.current_span().line)),
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_catch_expr()
    }

    fn parse_catch_expr(&mut self) -> Result<Expression, String> {
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

    fn parse_logical_or(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_logical_and()?;
        while self.match_token(&TokenKind::Pipe) {
            let span = self.current_span();
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::Or,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_equality()?;
        while self.match_token(&TokenKind::Ampersand) {
            let span = self.current_span();
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                op: BinaryOp::And,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, String> {
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

    fn parse_comparison(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_addition()?;
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
            let right = self.parse_addition()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_addition(&mut self) -> Result<Expression, String> {
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

    fn parse_multiplication(&mut self) -> Result<Expression, String> {
        let mut left = self.parse_unary()?;
        while self.check(&TokenKind::Star) || self.check(&TokenKind::Slash) || self.check(&TokenKind::Percent) {
            let op = if self.match_token(&TokenKind::Star) {
                BinaryOp::Mul
            } else if self.match_token(&TokenKind::Slash) {
                BinaryOp::Div
            } else {
                self.advance();
                BinaryOp::Mod
            };
            let span = self.current_span();
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
                span,
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
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

        self.parse_postfix()
    }

    fn parse_postfix(&mut self) -> Result<Expression, String> {
        let mut expr = self.parse_primary()?;

        loop {
            let span = self.current_span();
            if self.match_token(&TokenKind::Dot) {
                let member_name = match self.advance().kind {
                    TokenKind::Ident(n) => n,
                    other => return Err(format!("Expected member identifier after '.', found {:?}", other)),
                };
                expr = Expression::FieldAccess {
                    object: Box::new(expr),
                    field: member_name,
                    span,
                };
            } else if self.match_token(&TokenKind::LParen) {
                let mut args = Vec::new();
                while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                    args.push(self.parse_expression()?);
                    if !self.match_token(&TokenKind::Comma) {
                        break;
                    }
                }
                self.expect(TokenKind::RParen)?;
                expr = Expression::Call {
                    callee: Box::new(expr),
                    args,
                    span,
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        let span = self.current_span();

        if self.match_token(&TokenKind::Dot) {
            // Enum variant literal: `.Pending` or `.Success("ok")`
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
            TokenKind::Ident(name) => {
                let id = name.clone();
                self.advance();

                // Check for Struct Initialization: `User { id: 1, name: "Ali" }`
                if self.check(&TokenKind::LBrace) && id.chars().next().map_or(false, |c| c.is_uppercase()) {
                    self.advance(); // consume '{'
                    let mut fields = Vec::new();
                    while !self.check(&TokenKind::RBrace) && !self.check(&TokenKind::EOF) {
                        let fname = match self.advance().kind {
                            TokenKind::Ident(n) => n,
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

                // Check for Enum Qualified Init: `Status.Pending` or `Status.Failed("network")`
                if self.match_token(&TokenKind::Dot) {
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
            TokenKind::Alloc => {
                self.advance();
                let target_type = self.parse_type()?;
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
