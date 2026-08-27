use crate::ast::Type;
use crate::lexer::TokenKind;
use crate::parser::Parser;

impl Parser {
    pub(crate) fn parse_type(&mut self) -> Result<Type, String> {
        let _span = self.current_span();
        if self.match_token(&TokenKind::Bang) {
            let inner = self.parse_type()?;
            return Ok(Type::Result(Box::new(inner), None));
        }

        if self.match_token(&TokenKind::StarStar) {
            let inner = self.parse_type()?;
            return Ok(Type::Pointer(Box::new(Type::Pointer(Box::new(inner)))));
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
            } else {
                let inner = self.parse_type()?;
                self.expect(TokenKind::RBracket)?;
                return Ok(Type::Slice(Box::new(inner)));
            }
        }

        if self.match_token(&TokenKind::LParen) {
            let mut tuple_types = Vec::new();
            while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                tuple_types.push(self.parse_type()?);
                if !self.match_token(&TokenKind::Comma) {
                    break;
                }
            }
            self.expect(TokenKind::RParen)?;
            return Ok(Type::Tuple(tuple_types));
        }

        match self.peek_kind() {
            TokenKind::Operation => {
                self.advance();
                if self.match_token(&TokenKind::Less) {
                    let tin = self.parse_type()?;
                    let mut tout = None;
                    if self.match_token(&TokenKind::Comma) {
                        tout = Some(Box::new(self.parse_type()?));
                    }
                    self.expect(TokenKind::Greater)?;
                    Ok(Type::Operation(Some(Box::new(tin)), tout))
                } else {
                    Ok(Type::Operation(None, None))
                }
            }
            TokenKind::Event => {
                self.advance();
                if self.match_token(&TokenKind::Less) {
                    let ev_name = self.parse_identifier_or_keyword()?;
                    self.expect(TokenKind::Greater)?;
                    Ok(Type::Event(ev_name))
                } else {
                    Ok(Type::Event("Any".into()))
                }
            }
            TokenKind::Fn => {
                self.advance();
                if self.match_token(&TokenKind::LParen) {
                    while !self.check(&TokenKind::RParen) && !self.check(&TokenKind::EOF) {
                        self.parse_type()?;
                        if !self.match_token(&TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect(TokenKind::RParen)?;
                }
                if self.match_token(&TokenKind::Arrow) {
                    let ret_ty = self.parse_type()?;
                    Ok(Type::Custom(format!("fn_to_{:?}", ret_ty)))
                } else {
                    Ok(Type::Custom("fn".into()))
                }
            }
            _ => {
                let type_name = self.parse_identifier_or_keyword()?;
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
                    "f32x4" => Type::Simd(Box::new(Type::F32), 4),
                    "f32x8" => Type::Simd(Box::new(Type::F32), 8),
                    "i32x4" => Type::Simd(Box::new(Type::I32), 4),
                    "i32x8" => Type::Simd(Box::new(Type::I32), 8),
                    "str" | "string" => Type::Str,
                    "Allocator" => Type::Allocator,
                    "Box" | "box" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Box(Box::new(inner))
                    }
                    "Rc" | "rc" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Rc(Box::new(inner))
                    }
                    "Arc" | "arc" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Arc(Box::new(inner))
                    }
                    "Channel" | "channel" => {
                        self.expect(TokenKind::Less)?;
                        let inner = self.parse_type()?;
                        self.expect(TokenKind::Greater)?;
                        Type::Channel(Box::new(inner))
                    }
                    "Operation" | "operation" | "Op" | "op" => {
                        if self.match_token(&TokenKind::Less) {
                            let tin = self.parse_type()?;
                            let mut tout = None;
                            if self.match_token(&TokenKind::Comma) {
                                tout = Some(Box::new(self.parse_type()?));
                            }
                            self.expect(TokenKind::Greater)?;
                            Type::Operation(Some(Box::new(tin)), tout)
                        } else {
                            Type::Operation(None, None)
                        }
                    }
                    "OperationResult" => Type::OperationResult,
                    "Event" | "event" => {
                        if self.match_token(&TokenKind::Less) {
                            let ev_name = self.parse_identifier_or_keyword()?;
                            self.expect(TokenKind::Greater)?;
                            Type::Event(ev_name)
                        } else {
                            Type::Event("Any".into())
                        }
                    }
                    "region" => {
                        if self.match_token(&TokenKind::Less) {
                            let reg_name = self.parse_identifier_or_keyword()?;
                            self.expect(TokenKind::Greater)?;
                            Type::Region(reg_name)
                        } else {
                            Type::Region("default".into())
                        }
                    }
                    other => {
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
        }
    }

}
