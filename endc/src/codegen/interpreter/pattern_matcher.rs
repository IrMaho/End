use super::state::Interpreter;
use super::value::Value;
use crate::ast::{Literal, Pattern};

impl Interpreter {
    pub(crate) fn matches_pattern(&self, target: &Value, pattern: &Pattern) -> Option<Vec<(String, Value)>> {
        match pattern {
            Pattern::Wildcard => Some(Vec::new()),
            Pattern::Ident(id) => Some(vec![(id.clone(), target.clone())]),
            Pattern::Literal(lit) => {
                let lit_val = match lit {
                    Literal::Int(n) => Value::Int(*n),
                    Literal::Float(f) => Value::Float(*f),
                    Literal::String(s) => Value::String(s.clone()),
                    Literal::Bool(b) => Value::Bool(*b),
                    Literal::Null => Value::Pointer(0),
                };
                if *target == lit_val {
                    Some(Vec::new())
                } else {
                    None
                }
            }
            Pattern::Variant { variant_name, binding, .. } => {
                if let Value::Enum(_, vname, payload) = target {
                    if vname == variant_name {
                        let mut out = Vec::new();
                        if let (Some(b), Some(p)) = (binding, payload) {
                            out.push((b.clone(), *p.clone()));
                        }
                        return Some(out);
                    }
                }
                None
            }
            Pattern::Binding(id) => Some(vec![(id.clone(), target.clone())]),
            Pattern::Tuple(_) | Pattern::Struct { .. } => Some(Vec::new()),
        }
    }
}
