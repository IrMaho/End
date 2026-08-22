use serde::{Deserialize, Serialize};
use crate::ast::span::Span;
use crate::ast::types::Type;
use crate::ast::operators::{BinaryOp, Literal, UnaryOp};
use crate::ast::pattern::{Block, MatchArm, Pattern};
use crate::ast::decl::functions_traits::FunctionParam;
use crate::ast::stmt::Statement;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Lit(Literal, Span),
    Ident(String, Span),
    Binary {
        left: Box<Expression>,
        op: BinaryOp,
        right: Box<Expression>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expression>,
        span: Span,
    },
    Call {
        callee: Box<Expression>,
        args: Vec<Expression>,
        span: Span,
    },
    FieldAccess {
        object: Box<Expression>,
        field: String,
        span: Span,
    },
    Index {
        array: Box<Expression>,
        index: Box<Expression>,
        span: Span,
    },
    StructInit {
        name: String,
        fields: Vec<(String, Expression)>,
        span: Span,
    },
    EnumInit {
        enum_name: Option<String>,
        variant_name: String,
        payload: Option<Box<Expression>>,
        span: Span,
    },
    Alloc {
        allocator: Box<Expression>,
        target_type: Type,
        span: Span,
    },
    Promote {
        expr: Box<Expression>,
        target_region: String,
        span: Span,
    },
    Catch {
        expr: Box<Expression>,
        error_name: String,
        handler: Box<Statement>,
        span: Span,
    },
    Match {
        expr: Box<Expression>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Block(Block),
    NameOf {
        target: String,
        span: Span,
    },
    PathOf {
        target: String,
        span: Span,
    },
    TypeOf {
        expr: Box<Expression>,
        span: Span,
    },
    DocOf {
        target: String,
        span: Span,
    },
    CodeOf {
        expr: Box<Expression>,
        code: String,
        span: Span,
    },
    Dbg {
        expr: Box<Expression>,
        code: String,
        span: Span,
    },
    AssertDebug {
        condition: Box<Expression>,
        code: String,
        span: Span,
    },
    Translate {
        key: String,
        args: Vec<(String, Expression)>,
        span: Span,
    },
    FieldsOf {
        target: String,
        span: Span,
    },
    SqlExpr {
        expr: Box<Expression>,
        span: Span,
    },
    Cast {
        expr: Box<Expression>,
        target_type: Type,
        span: Span,
    },
    Await {
        expr: Box<Expression>,
        span: Span,
    },
    Pipe {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
        span: Span,
    },
    InlineC {
        code: String,
        span: Span,
    },
    UnitLit {
        value: f64,
        unit: String,
        span: Span,
    },
    NullCollapse {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    // Operation Values & Operation Algebra Expressions
    OperationLiteral {
        name: Option<String>,
        params: Vec<FunctionParam>,
        return_type: Type,
        requires: Vec<String>,
        guarantees: Vec<String>,
        effects: Vec<String>,
        emits: Vec<String>,
        body: Block,
        span: Span,
    },
    Compose {
        ops: Vec<Expression>,
        span: Span,
    },
    Repeat {
        op: Box<Expression>,
        count: Box<Expression>,
        is_retry: bool,
        span: Span,
    },
    Parallel {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    Alternative {
        left: Box<Expression>,
        right: Box<Expression>,
        span: Span,
    },
    ConditionalOp {
        op: Box<Expression>,
        condition: Box<Expression>,
        span: Span,
    },
    Memoize {
        op: Box<Expression>,
        span: Span,
    },
    // Modern Expressive Syntax Expressions
    Tuple(Vec<Expression>, Span),
    ListLiteral(Vec<crate::ast::expr::collections::CollectionElement>, Span),
    ListComprehension {
        expr: Box<Expression>,
        var: String,
        iterable: Box<Expression>,
        condition: Option<Box<Expression>>,
        span: Span,
    },
    DictComprehension {
        key: Box<Expression>,
        value: Box<Expression>,
        key_var: String,
        val_var: Option<String>,
        iterable: Box<Expression>,
        condition: Option<Box<Expression>>,
        span: Span,
    },
    SetComprehension {
        expr: Box<Expression>,
        var: String,
        iterable: Box<Expression>,
        condition: Option<Box<Expression>>,
        span: Span,
    },
    Cascade {
        target: Box<Expression>,
        operations: Vec<Expression>,
        is_null_aware: bool,
        span: Span,
    },
    Spread {
        expr: Box<Expression>,
        is_null_aware: bool,
        span: Span,
    },
    Range {
        start: Box<Expression>,
        end: Box<Expression>,
        inclusive: bool,
        span: Span,
    },
    Walrus {
        name: String,
        expr: Box<Expression>,
        span: Span,
    },
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Box<Expression>,
        span: Span,
    },
    Lambda {
        params: Vec<FunctionParam>,
        body: Box<Expression>,
        is_implicit: bool,
        span: Span,
    },
    NamedArg {
        name: String,
        value: Box<Expression>,
        span: Span,
    },
    CopyExpr {
        target: Box<Expression>,
        overrides: Vec<(String, Expression)>,
        span: Span,
    },
    ResultBuilder {
        name: String,
        body: Block,
        span: Span,
    },
    IsPattern {
        expr: Box<Expression>,
        pattern: Pattern,
        span: Span,
    },
    InterpolatedString {
        parts: Vec<crate::ast::expr::collections::StringPart>,
        span: Span,
    },
}

