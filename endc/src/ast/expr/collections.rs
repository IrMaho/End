use serde::{Deserialize, Serialize};
use crate::ast::expr::Expression;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CollectionElement {
    Expr(Expression),
    Spread {
        expr: Expression,
        is_null_aware: bool,
    },
    If {
        condition: Expression,
        element: Box<CollectionElement>,
        else_element: Option<Box<CollectionElement>>,
    },
    For {
        item: String,
        iterable: Expression,
        element: Box<CollectionElement>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Literal(String),
    Expr(Expression),
}
