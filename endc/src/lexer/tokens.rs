use serde::{Deserialize, Serialize};
use crate::ast::Span;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Struct,   // 'st' or 'struct'
    Enum,     // 'enum'
    Fn,       // 'fn'
    Val,      // 'val' (immutable)
    Mut,      // 'mut' (mutable)
    Return,   // 'ret' or 'return'
    If,       // 'if'
    Else,     // 'else'
    While,    // 'while'
    For,      // 'for'
    Parallel, // 'parallel'
    In,       // 'in'
    Match,    // 'match'
    Defer,    // 'defer'
    Region,   // 'region'
    Asm,      // 'asm'
    Target,   // 'target'
    Import,   // 'import'
    As,       // 'as'
    Pub,      // 'pub'
    Alloc,    // 'alloc'
    Catch,    // 'catch'
    Null,     // 'null'
    True,     // 'true'
    False,    // 'false'
    Spawn,    // 'spawn'
    Skip,     // 'skip'
    Trait,    // 'trait'
    Impl,     // 'impl'
    Async,    // 'async'
    Await,    // 'await'
    Mod,      // 'mod' or 'module'
    Derives,  // 'derives'
    Override, // 'override'
    Extend,   // 'extend'
    Bridge,   // 'bridge'
    With,     // 'with'
    InlineC,  // 'inline_c'
    Extern,   // 'extern'
    Lease,    // 'lease'
    Borrow,   // 'borrow'
    During,   // 'during'

    // 50 Next-Gen & AI-Native Semantics Tokens
    Intent,       // 'intent'
    Prove,        // 'prove'
    Assume,       // 'assume'
    Guarantee,    // 'guarantee'
    Invariant,    // 'invariant'
    Because,      // 'because'
    Why,          // 'why'
    Protect,      // 'protect'
    Frozen,       // 'frozen'
    MutableBy,    // 'mutable_by'
    Owned,        // 'owned'
    Handoff,      // 'handoff'
    ReturnTo,     // 'return_to'
    Compute,      // 'compute'
    RaceFree,     // 'race_free'
    Order,        // 'order'
    Deterministic,// 'deterministic'
    Replay,       // 'replay'
    Checkpoint,   // 'checkpoint'
    Rollback,     // 'rollback'
    Transaction,  // 'transaction'
    Speculative,  // 'speculative'
    Fallback,     // 'fallback'
    Budget,       // 'budget'
    Deadline,     // 'deadline'
    Priority,     // 'priority'
    Quality,      // 'quality'
    Tradeoff,     // 'tradeoff'
    Adapt,        // 'adapt'
    Observe,      // 'observe'
    Watch,        // 'watch'
    React,        // 'react'
    Stream,       // 'stream'
    Flow,         // 'flow'
    Choose,       // 'choose'
    Race,         // 'race'
    Hedge,        // 'hedge'
    CancelSafe,   // 'cancel_safe'
    Agent,        // 'agent'
    Task,         // 'task'
    Accept,       // 'accept'
    Reject,       // 'reject'
    Baseline,     // 'baseline'
    Regression,   // 'regression'
    Explain,      // 'explain'
    Context,      // 'context'
    Slice,        // 'slice'
    Patch,        // 'patch'
    Evolve,       // 'evolve'
    Verify,       // 'verify'
    Goal,         // 'goal'
    Preserve,     // 'preserve'
    Allow,        // 'allow'
    To,           // 'to'
    On,           // 'on'
    MutateToken,  // 'mutate'

    // Agent-Modular Architectural Tokens
    Boundary,         // 'boundary'
    Responsibility,   // 'responsibility'
    Owns,             // 'owns'
    Exposes,          // 'exposes'
    DependsOnly,      // 'depends_only'
    Depends,          // 'depends'
    Forbid,           // 'forbid'
    Layer,            // 'layer'
    Direction,        // 'direction'
    Split,            // 'split'
    Partition,        // 'partition'
    Extract,          // 'extract'
    Cluster,          // 'cluster'
    Separate,         // 'separate'
    Contract,         // 'contract'
    Port,             // 'port'
    Adapter,          // 'adapter'
    Facade,           // 'facade'
    Gateway,          // 'gateway'
    Compat,           // 'compat'
    Stable,           // 'stable'
    Sealed,           // 'sealed'
    Friend,           // 'friend'
    PrivateTo,        // 'private_to'
    Surface,          // 'surface'
    Leak,             // 'leak'
    Purity,           // 'purity'
    View,             // 'view'
    Lens,             // 'lens'
    AgentScope,       // 'agent_scope' or 'scope'
    BudgetContext,    // 'budget_context'
    TokenBudget,      // 'token_budget'
    Move,             // 'move'
    Migrate,          // 'migrate'
    Redirect,         // 'redirect'
    Deprecate,        // 'deprecate' or 'deprecate_after'
    CycleFree,        // 'cycle_free'
    MaxFanout,        // 'max_fanout' or 'fanout'
    MaxFanin,         // 'max_fanin' or 'fan_in' or 'fanin'
    MaxDepth,         // 'max_dependency_depth' or 'depth'
    Cohesion,         // 'cohesion'
    Modularize,       // 'modularize'
    Decompose,        // 'decompose'
    Architecture,     // 'architecture'
    Repair,           // 'repair'
    Gravity,          // 'gravity'
    Deny,             // 'deny'
    Into,             // 'into'
    From,             // 'from'
    Toward,           // 'toward'
    Optimize,         // 'optimize'
    RejectIf,         // 'reject_if'
    Never,            // 'never'
    After,            // 'after'
    Remove,           // 'remove'
    Hide,             // 'hide'
    Focus,            // 'focus'
    By,               // 'by'
    Through,          // 'through'

    // Operation Values & Operation Algebra Tokens
    Operation,        // 'operation' or 'op'
    Event,            // 'event'
    Hub,              // 'hub'
    Emit,             // 'emit'
    Compose,          // 'compose'
    Retry,            // 'retry'
    Repeat,           // 'repeat'
    When,             // 'when'
    Subscribes,       // 'subscribes'
    Analyze,          // 'analyze'
    Memoize,          // 'memoize'
    Equivalent,       // 'equivalent'
    Merge,            // 'merge'
    Inline,           // 'inline'
    Then,             // 'then'
    Requires,         // 'requires'
    Effects,          // 'effects'
    Version,          // 'version'

    // Agent Contract System Tokens
    Feature,          // 'feature'
    Skill,            // 'skill'
    Skills,           // 'skills'
    Satisfies,        // 'satisfies'
    Rules,            // 'rules'
    Constraints,      // 'constraints'
    Requirement,      // 'requirement' or 'requirements'
    Implements,       // 'implements'
    Verifies,         // 'verifies'
    Claim,            // 'claim'
    Complete,         // 'complete'
    Evidence,         // 'evidence'
    Todo,             // 'todo'
    Knowledge,        // 'knowledge'
    Decision,         // 'decision'
    Approval,         // 'approval'
    Review,           // 'review'
    ReviewBy,         // 'review_by'
    Confidence,       // 'confidence'
    Change,           // 'change'
    AgentBoundary,    // 'agent_boundary'
    AgentContext,     // 'agent_context'
    ContextFirewall,  // 'context_firewall'
    AgentApi,         // 'agent_api'
    Agentability,     // 'agentability'
    RegressionGuard,  // 'regression_guard'
    Adversarial,      // 'adversarial'
    Tasks,            // 'tasks'
    Profile,          // 'profile'
    Hard,             // 'hard'
    Soft,             // 'soft'
    Structural,       // 'structural'
    Semantic,         // 'semantic'
    Behavioral,       // 'behavioral'
    Performance,      // 'performance'
    Security,         // 'security'
    Testing,          // 'testing'
    Summary,          // 'summary'
    Risks,            // 'risks'
    Recommendation,   // 'recommendation'
    Notes,            // 'notes'

    // Extensibility DNA Tokens
    Partial,          // 'partial'
    Augment,          // 'augment'
    ExtensionOnly,    // 'extension_only'
    ExtensionPoint,   // 'extension_point'
    Replace,          // 'replace'
    Migration,        // 'migration'
    Overlay,          // 'overlay'
    Open,             // 'open'
    Closed,           // 'closed'
    Syntax,           // 'syntax'
    CompilerPlugin,   // 'compiler_plugin'
    Lint,             // 'lint'
    Analyzer,         // 'analyzer'
    TypeRule,         // 'type_rule'
    Optimizer,        // 'optimizer'
    BuildPlugin,      // 'build_plugin'
    Generator,        // 'generator'
    Reflect,          // 'reflect'
    Lock,             // 'lock'
    AgentExtension,   // 'agent_extension'
    Proposal,         // 'proposal'
    Evolvable,        // 'evolvable'
    OwnedBy,          // 'owned_by'
    ArchitectureTest, // 'architecture_test'
    At,               // 'at'
    Provides,         // 'provides'
    Guarantees,       // 'guarantees'
    Rename,           // 'rename'
    Use,              // 'use'
    Snapshot,         // 'snapshot'
    ReplaceWith,      // 'replace_with'
    Api,              // 'api'
    Needs,            // 'needs'
    Expose,           // 'expose'
    Replaceable,      // 'replaceable'
    Lifecycle,        // 'lifecycle'
    Decorate,         // 'decorate'
    Impact,           // 'impact'
    Must,             // 'must'
    Reason,           // 'reason'
    Internal,         // 'internal'
    Private,          // 'private'
    Extends,          // 'extends'
    Extension,        // 'extension'
    Implementation,   // 'implementation'
    Test,             // 'test'
    BangArrow,        // '!->'
    Begin,            // 'begin'
    Commit,           // 'commit'
    Not,              // 'not'


    // Revolutionary Syntactic Tokens
    ValBang,          // 'val!'
    QuestionQuestion, // '??'
    Question,         // '?'
    TildeArrow,       // '~>'
    LessPlusEqual,    // '<+='
    UnitLit(f64, String), // e.g. 120[km/h]
    MorphicIdent(String), // e.g. '{platform}_send'

    // Meta-Syntax & Reflection Macros
    NameOf,       // 'nameof!'
    PathOf,       // 'pathof!'
    TypeOf,       // 'typeof!'
    DocOf,        // 'docof!'
    CodeOf,       // 'codeof!'
    Dbg,          // 'dbg!'
    AssertDebug,  // 'assert_debug!'
    Translate,    // 't!'
    FieldsOf,     // 'fields_of!'
    SqlExpr,      // 'sql_expr!'

    // Directives
    Directive(String), // '@agent_note', '@target', '@c', etc.

    // Literals & Identifiers
    Ident(String),
    IntLit(i64),
    FloatLit(f64),
    StringLit(String),

    // Symbols & Operators
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    Bang,
    BangEqual,
    Less,
    LessEqual,
    Shl,       // '<<'
    Greater,
    GreaterEqual,
    Shr,       // '>>'
    Arrow,     // '->'
    FatArrow,  // '=>'
    Colon,     // ':'
    SemiColon, // ';'
    Comma,     // ','
    Dot,       // '.'
    Ampersand, // '&'
    AmpAmp,    // '&&'
    Pipe,      // '|'
    PipePipe,  // '||'
    PipeGreater, // '|>'
    Caret,     // '^'
    Tilde,     // '~'
    Underscore,// '_'
    LParen,    // '('
    RParen,    // ')'
    LBrace,    // '{'
    RBrace,    // '}'
    LBracket,  // '['
    RBracket,  // ']'

    // Modern Expressive Operators
    ColonEqual,           // ':='
    DotDot,               // '..'
    DotDotDot,            // '...'
    DotDotLess,           // '..<'
    QuestionDot,          // '?.'
    QuestionDotDot,       // '?..'
    QuestionQuestionEqual,// '??='
    DotDotDotQuestion,    // '...?'
    StarStar,             // '**'
    Is,                   // 'is'

    // Capability & Surface Composition Tokens
    Access,
    Grant,
    Adopt,
    Implement,
    Attach,
    Detach,
    Mixin,
    Capability,
    Provide,
    Require,
    Resolve,
    Select,
    Project,
    Delegate,
    Proxy,
    Intercept,
    Hook,
    Enable,
    Disable,
    Scope,
    FeatureSwitch,
    Traitify,
    Equip,
    Fuse,
    Shape,
    Only,
    Section,
    Before,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}
