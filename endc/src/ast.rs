use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
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
    Operation(Option<Box<Type>>, Option<Box<Type>>), // Operation<TIn, TOut>
    Event(String),                                   // Event type
    OperationResult,                                 // Rich OperationResult
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
            Type::Operation(tin, tout) => {
                write!(f, "Operation")?;
                if tin.is_some() || tout.is_some() {
                    write!(f, "<{}, {}>", tin.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "void".to_string()),
                                          tout.as_ref().map(|t| t.to_string()).unwrap_or_else(|| "void".to_string()))?;
                }
                Ok(())
            }
            Type::Event(name) => write!(f, "Event<{}>", name),
            Type::OperationResult => write!(f, "OperationResult"),
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StructDef {
    pub name: String,
    pub generic_params: Vec<String>,
    pub is_pub: bool,
    pub is_partial: bool,
    pub is_sealed: bool,
    pub is_extension_only: bool,
    pub is_open: bool,
    pub is_closed: bool,
    pub friend_modules: Vec<String>,
    pub extension_points: Vec<String>,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleFacets {
    pub api: Vec<FunctionDef>,
    pub implementation: Vec<FunctionDef>,
    pub tests: Vec<FunctionDef>,
    pub extension: Vec<FunctionDef>,
    pub architecture: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleContract {
    pub requires: Vec<String>,
    pub provides: Vec<String>,
    pub guarantees: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModuleDef {
    pub name: String,
    pub parent: Option<String>,
    pub is_pub: bool,
    pub is_partial: bool,
    pub is_evolvable: bool,
    pub responsibility: Option<String>,
    pub owns: Vec<String>,
    pub exposes: Vec<String>,
    pub depends: Vec<String>,
    pub depends_only: Option<Vec<String>>,
    pub forbid: Vec<String>,
    pub is_sealed: bool,
    pub purity: Option<String>,
    pub cohesion: Option<f64>,
    pub facets: Option<ModuleFacets>,
    pub contract: Option<ModuleContract>,
    pub overlay_target: Option<String>,
    pub skills: Vec<String>,
    pub structs: Vec<StructDef>,
    pub functions: Vec<FunctionDef>,
    pub overrides: Vec<FunctionDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

// ── 50 Super Revolutionary Feature-Oriented Paradigm AST Structures ──

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureApi {
    pub functions: Vec<FunctionDef>,
    pub structs: Vec<StructDef>,
    pub enums: Vec<EnumDef>,
    pub traits: Vec<TraitDef>,
    pub exposed_symbols: Vec<String>,
    pub raw_signatures: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureImpl {
    pub name: Option<String>,
    pub target_contract: Option<String>,
    pub functions: Vec<FunctionDef>,
    pub structs: Vec<StructDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureContractClause {
    pub rule: String,
    pub is_negative: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDependency {
    pub name: String,
    pub sub_contract: Option<String>, // e.g. "api" in "Authentication.api"
    pub type_params: Vec<String>,     // e.g. ["Transactional"]
    pub why: Option<String>,          // e.g. "Payment signatures require cryptographic verification"
    pub is_typed: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureExtensionPoint {
    pub name: String,
    pub allowed_types: Vec<String>,
    pub priority: Option<i64>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureBoundary {
    pub layers: Vec<String>, // ["api", "domain", "infrastructure"]
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeaturePermission {
    pub allow: Vec<String>,
    pub deny: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureLifecycle {
    pub state: String, // "experimental", "stable", "deprecated"
    pub replace_with: Option<String>,
    pub migration_path: Option<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDecision {
    pub target: String,
    pub reason: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureDef {
    pub name: String,
    pub version: Option<String>,
    pub owner: Option<String>,
    pub parent: Option<String>,
    pub architecture_template: Option<String>,
    pub is_pub: bool,
    pub is_replaceable: bool,
    pub is_evolvable: bool,
    pub api: Option<FeatureApi>,
    pub implementations: Vec<FeatureImpl>,
    pub needs: Vec<FeatureDependency>,
    pub boundary: Option<FeatureBoundary>,
    pub exposes: Vec<String>,
    pub extensions: Vec<FeatureExtensionPoint>,
    pub compose: Vec<String>,
    pub contracts: Vec<FeatureContractClause>,
    pub invariants: Vec<Expression>,
    pub tests: Vec<FunctionDef>,
    pub requires_capabilities: Vec<String>,
    pub permissions: Option<FeaturePermission>,
    pub lifecycle: Option<FeatureLifecycle>,
    pub decisions: Vec<FeatureDecision>,
    pub nested_features: Vec<FeatureDef>,
    pub forbids: Vec<String>,
    pub allows: Vec<String>,
    pub decorations: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ContractDef {
    pub name: String,
    pub methods: Vec<TraitMethodDef>,
    pub clauses: Vec<String>,
    pub is_evolved: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ArchitectureTemplateDef {
    pub name: String,
    pub required_layers: Vec<String>,
    pub rules: Vec<String>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ArchitectureRuleDef {
    pub name: String,
    pub allowed_flows: Vec<(String, String)>,
    pub forbidden_flows: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct FeatureMigrationDef {
    pub feature_name: String,
    pub from_version: String,
    pub to_version: String,
    pub renames: Vec<(String, String)>,
    pub replacements: Vec<(String, String)>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct BlastRadiusReport {
    pub target_symbol: String,
    pub affected_features: Vec<String>,
    pub affected_modules: Vec<String>,
    pub affected_symbols: Vec<String>,
    pub affected_public_apis: Vec<String>,
    pub required_migrations: Vec<String>,
}


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ExtensionBlock {
    pub target: String,
    pub is_struct: bool,
    pub is_augment: bool,
    pub trait_name: Option<String>,
    pub at_hook: Option<String>,
    pub required_capability: Option<String>,
    pub when_feature: Option<String>,
    pub generic_params: Vec<String>,
    pub version_req: Option<String>,
    pub owned_by: Option<String>,
    pub lifecycle: Option<String>,
    pub functions: Vec<FunctionDef>,
    pub overrides: Vec<FunctionDef>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OperationDef {
    pub name: String,
    pub params: Vec<FunctionParam>,
    pub return_type: Type,
    pub is_pub: bool,
    pub requires: Vec<String>,
    pub guarantees: Vec<String>,
    pub effects: Vec<String>,
    pub emits: Vec<String>,
    pub version: Option<usize>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventDef {
    pub name: String,
    pub is_pub: bool,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventHandlerDef {
    pub event_name: String,
    pub handler_op: Option<Expression>,
    pub body: Option<Block>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventHubDef {
    pub name: String,
    pub is_pub: bool,
    pub owns_events: Vec<String>,
    pub handlers: Vec<EventHandlerDef>,
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
    // Operation Values, Event-Native Architecture & Algebra Statements
    OperationDecl(OperationDef),
    EventDecl(EventDef),
    EventHubDecl(EventHubDef),
    EmitEvent {
        event_name: String,
        args: Vec<Expression>,
        span: Span,
    },
    ObserveOp {
        op_expr: Expression,
        alias: String,
        span: Span,
    },
    AnalyzeOp {
        op_expr: Expression,
        span: Span,
    },
    ExtractOpDecl {
        op_name: String,
        from_mod: String,
        condition: String,
        span: Span,
    },
    InlineOpDecl {
        op_name: String,
        span: Span,
    },
    SplitOpDecl {
        op_name: String,
        sub_ops: Vec<String>,
        span: Span,
    },
    MergeOpDecl {
        source_ops: Vec<String>,
        as_name: String,
        span: Span,
    },
    ExplainOpDecl {
        op_name: String,
        span: Span,
    },
    EvolveOpDecl {
        op_name: String,
        preserve: Vec<String>,
        optimize: Vec<String>,
        allow: Vec<String>,
        reject: Vec<String>,
        span: Span,
    },
    // Agent Contract System (Intent → Task → Skill → Evidence → Verify) Statements
    FeatureDecl {
        name: String,
        requirement: Option<String>,
        skills: Vec<String>,
        tasks: Vec<String>,
        span: Span,
    },
    SkillDecl {
        name: String,
        rules: Vec<String>,
        constraints: Vec<String>,
        structural: Vec<String>,
        semantic: Vec<String>,
        behavioral: Vec<String>,
        architectural: Vec<String>,
        performance: Vec<String>,
        security: Vec<String>,
        testing: Vec<String>,
        agent: Vec<String>,
        requires: Vec<String>,
        hard: Vec<String>,
        soft: Vec<String>,
        for_scope: Option<String>,
        span: Span,
    },
    SatisfiesDecl {
        entity: String,
        skills: Vec<String>,
        span: Span,
    },
    ProjectSkillsDecl {
        profile: std::collections::HashMap<String, String>,
        span: Span,
    },
    AgentTaskContractDecl {
        name: String,
        owner: Option<String>,
        status: Option<String>,
        requirement: Option<String>,
        implementation: Option<String>,
        skills: Vec<String>,
        change_budget: Vec<String>,
        evidence: Vec<(String, String)>,
        span: Span,
    },
    ClaimTask {
        task_name: String,
        span: Span,
    },
    CompleteTask {
        task_name: String,
        result: String,
        confidence: Option<f64>,
        summary: Option<String>,
        evidence: Vec<String>,
        risks: Option<String>,
        recommendation: Option<String>,
        notes: Option<String>,
        span: Span,
    },
    VerifyTask {
        target: String,
        is_adversarial: bool,
        skill: Option<String>,
        span: Span,
    },
    RequirementDecl {
        req_id: String,
        description: String,
        span: Span,
    },
    ImplementsDecl {
        req_id: String,
        entities: Vec<String>,
        span: Span,
    },
    VerifiesDecl {
        req_id: String,
        entities: Vec<String>,
        span: Span,
    },
    TodoDecl {
        id: String,
        implement: String,
        requires: Vec<String>,
        verify: Vec<String>,
        status: String,
        span: Span,
    },
    AgentBoundaryDecl {
        module_name: String,
        span: Span,
    },
    AgentContextDecl {
        module_name: String,
        expose: Vec<String>,
        hide: Vec<String>,
        span: Span,
    },
    ContextFirewallDecl {
        module_name: String,
        deny: Vec<String>,
        expose: Vec<String>,
        span: Span,
    },
    AgentApiDecl {
        module_name: String,
        expose: Vec<String>,
        hide: Vec<String>,
        span: Span,
    },
    AgentabilityDecl {
        max_context_tokens: usize,
        max_operation_complexity: String,
        max_dependency_fanout: usize,
        span: Span,
    },
    IntentDecl {
        goal: String,
        preserve: Vec<String>,
        optimize: Vec<String>,
        span: Span,
    },
    SemanticCommitDecl {
        task: String,
        intent: String,
        satisfies: Vec<String>,
        evidence: Vec<String>,
        span: Span,
    },
    AgentReviewDecl {
        task_id: String,
        summary: String,
        completed: usize,
        unresolved: usize,
        risks: usize,
        confidence: f64,
        span: Span,
    },
    ApprovalDecl {
        required_items: Vec<String>,
        span: Span,
    },
    AgentLeaseDecl {
        module_name: String,
        owner: String,
        duration: String,
        span: Span,
    },
    KnowledgeDecl {
        name: String,
        decisions: Vec<String>,
        constraints: Vec<String>,
        span: Span,
    },
    DecisionDecl {
        id: String,
        choose: String,
        because: String,
        reject: String,
        span: Span,
    },
    AgentCapabilityDecl {
        capabilities: Vec<String>,
        cannot: Vec<String>,
        span: Span,
    },
    RegressionGuardDecl {
        items: Vec<String>,
        span: Span,
    },

    // Layer 1 Extensibility DNA Statements
    PartialDecl {
        kind: String,
        name: String,
        body_struct: Option<StructDef>,
        body_module: Option<ModuleDef>,
        span: Span,
    },
    AugmentDecl(ExtensionBlock),
    OverrideDecl {
        target: String,
        method: FunctionDef,
        span: Span,
    },
    ExtensionPointDecl {
        target: String,
        points: Vec<String>,
        span: Span,
    },
    LayerSealedDecl {
        target_kind: String,
        target_name: String,
        span: Span,
    },
    LayerFriendDecl {
        target_kind: String,
        target_name: String,
        friend_name: String,
        span: Span,
    },

    // Layer 2 Super Module System
    ReplaceModuleDecl {
        target: String,
        replacement: String,
        span: Span,
    },
    ModuleMigrationDecl {
        module_name: String,
        from_version: usize,
        to_version: usize,
        renames: Vec<(String, String)>,
        span: Span,
    },
    ModuleFacadeDecl {
        name: String,
        methods: Vec<String>,
        span: Span,
    },
    ModuleOverlayDecl {
        name: String,
        target_env: String,
        body: Block,
        span: Span,
    },
    ModuleComposeDecl {
        modules: Vec<String>,
        span: Span,
    },

    // Layer 3 Type System for Extensibility
    OpenClosedTypeDecl {
        is_open: bool,
        name: String,
        span: Span,
    },
    ExtensionTraitDecl {
        trait_name: String,
        target_type: String,
        methods: Vec<FunctionDef>,
        span: Span,
    },
    ExtensionConflictDecl {
        target_type: String,
        resolutions: Vec<(String, String)>,
        span: Span,
    },

    // Layer 4 Syntax Extensibility
    SyntaxDecl {
        name: String,
        pattern: Option<String>,
        namespace: Option<String>,
        version: Option<usize>,
        params: Vec<FunctionParam>,
        return_type: Option<Type>,
        body: Option<Block>,
        span: Span,
    },
    UseSyntaxDecl {
        namespace: String,
        version: Option<usize>,
        span: Span,
    },

    // Layer 5 Compile-time Extensibility
    CompilerPluginDecl {
        name: String,
        kind: String,
        span: Span,
    },
    CustomLinterDecl {
        name: String,
        rules: Vec<String>,
        span: Span,
    },
    CustomAnalyzerDecl {
        name: String,
        checks: Vec<String>,
        span: Span,
    },
    CustomTypeRuleDecl {
        target_type: String,
        rules: Vec<String>,
        span: Span,
    },
    CustomOptimizerDecl {
        name: String,
        pass: String,
        span: Span,
    },
    BuildPluginDecl {
        name: String,
        hooks: Vec<String>,
        span: Span,
    },
    GeneratorDecl {
        name: String,
        target_format: String,
        span: Span,
    },
    ReflectDecl {
        target_type: String,
        queries: Vec<String>,
        span: Span,
    },

    // Layer 6 Architecture as Code
    ArchitectureContractDecl {
        name: String,
        rules: Vec<String>,
        span: Span,
    },
    ForbiddenDependencyDecl {
        from: String,
        to: String,
        span: Span,
    },
    AllowedDependencyDecl {
        from: String,
        to: String,
        span: Span,
    },
    ArchitectureBoundaryDecl {
        name: String,
        components: Vec<String>,
        span: Span,
    },
    ArchitectureOwnerDecl {
        target: String,
        owner: String,
        span: Span,
    },
    ArchitectureStabilityDecl {
        target: String,
        level: String,
        span: Span,
    },
    ArchitectureEvolutionDecl {
        module_name: String,
        from_v: String,
        to_v: String,
        span: Span,
    },
    ArchitectureTestDecl {
        assertions: Vec<String>,
        span: Span,
    },

    // Layer 7 Dependency Intelligence
    ChangeBudgetDecl {
        max_files: Option<usize>,
        max_modules: Option<usize>,
        public_api_allowed: Option<bool>,
        span: Span,
    },
    DependencyLockDecl {
        locked: bool,
        span: Span,
    },
    SemanticImportDecl {
        feature_intent: String,
        alias: Option<String>,
        span: Span,
    },

    // Layer 8 API Evolution
    ApiStabilityDecl {
        target: String,
        level: String,
        span: Span,
    },
    DeprecationDecl {
        target: String,
        replace_with: Option<String>,
        span: Span,
    },
    ApiSnapshotDecl {
        module_name: String,
        span: Span,
    },
    VerifyCompatibilityDecl {
        target1: String,
        target2: String,
        span: Span,
    },

    // Layer 9 Agent-Native Extensibility
    AgentExtensionContractDecl {
        name: String,
        purpose: String,
        inputs: Vec<String>,
        outputs: Vec<String>,
        constraints: Vec<String>,
        tests: Vec<String>,
        permissions: Vec<String>,
        span: Span,
    },
    AgentChangeProposalDecl {
        title: String,
        files: Vec<String>,
        symbols: Vec<String>,
        dependencies: Vec<String>,
        risks: Vec<String>,
        migration: Option<String>,
        span: Span,
    },
    AgentProofGateDecl {
        checks: Vec<String>,
        span: Span,
    },
    AgentTransactionDecl {
        action: String,
        body: Option<Block>,
        span: Span,
    },

    // Layer 10 Code Evolution Engine & @evolvable
    WhyMetadataDecl {
        why: String,
        reason: Option<String>,
        span: Span,
    },
    EvolvableDecl {
        module_name: String,
        metrics_target: Option<String>,
        span: Span,
    },

    // ── 50 Super Revolutionary Feature-Oriented Paradigm Statements ──
    FeatureStatement(FeatureDef),
    ContractDefinition(ContractDef),
    ArchitectureTemplate(ArchitectureTemplateDef),
    ArchitectureRuleStatement(ArchitectureRuleDef),
    FeatureMigrationStatement(FeatureMigrationDef),
    ReplaceFeature {
        target: String,
        with_provider: String,
        span: Span,
    },
    DecorateFeature {
        target: String,
        decorators: Vec<String>,
        span: Span,
    },
    ComposeFeature {
        target: String,
        components: Vec<String>,
        span: Span,
    },
    EvolveFeature {
        target: String,
        adds: Vec<String>,
        replaces: Vec<(String, String)>,
        body: Option<Block>,
        span: Span,
    },
    EvolveContract {
        target: String,
        adds: Vec<TraitMethodDef>,
        clauses: Vec<String>,
        span: Span,
    },
    ImpactQuery {
        target: String,
        span: Span,
    },
    UseFeature {
        feature: String,
        implementation: Option<String>,
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
            Expression::OperationLiteral { span, .. } => span,
            Expression::Compose { span, .. } => span,
            Expression::Repeat { span, .. } => span,
            Expression::Parallel { span, .. } => span,
            Expression::Alternative { span, .. } => span,
            Expression::ConditionalOp { span, .. } => span,
            Expression::Memoize { span, .. } => span,
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
            Statement::OperationDecl(op) => &op.span,
            Statement::EventDecl(ev) => &ev.span,
            Statement::EventHubDecl(hub) => &hub.span,
            Statement::EmitEvent { span, .. } => span,
            Statement::ObserveOp { span, .. } => span,
            Statement::AnalyzeOp { span, .. } => span,
            Statement::ExtractOpDecl { span, .. } => span,
            Statement::InlineOpDecl { span, .. } => span,
            Statement::SplitOpDecl { span, .. } => span,
            Statement::MergeOpDecl { span, .. } => span,
            Statement::ExplainOpDecl { span, .. } => span,
            Statement::EvolveOpDecl { span, .. } => span,
            Statement::FeatureDecl { span, .. } => span,
            Statement::SkillDecl { span, .. } => span,
            Statement::SatisfiesDecl { span, .. } => span,
            Statement::ProjectSkillsDecl { span, .. } => span,
            Statement::AgentTaskContractDecl { span, .. } => span,
            Statement::ClaimTask { span, .. } => span,
            Statement::CompleteTask { span, .. } => span,
            Statement::VerifyTask { span, .. } => span,
            Statement::RequirementDecl { span, .. } => span,
            Statement::ImplementsDecl { span, .. } => span,
            Statement::VerifiesDecl { span, .. } => span,
            Statement::TodoDecl { span, .. } => span,
            Statement::AgentBoundaryDecl { span, .. } => span,
            Statement::AgentContextDecl { span, .. } => span,
            Statement::ContextFirewallDecl { span, .. } => span,
            Statement::AgentApiDecl { span, .. } => span,
            Statement::AgentabilityDecl { span, .. } => span,
            Statement::IntentDecl { span, .. } => span,
            Statement::SemanticCommitDecl { span, .. } => span,
            Statement::AgentReviewDecl { span, .. } => span,
            Statement::ApprovalDecl { span, .. } => span,
            Statement::AgentLeaseDecl { span, .. } => span,
            Statement::KnowledgeDecl { span, .. } => span,
            Statement::DecisionDecl { span, .. } => span,
            Statement::AgentCapabilityDecl { span, .. } => span,
            Statement::RegressionGuardDecl { span, .. } => span,

            // Layer 1
            Statement::PartialDecl { span, .. } => span,
            Statement::AugmentDecl(aug) => &aug.span,
            Statement::OverrideDecl { span, .. } => span,
            Statement::ExtensionPointDecl { span, .. } => span,
            Statement::LayerSealedDecl { span, .. } => span,
            Statement::LayerFriendDecl { span, .. } => span,

            // Layer 2
            Statement::ReplaceModuleDecl { span, .. } => span,
            Statement::ModuleMigrationDecl { span, .. } => span,
            Statement::ModuleFacadeDecl { span, .. } => span,
            Statement::ModuleOverlayDecl { span, .. } => span,
            Statement::ModuleComposeDecl { span, .. } => span,

            // Layer 3
            Statement::OpenClosedTypeDecl { span, .. } => span,
            Statement::ExtensionTraitDecl { span, .. } => span,
            Statement::ExtensionConflictDecl { span, .. } => span,

            // Layer 4
            Statement::SyntaxDecl { span, .. } => span,
            Statement::UseSyntaxDecl { span, .. } => span,

            // Layer 5
            Statement::CompilerPluginDecl { span, .. } => span,
            Statement::CustomLinterDecl { span, .. } => span,
            Statement::CustomAnalyzerDecl { span, .. } => span,
            Statement::CustomTypeRuleDecl { span, .. } => span,
            Statement::CustomOptimizerDecl { span, .. } => span,
            Statement::BuildPluginDecl { span, .. } => span,
            Statement::GeneratorDecl { span, .. } => span,
            Statement::ReflectDecl { span, .. } => span,

            // Layer 6
            Statement::ArchitectureContractDecl { span, .. } => span,
            Statement::ForbiddenDependencyDecl { span, .. } => span,
            Statement::AllowedDependencyDecl { span, .. } => span,
            Statement::ArchitectureBoundaryDecl { span, .. } => span,
            Statement::ArchitectureOwnerDecl { span, .. } => span,
            Statement::ArchitectureStabilityDecl { span, .. } => span,
            Statement::ArchitectureEvolutionDecl { span, .. } => span,
            Statement::ArchitectureTestDecl { span, .. } => span,

            // Layer 7
            Statement::ChangeBudgetDecl { span, .. } => span,
            Statement::DependencyLockDecl { span, .. } => span,
            Statement::SemanticImportDecl { span, .. } => span,

            // Layer 8
            Statement::ApiStabilityDecl { span, .. } => span,
            Statement::DeprecationDecl { span, .. } => span,
            Statement::ApiSnapshotDecl { span, .. } => span,
            Statement::VerifyCompatibilityDecl { span, .. } => span,

            // Layer 9
            Statement::AgentExtensionContractDecl { span, .. } => span,
            Statement::AgentChangeProposalDecl { span, .. } => span,
            Statement::AgentProofGateDecl { span, .. } => span,
            Statement::AgentTransactionDecl { span, .. } => span,

            // Layer 10
            Statement::WhyMetadataDecl { span, .. } => span,
            Statement::EvolvableDecl { span, .. } => span,

            // Feature-Oriented Paradigms
            Statement::FeatureStatement(f) => &f.span,
            Statement::ContractDefinition(c) => &c.span,
            Statement::ArchitectureTemplate(a) => &a.span,
            Statement::ArchitectureRuleStatement(r) => &r.span,
            Statement::FeatureMigrationStatement(m) => &m.span,
            Statement::ReplaceFeature { span, .. } => span,
            Statement::DecorateFeature { span, .. } => span,
            Statement::ComposeFeature { span, .. } => span,
            Statement::EvolveFeature { span, .. } => span,
            Statement::EvolveContract { span, .. } => span,
            Statement::ImpactQuery { span, .. } => span,
            Statement::UseFeature { span, .. } => span,
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
    pub features: Vec<FeatureDef>,
    pub contracts: Vec<ContractDef>,
    pub architecture_templates: Vec<ArchitectureTemplateDef>,
    pub architecture_rules: Vec<ArchitectureRuleDef>,
    pub feature_migrations: Vec<FeatureMigrationDef>,
    pub statements: Vec<Statement>,
    pub span: Span,
}




