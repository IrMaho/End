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

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[allow(dead_code)]
pub struct Lexer<'a> {
    pub source: &'a str,
    pub chars: Vec<char>,
    pub cursor: usize,
    pub line: usize,
    pub col: usize,
    pub filename: String,
}

impl<'a> Lexer<'a> {
    pub fn new(filename: impl Into<String>, source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().collect(),
            cursor: 0,
            line: 1,
            col: 1,
            filename: filename.into(),
        }
    }

    fn peek(&self) -> Option<char> {
        if self.cursor < self.chars.len() {
            Some(self.chars[self.cursor])
        } else {
            None
        }
    }

    fn peek_next(&self) -> Option<char> {
        if self.cursor + 1 < self.chars.len() {
            Some(self.chars[self.cursor + 1])
        } else {
            None
        }
    }

    fn advance(&mut self) -> Option<char> {
        if self.cursor < self.chars.len() {
            let ch = self.chars[self.cursor];
            self.cursor += 1;
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            Some(ch)
        } else {
            None
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' | '\r' | '\n' | '\u{feff}' => {
                    self.advance();
                }
                '/' => {
                    if self.peek_next() == Some('/') {
                        // Line comment
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if self.peek_next() == Some('*') {
                        // Block comment
                        self.advance();
                        self.advance();
                        while let Some(c) = self.peek() {
                            if c == '*' && self.peek_next() == Some('/') {
                                self.advance();
                                self.advance();
                                break;
                            }
                            self.advance();
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    pub fn next_token(&mut self) -> Result<Token, String> {
        self.skip_whitespace_and_comments();

        let start_line = self.line;
        let start_col = self.col;
        let span = Span::new(&self.filename, start_line, start_col);

        let ch = match self.peek() {
            Some(c) => c,
            None => {
                return Ok(Token {
                    kind: TokenKind::EOF,
                    span,
                })
            }
        };

        // Identifiers or keywords or directive
        if ch == '@' {
            self.advance();
            let mut name = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    name.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(Token {
                kind: TokenKind::Directive(format!("@{}", name)),
                span,
            });
        }

        if ch == '_' && !self.peek_next().map_or(false, |c| c.is_alphanumeric() || c == '_') {
            self.advance();
            return Ok(Token {
                kind: TokenKind::Underscore,
                span,
            });
        }

        // Morphic Template Identifiers: e.g. {platform}_send or {target}_Client
        if ch == '{' && self.peek_next().map_or(false, |c| c.is_alphabetic() || c == '_') {
            self.advance(); // consume '{'
            let mut morphic_str = String::from("{");
            while let Some(c) = self.peek() {
                morphic_str.push(c);
                self.advance();
                if c == '}' {
                    break;
                }
            }
            // Capture suffix like '_send' or '_Client'
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    morphic_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }
            return Ok(Token {
                kind: TokenKind::MorphicIdent(morphic_str),
                span,
            });
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut ident = String::new();
            while let Some(c) = self.peek() {
                if c.is_alphanumeric() || c == '_' {
                    ident.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            let kind = if self.peek() == Some('!') && self.peek_next() != Some('=') {
                self.advance();
                match ident.as_str() {
                    "val" => TokenKind::ValBang,
                    "nameof" => TokenKind::NameOf,
                    "pathof" => TokenKind::PathOf,
                    "typeof" => TokenKind::TypeOf,
                    "docof" => TokenKind::DocOf,
                    "codeof" => TokenKind::CodeOf,
                    "dbg" => TokenKind::Dbg,
                    "assert_debug" => TokenKind::AssertDebug,
                    "t" => TokenKind::Translate,
                    "fields_of" => TokenKind::FieldsOf,
                    "sql_expr" => TokenKind::SqlExpr,
                    _ => TokenKind::Ident(format!("{}!", ident)),
                }
            } else {
                match ident.as_str() {
                    "st" | "struct" => TokenKind::Struct,
                    "enum" => TokenKind::Enum,
                    "fn" => TokenKind::Fn,
                    "val" => TokenKind::Val,
                    "mut" | "var" => TokenKind::Mut,
                    "ret" | "return" => TokenKind::Return,
                    "if" => TokenKind::If,
                    "else" => TokenKind::Else,
                    "while" => TokenKind::While,
                    "for" => TokenKind::For,
                    "parallel" => TokenKind::Parallel,
                    "in" => TokenKind::In,
                    "match" => TokenKind::Match,
                    "defer" => TokenKind::Defer,
                    "region" => TokenKind::Region,
                    "asm" => TokenKind::Asm,
                    "target" => TokenKind::Target,
                    "import" => TokenKind::Import,
                    "as" => TokenKind::As,
                    "pub" => TokenKind::Pub,
                    "alloc" => TokenKind::Alloc,
                    "catch" => TokenKind::Catch,
                    "null" => TokenKind::Null,
                    "true" => TokenKind::True,
                    "false" => TokenKind::False,
                    "spawn" => TokenKind::Spawn,
                    "skip" => TokenKind::Skip,
                    "trait" => TokenKind::Trait,
                    "impl" => TokenKind::Impl,
                    "async" => TokenKind::Async,
                    "await" => TokenKind::Await,
                    "mod" | "module" => TokenKind::Mod,
                    "derives" => TokenKind::Derives,
                    "override" => TokenKind::Override,
                    "extend" => TokenKind::Extend,
                    "bridge" => TokenKind::Bridge,
                    "with" => TokenKind::With,
                    "inline_c" => TokenKind::InlineC,
                    "extern" => TokenKind::Extern,
                    "lease" => TokenKind::Lease,
                    "borrow" => TokenKind::Borrow,
                    "during" => TokenKind::During,
                    "intent" => TokenKind::Intent,
                    "prove" => TokenKind::Prove,
                    "assume" => TokenKind::Assume,
                    "guarantee" => TokenKind::Guarantee,
                    "invariant" => TokenKind::Invariant,
                    "because" => TokenKind::Because,
                    "why" => TokenKind::Why,
                    "protect" => TokenKind::Protect,
                    "frozen" => TokenKind::Frozen,
                    "mutable_by" => TokenKind::MutableBy,
                    "owned" => TokenKind::Owned,
                    "handoff" => TokenKind::Handoff,
                    "return_to" => TokenKind::ReturnTo,
                    "compute" => TokenKind::Compute,
                    "race_free" => TokenKind::RaceFree,
                    "order" => TokenKind::Order,
                    "deterministic" => TokenKind::Deterministic,
                    "replay" | "replayable" => TokenKind::Replay,
                    "checkpoint" => TokenKind::Checkpoint,
                    "rollback" => TokenKind::Rollback,
                    "transaction" => TokenKind::Transaction,
                    "speculative" => TokenKind::Speculative,
                    "fallback" => TokenKind::Fallback,
                    "budget" => TokenKind::Budget,
                    "deadline" => TokenKind::Deadline,
                    "priority" => TokenKind::Priority,
                    "quality" => TokenKind::Quality,
                    "tradeoff" => TokenKind::Tradeoff,
                    "adapt" => TokenKind::Adapt,
                    "observe" => TokenKind::Observe,
                    "watch" => TokenKind::Watch,
                    "react" => TokenKind::React,
                    "stream" => TokenKind::Stream,
                    "flow" => TokenKind::Flow,
                    "choose" => TokenKind::Choose,
                    "race" => TokenKind::Race,
                    "hedge" => TokenKind::Hedge,
                    "cancel_safe" => TokenKind::CancelSafe,
                    "agent" => TokenKind::Agent,
                    "task" => TokenKind::Task,
                    "accept" => TokenKind::Accept,
                    "reject" => TokenKind::Reject,
                    "baseline" => TokenKind::Baseline,
                    "regression" => TokenKind::Regression,
                    "explain" => TokenKind::Explain,
                    "context" => TokenKind::Context,
                    "slice" => TokenKind::Slice,
                    "patch" => TokenKind::Patch,
                    "evolve" => TokenKind::Evolve,
                    "verify" => TokenKind::Verify,
                    "goal" => TokenKind::Goal,
                    "preserve" => TokenKind::Preserve,
                    "allow" => TokenKind::Allow,
                    "to" => TokenKind::To,
                    "on" => TokenKind::On,
                    "mutate" => TokenKind::MutateToken,
                    "boundary" => TokenKind::Boundary,
                    "responsibility" => TokenKind::Responsibility,
                    "owns" => TokenKind::Owns,
                    "exposes" => TokenKind::Exposes,
                    "depends_only" => TokenKind::DependsOnly,
                    "depends" => TokenKind::Depends,
                    "forbid" => TokenKind::Forbid,
                    "layer" => TokenKind::Layer,
                    "direction" => TokenKind::Direction,
                    "split" => TokenKind::Split,
                    "partition" => TokenKind::Partition,
                    "extract" => TokenKind::Extract,
                    "cluster" => TokenKind::Cluster,
                    "separate" => TokenKind::Separate,
                    "contract" => TokenKind::Contract,
                    "port" => TokenKind::Port,
                    "adapter" => TokenKind::Adapter,
                    "facade" => TokenKind::Facade,
                    "gateway" => TokenKind::Gateway,
                    "compat" => TokenKind::Compat,
                    "stable" => TokenKind::Stable,
                    "sealed" => TokenKind::Sealed,
                    "friend" => TokenKind::Friend,
                    "private_to" => TokenKind::PrivateTo,
                    "surface" => TokenKind::Surface,
                    "leak" => TokenKind::Leak,
                    "purity" => TokenKind::Purity,
                    "view" => TokenKind::View,
                    "lens" => TokenKind::Lens,
                    "agent_scope" | "scope" => TokenKind::AgentScope,
                    "budget_context" => TokenKind::BudgetContext,
                    "token_budget" => TokenKind::TokenBudget,
                    "move" => TokenKind::Move,
                    "migrate" => TokenKind::Migrate,
                    "redirect" => TokenKind::Redirect,
                    "deprecate" | "deprecate_after" => TokenKind::Deprecate,
                    "cycle_free" => TokenKind::CycleFree,
                    "max_fanout" | "fanout" => TokenKind::MaxFanout,
                    "max_fanin" | "fan_in" | "fanin" => TokenKind::MaxFanin,
                    "max_dependency_depth" | "depth" => TokenKind::MaxDepth,
                    "cohesion" => TokenKind::Cohesion,
                    "modularize" => TokenKind::Modularize,
                    "decompose" => TokenKind::Decompose,
                    "architecture" => TokenKind::Architecture,
                    "repair" => TokenKind::Repair,
                    "gravity" => TokenKind::Gravity,
                    "deny" => TokenKind::Deny,
                    "into" => TokenKind::Into,
                    "from" => TokenKind::From,
                    "toward" => TokenKind::Toward,
                    "optimize" => TokenKind::Optimize,
                    "reject_if" => TokenKind::RejectIf,
                    "never" => TokenKind::Never,
                    "after" => TokenKind::After,
                    "remove" => TokenKind::Remove,
                    "hide" => TokenKind::Hide,
                    "focus" => TokenKind::Focus,
                    "by" => TokenKind::By,
                    "through" => TokenKind::Through,
                    "operation" => TokenKind::Operation,
                    "event" => TokenKind::Event,
                    "hub" => TokenKind::Hub,
                    "emit" => TokenKind::Emit,
                    "compose" => TokenKind::Compose,
                    "retry" => TokenKind::Retry,
                    "repeat" => TokenKind::Repeat,
                    "when" => TokenKind::When,
                    "subscribes" => TokenKind::Subscribes,
                    "analyze" => TokenKind::Analyze,
                    "memoize" => TokenKind::Memoize,
                    "equivalent" => TokenKind::Equivalent,
                    "merge" => TokenKind::Merge,
                    "inline" => TokenKind::Inline,
                    "then" => TokenKind::Then,
                    "requires" | "require" => TokenKind::Requires,
                    "guarantees" => TokenKind::Guarantee,
                    "emits" => TokenKind::Emit,
                    "effects" | "effect" => TokenKind::Effects,
                    "version" => TokenKind::Version,
                    "feature" => TokenKind::Feature,
                    "skill" => TokenKind::Skill,
                    "skills" => TokenKind::Skills,
                    "satisfies" => TokenKind::Satisfies,
                    "rules" | "rule" => TokenKind::Rules,
                    "constraints" | "constraint" => TokenKind::Constraints,
                    "requirement" | "requirements" => TokenKind::Requirement,
                    "implements" => TokenKind::Implements,
                    "verifies" => TokenKind::Verifies,
                    "claim" => TokenKind::Claim,
                    "complete" => TokenKind::Complete,
                    "evidence" => TokenKind::Evidence,
                    "todo" => TokenKind::Todo,
                    "knowledge" => TokenKind::Knowledge,
                    "decision" => TokenKind::Decision,
                    "approval" => TokenKind::Approval,
                    "review" => TokenKind::Review,
                    "review_by" => TokenKind::ReviewBy,
                    "confidence" => TokenKind::Confidence,
                    "change" => TokenKind::Change,
                    "agent_boundary" => TokenKind::AgentBoundary,
                    "agent_context" => TokenKind::AgentContext,
                    "context_firewall" => TokenKind::ContextFirewall,
                    "agent_api" => TokenKind::AgentApi,
                    "agentability" => TokenKind::Agentability,
                    "regression_guard" => TokenKind::RegressionGuard,
                    "adversarial" => TokenKind::Adversarial,
                    "tasks" => TokenKind::Tasks,
                    "profile" => TokenKind::Profile,
                    "proof" => TokenKind::Prove,
                    "hard" => TokenKind::Hard,
                    "soft" => TokenKind::Soft,
                    "structural" => TokenKind::Structural,
                    "semantic" => TokenKind::Semantic,
                    "behavioral" => TokenKind::Behavioral,
                    "performance" => TokenKind::Performance,
                    "security" => TokenKind::Security,
                    "testing" => TokenKind::Testing,
                    "summary" => TokenKind::Summary,
                    "risks" => TokenKind::Risks,
                    "recommendation" => TokenKind::Recommendation,
                    "notes" => TokenKind::Notes,

                    // Extensibility DNA Keywords
                    "partial" => TokenKind::Partial,
                    "augment" => TokenKind::Augment,
                    "extension_only" => TokenKind::ExtensionOnly,
                    "extension_point" => TokenKind::ExtensionPoint,
                    "replace" => TokenKind::Replace,
                    "migration" => TokenKind::Migration,
                    "overlay" => TokenKind::Overlay,
                    "open" => TokenKind::Open,
                    "closed" => TokenKind::Closed,
                    "syntax" => TokenKind::Syntax,
                    "compiler_plugin" => TokenKind::CompilerPlugin,
                    "lint" => TokenKind::Lint,
                    "analyzer" => TokenKind::Analyzer,
                    "type_rule" => TokenKind::TypeRule,
                    "optimizer" => TokenKind::Optimizer,
                    "build_plugin" => TokenKind::BuildPlugin,
                    "generator" => TokenKind::Generator,
                    "reflect" => TokenKind::Reflect,
                    "lock" => TokenKind::Lock,
                    "agent_extension" => TokenKind::AgentExtension,
                    "proposal" => TokenKind::Proposal,
                    "evolvable" => TokenKind::Evolvable,
                    "owned_by" => TokenKind::OwnedBy,
                    "architecture_test" => TokenKind::ArchitectureTest,
                    "at" => TokenKind::At,
                    "provides" => TokenKind::Provides,
                    "rename" => TokenKind::Rename,
                    "use" => TokenKind::Use,
                    "snapshot" => TokenKind::Snapshot,
                    "replace_with" => TokenKind::ReplaceWith,
                    "api" => TokenKind::Api,
                    "needs" => TokenKind::Needs,
                    "expose" => TokenKind::Expose,
                    "replaceable" => TokenKind::Replaceable,
                    "lifecycle" => TokenKind::Lifecycle,
                    "decorate" => TokenKind::Decorate,
                    "impact" => TokenKind::Impact,
                    "must" => TokenKind::Must,
                    "reason" => TokenKind::Reason,
                    "internal" => TokenKind::Internal,
                    "private" => TokenKind::Private,
                    "public" => TokenKind::Pub,
                    "extends" => TokenKind::Extends,
                    "extension" | "extensions" => TokenKind::Extension,
                    "implementation" => TokenKind::Implementation,
                    "test" => TokenKind::Test,
                    "begin" => TokenKind::Begin,
                    "commit" => TokenKind::Commit,
                    "not" => TokenKind::Not,

                    _ => TokenKind::Ident(ident),
                }
            };

            return Ok(Token { kind, span });
        }

        // Numbers (integers, hex, binary, or floats)
        if ch.is_ascii_digit() {
            let mut num_str = String::new();
            let mut is_float = false;

            if ch == '0' && (self.peek_next() == Some('x') || self.peek_next() == Some('X')) {
                self.advance(); // consume '0'
                self.advance(); // consume 'x'
                let mut hex_str = String::new();
                while let Some(c) = self.peek() {
                    if c.is_ascii_hexdigit() {
                        hex_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&hex_str, 16)
                    .or_else(|_| u64::from_str_radix(&hex_str, 16).map(|u| u as i64))
                    .map_err(|e| format!("Invalid hex literal 0x{}: {}", hex_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            if ch == '0' && (self.peek_next() == Some('b') || self.peek_next() == Some('B')) {
                self.advance(); // consume '0'
                self.advance(); // consume 'b'
                let mut bin_str = String::new();
                while let Some(c) = self.peek() {
                    if c == '0' || c == '1' {
                        bin_str.push(c);
                        self.advance();
                    } else {
                        break;
                    }
                }
                let val = i64::from_str_radix(&bin_str, 2)
                    .map_err(|e| format!("Invalid binary literal 0b{}: {}", bin_str, e))?;
                return Ok(Token { kind: TokenKind::IntLit(val), span });
            }

            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.advance();
                } else if c == '.' && self.peek_next().map_or(false, |next| next.is_ascii_digit()) {
                    is_float = true;
                    num_str.push(c);
                    self.advance();
                } else {
                    break;
                }
            }

            if self.peek() == Some('[') {
                self.advance(); // consume '['
                let mut unit_str = String::new();
                while let Some(c) = self.peek() {
                    if c == ']' {
                        self.advance(); // consume ']'
                        break;
                    } else {
                        unit_str.push(c);
                        self.advance();
                    }
                }
                let val_f: f64 = num_str.parse().unwrap_or(0.0);
                return Ok(Token {
                    kind: TokenKind::UnitLit(val_f, unit_str),
                    span,
                });
            }

            let kind = if is_float {
                let val: f64 = num_str.parse().map_err(|e| format!("Invalid float: {}", e))?;
                TokenKind::FloatLit(val)
            } else {
                let val: i64 = num_str.parse().map_err(|e| format!("Invalid integer: {}", e))?;
                TokenKind::IntLit(val)
            };

            return Ok(Token { kind, span });
        }

        // String literals
        if ch == '"' {
            self.advance(); // consume opening quote
            let mut s = String::new();
            while let Some(c) = self.peek() {
                if c == '"' {
                    self.advance(); // consume closing quote
                    return Ok(Token {
                        kind: TokenKind::StringLit(s),
                        span,
                    });
                } else if c == '\\' {
                    self.advance();
                    match self.advance() {
                        Some('n') => s.push('\n'),
                        Some('r') => s.push('\r'),
                        Some('t') => s.push('\t'),
                        Some('\\') => s.push('\\'),
                        Some('"') => s.push('"'),
                        Some(other) => s.push(other),
                        None => return Err("Unexpected EOF in escape string sequence".into()),
                    }
                } else {
                    s.push(c);
                    self.advance();
                }
            }
            return Err("Unterminated string literal".into());
        }

        // Operators & Single character tokens
        self.advance();
        let kind = match ch {
            '+' => TokenKind::Plus,
            '-' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Arrow
                } else {
                    TokenKind::Minus
                }
            }
            '*' => TokenKind::Star,
            '/' => TokenKind::Slash,
            '%' => TokenKind::Percent,
            '=' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::EqualEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::FatArrow
                } else {
                    TokenKind::Equal
                }
            }
            '!' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::BangEqual
                } else if self.peek() == Some('-') && self.peek_next() == Some('>') {
                    self.advance(); // consume '-'
                    self.advance(); // consume '>'
                    TokenKind::BangArrow
                } else {
                    TokenKind::Bang
                }
            }
            '<' => {
                if self.peek() == Some('+') && self.peek_next() == Some('=') {
                    self.advance(); // consume '+'
                    self.advance(); // consume '='
                    TokenKind::LessPlusEqual
                } else if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::LessEqual
                } else if self.peek() == Some('<') {
                    self.advance();
                    TokenKind::Shl
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.peek() == Some('=') {
                    self.advance();
                    TokenKind::GreaterEqual
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::Shr
                } else {
                    TokenKind::Greater
                }
            }
            ':' => TokenKind::Colon,
            ';' => TokenKind::SemiColon,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            '&' => {
                if self.peek() == Some('&') {
                    self.advance();
                    TokenKind::AmpAmp
                } else {
                    TokenKind::Ampersand
                }
            }
            '|' => {
                if self.peek() == Some('|') {
                    self.advance();
                    TokenKind::PipePipe
                } else if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::PipeGreater
                } else {
                    TokenKind::Pipe
                }
            }
            '^' => TokenKind::Caret,
            '~' => {
                if self.peek() == Some('>') {
                    self.advance();
                    TokenKind::TildeArrow
                } else {
                    TokenKind::Tilde
                }
            }
            '?' => {
                if self.peek() == Some('?') {
                    self.advance();
                    TokenKind::QuestionQuestion
                } else {
                    TokenKind::Question
                }
            }
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            '{' => TokenKind::LBrace,
            '}' => TokenKind::RBrace,
            '[' => TokenKind::LBracket,
            ']' => TokenKind::RBracket,
            other => return Err(format!("Unexpected character: '{}' at line {}, col {}", other, start_line, start_col)),
        };

        Ok(Token { kind, span })
    }

    pub fn tokenize_all(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();
        loop {
            let tok = self.next_token()?;
            let is_eof = tok.kind == TokenKind::EOF;
            tokens.push(tok);
            if is_eof {
                break;
            }
        }
        Ok(tokens)
    }
}
