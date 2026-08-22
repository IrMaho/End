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
    pub morphic_param: Option<String>,
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
pub struct ModuleDef {
    pub name: String,
    pub parent: Option<String>,
    pub is_pub: bool,
    pub responsibility: Option<String>,
    pub owns: Vec<String>,
    pub exposes: Vec<String>,
    pub depends: Vec<String>,
    pub depends_only: Option<Vec<String>>,
    pub forbid: Vec<String>,
    pub is_sealed: bool,
    pub purity: Option<String>,
    pub cohesion: Option<f64>,
    pub structs: Vec<StructDef>,
    pub functions: Vec<FunctionDef>,
    pub overrides: Vec<FunctionDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExtensionBlock {
    pub target: String,
    pub is_struct: bool,
    pub functions: Vec<FunctionDef>,
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
        is_lease: bool,
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
    Skip {
        span: Span,
    },
    InlineC {
        code: String,
        span: Span,
    },
    LeaseBlock {
        name: String,
        var_type: Option<Type>,
        initializer: Expression,
        condition: Option<Expression>,
        body: Block,
        span: Span,
    },
    LeaseCpu {
        cores: Expression,
        priority: Option<Expression>,
        body: Block,
        span: Span,
    },
    LeaseEvent {
        event_expr: Expression,
        condition: Option<Expression>,
        body: Block,
        span: Span,
    },
    LeaseLoop {
        budget: Option<Expression>,
        item_name: String,
        iterable: Expression,
        body: Block,
        span: Span,
    },
    QuantumUnwrap {
        name: String,
        var_type: Option<Type>,
        expr: Expression,
        fallback: Expression,
        span: Span,
    },
    AtomicOp {
        target: String,
        op: BinaryOp,
        value: Expression,
        span: Span,
    },

    // ── 50 Next-Gen & AI-Native Semantics AST Variants ──
    Intent {
        name: Option<String>,
        goal: String,
        preserve: Vec<String>,
        body: Option<Block>,
        span: Span,
    },
    Prove {
        condition: Expression,
        span: Span,
    },
    Assume {
        condition: Expression,
        span: Span,
    },
    Guarantee {
        condition: Expression,
        span: Span,
    },
    Invariant {
        condition: Expression,
        span: Span,
    },
    Because {
        rationale: String,
        span: Span,
    },
    Why {
        target: String,
        rationale: String,
        span: Span,
    },
    IntentDiff {
        preserve: Vec<String>,
        change: Vec<String>,
        span: Span,
    },
    ProtectBlock {
        body: Block,
        span: Span,
    },
    Frozen {
        symbol: String,
        span: Span,
    },
    MutableBy {
        roles: Vec<String>,
        span: Span,
    },
    Owned {
        name: String,
        var_type: Option<Type>,
        initializer: Expression,
        span: Span,
    },
    Handoff {
        resource: String,
        target_domain: String,
        span: Span,
    },
    ReturnTo {
        source_domain: String,
        resource: String,
        span: Span,
    },
    ComputeBlock {
        target: String,
        body: Block,
        fallback: Option<Block>,
        span: Span,
    },
    RaceFreeBlock {
        body: Block,
        span: Span,
    },
    Order {
        mode: String,
        span: Span,
    },
    DeterministicBlock {
        body: Block,
        span: Span,
    },
    ReplayBlock {
        body: Block,
        span: Span,
    },
    Checkpoint {
        state_name: String,
        span: Span,
    },
    Rollback {
        checkpoint_name: String,
        span: Span,
    },
    TransactionBlock {
        body: Block,
        span: Span,
    },
    SpeculativeBlock {
        body: Block,
        span: Span,
    },
    FallbackBlock {
        target: String,
        body: Block,
        span: Span,
    },
    BudgetBlock {
        specs: Vec<(String, String)>,
        body: Option<Block>,
        span: Span,
    },
    DeadlineBlock {
        duration: String,
        body: Block,
        span: Span,
    },
    PriorityBlock {
        level: String,
        body: Block,
        span: Span,
    },
    QualityBlock {
        min_metric: String,
        max_latency: String,
        body: Block,
        span: Span,
    },
    TradeoffBlock {
        prefer: String,
        sacrifice: String,
        body: Block,
        span: Span,
    },
    AdaptBlock {
        branches: Vec<(Expression, Block)>,
        span: Span,
    },
    Observe {
        metrics: Vec<String>,
        span: Span,
    },
    WatchBlock {
        target: String,
        event: String,
        handler: Block,
        span: Span,
    },
    ReactBlock {
        event: Expression,
        handler: Block,
        span: Span,
    },
    StreamBlock {
        source: Expression,
        operations: Vec<Expression>,
        span: Span,
    },
    FlowBlock {
        steps: Vec<Expression>,
        span: Span,
    },
    ParallelChoose {
        branches: Vec<(String, Block)>,
        span: Span,
    },
    RaceBlock {
        branches: Vec<Block>,
        span: Span,
    },
    HedgeBlock {
        delay_ms: Expression,
        primary: Block,
        fallback: Block,
        span: Span,
    },
    CancelSafeBlock {
        body: Block,
        span: Span,
    },
    AgentContract {
        name: String,
        scope: String,
        goal: String,
        constraints: Vec<String>,
        body: Option<Block>,
        span: Span,
    },
    TaskDecl {
        name: String,
        body: Block,
        span: Span,
    },
    AcceptBlock {
        conditions: Vec<String>,
        span: Span,
    },
    RejectBlock {
        conditions: Vec<String>,
        span: Span,
    },
    BaselineBlock {
        metrics: Vec<(String, String)>,
        span: Span,
    },
    RegressionCheck {
        condition: String,
        span: Span,
    },
    ExplainBlock {
        topic: String,
        rationale: String,
        span: Span,
    },
    ContextBlock {
        name: String,
        includes: Vec<String>,
        excludes: Vec<String>,
        body: Option<Block>,
        span: Span,
    },
    SliceDecl {
        name: String,
        from_target: String,
        includes: Vec<String>,
        excludes: Vec<String>,
        span: Span,
    },
    PatchDecl {
        target: String,
        body: Block,
        span: Span,
    },
    EvolveBlock {
        target: String,
        intent: String,
        preserve: Vec<String>,
        budget: Option<String>,
        allow: Vec<String>,
        reject: Vec<String>,
        verify: Vec<String>,
        accept: Vec<String>,
        body: Option<Block>,
        span: Span,
    },
    VerifyBlock {
        invariants: Vec<Expression>,
        span: Span,
    },
    // Family 1 & 2: Architectural Units & Boundaries & Dependencies
    BoundaryDecl {
        name: String,
        allows: Vec<String>,
        denies: Vec<String>,
        is_sealed: bool,
        span: Span,
    },
    ResponsibilityDecl {
        module_name: String,
        description: String,
        span: Span,
    },
    OwnsDecl {
        module_name: String,
        symbols: Vec<String>,
        span: Span,
    },
    ExposesDecl {
        module_name: String,
        symbols: Vec<String>,
        span: Span,
    },
    DependsDecl {
        from_module: String,
        target_module: String,
        is_only: bool,
        span: Span,
    },
    ForbidDecl {
        from: String,
        to: String,
        span: Span,
    },
    LayerDecl {
        name: String,
        forbid_depends: Vec<String>,
        span: Span,
    },
    DirectionDecl {
        from: String,
        to: String,
        span: Span,
    },
    // Family 3: Agent-Native Decomposition
    SplitDecl {
        entity: String,
        parts: Vec<String>,
        span: Span,
    },
    PartitionDecl {
        entity: String,
        by: String,
        parts: Vec<String>,
        span: Span,
    },
    ExtractDecl {
        symbols: Vec<String>,
        into_module: String,
        span: Span,
    },
    ClusterDecl {
        by: String,
        predicate: String,
        span: Span,
    },
    SeparateDecl {
        left: String,
        right: String,
        span: Span,
    },
    // Family 4: Dependency Intelligence & Contracts
    ModuleContractDecl {
        module_name: String,
        accepts: Vec<String>,
        returns: Vec<String>,
        guarantees: Vec<String>,
        span: Span,
    },
    PortDecl {
        name: String,
        methods: Vec<String>,
        span: Span,
    },
    AdapterDecl {
        name: String,
        port: String,
        body: Block,
        span: Span,
    },
    FacadeDecl {
        name: String,
        exposes: Vec<String>,
        span: Span,
    },
    GatewayDecl {
        from_mod: String,
        to_mod: String,
        allowed_calls: Vec<String>,
        span: Span,
    },
    // Family 5: Architectural Invariants & Stability
    ArchInvariantDecl {
        rule: String,
        span: Span,
    },
    PreserveRefactorDecl {
        preserves: Vec<String>,
        body: Block,
        span: Span,
    },
    CompatDecl {
        module_name: String,
        version: String,
        body: Block,
        span: Span,
    },
    StableDecl {
        api_name: String,
        span: Span,
    },
    SealedDecl {
        boundary_name: String,
        span: Span,
    },
    // Family 6: Dependency Firewall & Visibility
    FriendDecl {
        module_name: String,
        friend_module: String,
        span: Span,
    },
    PrivateToDecl {
        symbol: String,
        module_name: String,
        span: Span,
    },
    SurfaceDecl {
        name: String,
        exposes: Vec<String>,
        hides: Vec<String>,
        span: Span,
    },
    LeakCheckDecl {
        module_name: String,
        symbol: String,
        through: String,
        span: Span,
    },
    PurityDecl {
        module_name: String,
        level: String,
        span: Span,
    },
    // Family 7: Agent Context Architecture
    ViewDecl {
        name: String,
        includes: Vec<String>,
        span: Span,
    },
    LensDecl {
        name: String,
        focus: String,
        hide: String,
        span: Span,
    },
    AgentScopeDecl {
        name: String,
        modules: Vec<String>,
        forbid: Vec<String>,
        span: Span,
    },
    BudgetContextDecl {
        name: String,
        token_budget: usize,
        priority: Vec<String>,
        span: Span,
    },
    // Family 8: Safe Refactoring
    MoveDecl {
        symbol: String,
        from_mod: String,
        to_mod: String,
        span: Span,
    },
    MigrateDecl {
        entity: String,
        from_mod: String,
        to_mod: String,
        span: Span,
    },
    BridgeDecl {
        from_mod: String,
        to_mod: String,
        via: String,
        span: Span,
    },
    RedirectDecl {
        from_api: String,
        to_api: String,
        span: Span,
    },
    DeprecateDecl {
        target_api: String,
        after_cond: String,
        remove_cond: String,
        span: Span,
    },
    // Family 9: Anti-Spaghetti Metrics
    CycleFreeDecl {
        scope: String,
        span: Span,
    },
    FanoutDecl {
        module_name: String,
        limit: usize,
        span: Span,
    },
    FaninDecl {
        module_name: String,
        limit: usize,
        span: Span,
    },
    DepthDecl {
        limit: usize,
        span: Span,
    },
    CohesionDecl {
        module_name: String,
        min_threshold: f64,
        span: Span,
    },
    // Family 10: Automated Modularization & Self-Evolution
    ModularizeDecl {
        target: String,
        target_files_min: usize,
        target_files_max: usize,
        preserve: Vec<String>,
        span: Span,
    },
    DecomposeDecl {
        target: String,
        target_modules: Option<usize>,
        optimize: Vec<String>,
        preserve: Vec<String>,
        gravity: Option<String>,
        span: Span,
    },
    ArchitectureDecl {
        name: String,
        layers: Vec<String>,
        rules: Vec<String>,
        directions: Vec<(String, String)>,
        invariants: Vec<String>,
        cycle_free: bool,
        max_depth: Option<usize>,
        span: Span,
    },
    RepairDecl {
        target: String,
        span: Span,
    },
    EvolveArchDecl {
        from: String,
        toward: String,
        target_modules: usize,
        preserve: Vec<String>,
        optimize: Vec<String>,
        reject_if: Vec<String>,
        verify: Vec<String>,
        span: Span,
    },
    GravityDecl {
        weights: Vec<(String, f64)>,
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
    BitNot,
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
            Expression::Pipe { span, .. } => span,
            Expression::InlineC { span, .. } => span,
            Expression::SqlExpr { span, .. } => span,
            Expression::Cast { span, .. } => span,
            Expression::Await { span, .. } => span,
            Expression::UnitLit { span, .. } => span,
            Expression::NullCollapse { span, .. } => span,
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
            Statement::Skip { span, .. } => span,
            Statement::InlineC { span, .. } => span,
            Statement::LeaseBlock { span, .. } => span,
            Statement::LeaseCpu { span, .. } => span,
            Statement::LeaseEvent { span, .. } => span,
            Statement::LeaseLoop { span, .. } => span,
            Statement::QuantumUnwrap { span, .. } => span,
            Statement::AtomicOp { span, .. } => span,
            Statement::Intent { span, .. } => span,
            Statement::Prove { span, .. } => span,
            Statement::Assume { span, .. } => span,
            Statement::Guarantee { span, .. } => span,
            Statement::Invariant { span, .. } => span,
            Statement::Because { span, .. } => span,
            Statement::Why { span, .. } => span,
            Statement::IntentDiff { span, .. } => span,
            Statement::ProtectBlock { span, .. } => span,
            Statement::Frozen { span, .. } => span,
            Statement::MutableBy { span, .. } => span,
            Statement::Owned { span, .. } => span,
            Statement::Handoff { span, .. } => span,
            Statement::ReturnTo { span, .. } => span,
            Statement::ComputeBlock { span, .. } => span,
            Statement::RaceFreeBlock { span, .. } => span,
            Statement::Order { span, .. } => span,
            Statement::DeterministicBlock { span, .. } => span,
            Statement::ReplayBlock { span, .. } => span,
            Statement::Checkpoint { span, .. } => span,
            Statement::Rollback { span, .. } => span,
            Statement::TransactionBlock { span, .. } => span,
            Statement::SpeculativeBlock { span, .. } => span,
            Statement::FallbackBlock { span, .. } => span,
            Statement::BudgetBlock { span, .. } => span,
            Statement::DeadlineBlock { span, .. } => span,
            Statement::PriorityBlock { span, .. } => span,
            Statement::QualityBlock { span, .. } => span,
            Statement::TradeoffBlock { span, .. } => span,
            Statement::AdaptBlock { span, .. } => span,
            Statement::Observe { span, .. } => span,
            Statement::WatchBlock { span, .. } => span,
            Statement::ReactBlock { span, .. } => span,
            Statement::StreamBlock { span, .. } => span,
            Statement::FlowBlock { span, .. } => span,
            Statement::ParallelChoose { span, .. } => span,
            Statement::RaceBlock { span, .. } => span,
            Statement::HedgeBlock { span, .. } => span,
            Statement::CancelSafeBlock { span, .. } => span,
            Statement::AgentContract { span, .. } => span,
            Statement::TaskDecl { span, .. } => span,
            Statement::AcceptBlock { span, .. } => span,
            Statement::RejectBlock { span, .. } => span,
            Statement::BaselineBlock { span, .. } => span,
            Statement::RegressionCheck { span, .. } => span,
            Statement::ExplainBlock { span, .. } => span,
            Statement::ContextBlock { span, .. } => span,
            Statement::SliceDecl { span, .. } => span,
            Statement::PatchDecl { span, .. } => span,
            Statement::EvolveBlock { span, .. } => span,
            Statement::VerifyBlock { span, .. } => span,
            Statement::BoundaryDecl { span, .. } => span,
            Statement::ResponsibilityDecl { span, .. } => span,
            Statement::OwnsDecl { span, .. } => span,
            Statement::ExposesDecl { span, .. } => span,
            Statement::DependsDecl { span, .. } => span,
            Statement::ForbidDecl { span, .. } => span,
            Statement::LayerDecl { span, .. } => span,
            Statement::DirectionDecl { span, .. } => span,
            Statement::SplitDecl { span, .. } => span,
            Statement::PartitionDecl { span, .. } => span,
            Statement::ExtractDecl { span, .. } => span,
            Statement::ClusterDecl { span, .. } => span,
            Statement::SeparateDecl { span, .. } => span,
            Statement::ModuleContractDecl { span, .. } => span,
            Statement::PortDecl { span, .. } => span,
            Statement::AdapterDecl { span, .. } => span,
            Statement::FacadeDecl { span, .. } => span,
            Statement::GatewayDecl { span, .. } => span,
            Statement::ArchInvariantDecl { span, .. } => span,
            Statement::PreserveRefactorDecl { span, .. } => span,
            Statement::CompatDecl { span, .. } => span,
            Statement::StableDecl { span, .. } => span,
            Statement::SealedDecl { span, .. } => span,
            Statement::FriendDecl { span, .. } => span,
            Statement::PrivateToDecl { span, .. } => span,
            Statement::SurfaceDecl { span, .. } => span,
            Statement::LeakCheckDecl { span, .. } => span,
            Statement::PurityDecl { span, .. } => span,
            Statement::ViewDecl { span, .. } => span,
            Statement::LensDecl { span, .. } => span,
            Statement::AgentScopeDecl { span, .. } => span,
            Statement::BudgetContextDecl { span, .. } => span,
            Statement::MoveDecl { span, .. } => span,
            Statement::MigrateDecl { span, .. } => span,
            Statement::BridgeDecl { span, .. } => span,
            Statement::RedirectDecl { span, .. } => span,
            Statement::DeprecateDecl { span, .. } => span,
            Statement::CycleFreeDecl { span, .. } => span,
            Statement::FanoutDecl { span, .. } => span,
            Statement::FaninDecl { span, .. } => span,
            Statement::DepthDecl { span, .. } => span,
            Statement::CohesionDecl { span, .. } => span,
            Statement::ModularizeDecl { span, .. } => span,
            Statement::DecomposeDecl { span, .. } => span,
            Statement::ArchitectureDecl { span, .. } => span,
            Statement::RepairDecl { span, .. } => span,
            Statement::EvolveArchDecl { span, .. } => span,
            Statement::GravityDecl { span, .. } => span,
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
    pub modules: Vec<ModuleDef>,
    pub extensions: Vec<ExtensionBlock>,
    pub statements: Vec<Statement>,
    pub span: Span,
}



