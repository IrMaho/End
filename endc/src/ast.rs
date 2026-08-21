use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub line: usize,
    pub col: usize,
    pub file: String,
}

impl Span {
    pub fn new(file: impl Into<String>, line: usize, col: usize) -> Self {
        Self {
            file: file.into(),
            line,
            col,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Void,
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Str,
    Custom(String),
    Pointer(Box<Type>),
    Slice(Box<Type>),
    Array(Box<Type>, usize),
    Simd(Box<Type>, usize), // e.g. f32x4, i32x8
    Tuple(Vec<Type>),
    Generic(String, Vec<Type>),
    Result(Box<Type>, Option<Box<Type>>), // Result<T, E> or !T
    Region(String),                       // Region reference
    Box(Box<Type>),                       // Heap Box<T> (Tier 2)
    Rc(Box<Type>),                        // Reference Counted Rc<T> (Tier 3)
    Arc(Box<Type>),                       // Atomic Ref Counted Arc<T> (Tier 3)
    Channel(Box<Type>),                   // MPSC Channel<T>
    Allocator,
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Void => write!(f, "void"),
            Type::Bool => write!(f, "bool"),
            Type::I8 => write!(f, "i8"),
            Type::I16 => write!(f, "i16"),
            Type::I32 => write!(f, "i32"),
            Type::I64 => write!(f, "i64"),
            Type::U8 => write!(f, "u8"),
            Type::U16 => write!(f, "u16"),
            Type::U32 => write!(f, "u32"),
            Type::U64 => write!(f, "u64"),
            Type::F32 => write!(f, "f32"),
            Type::F64 => write!(f, "f64"),
            Type::Str => write!(f, "str"),
            Type::Custom(name) => write!(f, "{}", name),
            Type::Pointer(inner) => write!(f, "*{}", inner),
            Type::Slice(inner) => write!(f, "[]{}", inner),
            Type::Array(inner, size) => write!(f, "[{}]{}", size, inner),
            Type::Simd(inner, lanes) => write!(f, "{}x{}", inner, lanes),
            Type::Tuple(types) => {
                write!(f, "(")?;
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", t)?;
                }
                write!(f, ")")
            }
            Type::Generic(name, params) => {
                write!(f, "{}<", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p)?;
                }
                write!(f, ">")
            }
            Type::Result(inner, None) => write!(f, "!{}", inner),
            Type::Result(inner, Some(err)) => write!(f, "Result<{}, {}>", inner, err),
            Type::Region(name) => write!(f, "region<{}>", name),
            Type::Box(inner) => write!(f, "Box<{}>", inner),
            Type::Rc(inner) => write!(f, "Rc<{}>", inner),
            Type::Arc(inner) => write!(f, "Arc<{}>", inner),
            Type::Channel(inner) => write!(f, "Channel<{}>", inner),
            Type::Allocator => write!(f, "Allocator"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub args: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub field_type: Type,
    pub is_pub: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub fields: Vec<StructField>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub payload: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EnumDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub variants: Vec<EnumVariant>,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionParam {
    pub name: String,
    pub param_type: Type,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub body: Block,
    pub directives: Vec<Directive>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitMethodDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraitDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub methods: Vec<TraitMethodDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImplBlock {
    pub trait_name: Option<String>,
    pub target_type: Type,
    pub methods: Vec<FunctionDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ImportStmt {
    pub kind: ImportKind,
    pub path: String,
    pub alias: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ImportKind {
    Standard,
    C(String),
    Zig(String),
    Rust(String),
    Go(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Variant {
        enum_name: Option<String>,
        variant_name: String,
        binding: Option<String>,
    },
    Literal(Literal),
    Ident(String),
    Wildcard,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expression>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    VarDecl {
        name: String,
        var_type: Option<Type>,
        is_mut: bool,
        initializer: Option<Expression>,
        span: Span,
    },
    Assignment {
        target: Expression,
        value: Expression,
        span: Span,
    },
    Return {
        value: Option<Expression>,
        span: Span,
    },
    Expression(Expression),
    If {
        condition: Expression,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    While {
        condition: Expression,
        body: Block,
        span: Span,
    },
    ForIn {
        item_name: String,
        iterable: Expression,
        body: Block,
        span: Span,
    },
    ParallelFor {
        item_name: String,
        iterable: Expression,
        body: Block,
        span: Span,
    },
    Match {
        expr: Expression,
        arms: Vec<MatchArm>,
        span: Span,
    },
    RegionBlock {
        name: String,
        body: Block,
        span: Span,
    },
    AsmBlock {
        arch: String,
        code: String,
        span: Span,
    },
    TargetBlock {
        target: String,
        body: Block,
        span: Span,
    },
    Defer {
        expr: Expression,
        span: Span,
    },
    Spawn {
        call: Expression,
        span: Span,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    LessThan,
    LessEqual,
    GreaterThan,
    GreaterEqual,
    And,
    Or,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
    Not,
    AddressOf,
    Deref,
}

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
}

impl Expression {
    pub fn span(&self) -> &Span {
        match self {
            Expression::Lit(_, s) => s,
            Expression::Ident(_, s) => s,
            Expression::Binary { span, .. } => span,
            Expression::Unary { span, .. } => span,
            Expression::Call { span, .. } => span,
            Expression::FieldAccess { span, .. } => span,
            Expression::Index { span, .. } => span,
            Expression::StructInit { span, .. } => span,
            Expression::EnumInit { span, .. } => span,
            Expression::Alloc { span, .. } => span,
            Expression::Promote { span, .. } => span,
            Expression::Catch { span, .. } => span,
            Expression::Match { span, .. } => span,
            Expression::Block(b) => &b.span,
            Expression::NameOf { span, .. } => span,
            Expression::PathOf { span, .. } => span,
            Expression::TypeOf { span, .. } => span,
            Expression::DocOf { span, .. } => span,
            Expression::CodeOf { span, .. } => span,
            Expression::Dbg { span, .. } => span,
            Expression::AssertDebug { span, .. } => span,
            Expression::Translate { span, .. } => span,
            Expression::FieldsOf { span, .. } => span,
            Expression::SqlExpr { span, .. } => span,
        }
    }
}

impl Statement {
    pub fn span(&self) -> &Span {
        match self {
            Statement::VarDecl { span, .. } => span,
            Statement::Assignment { span, .. } => span,
            Statement::Return { span, .. } => span,
            Statement::Expression(expr) => expr.span(),
            Statement::If { span, .. } => span,
            Statement::While { span, .. } => span,
            Statement::ForIn { span, .. } => span,
            Statement::ParallelFor { span, .. } => span,
            Statement::Match { span, .. } => span,
            Statement::RegionBlock { span, .. } => span,
            Statement::AsmBlock { span, .. } => span,
            Statement::TargetBlock { span, .. } => span,
            Statement::Defer { span, .. } => span,
            Statement::Spawn { span, .. } => span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub imports: Vec<ImportStmt>,
    pub enums: Vec<EnumDef>,
    pub structs: Vec<StructDef>,
    pub traits: Vec<TraitDef>,
    pub impls: Vec<ImplBlock>,
    pub functions: Vec<FunctionDef>,
    pub span: Span,
}
