// ? End Mid-Level Intermediate Representation (MIR) & Control Flow Graph (CFG)
// SSA Basic Blocks, Register Places, Explicit Drops, and Region Invariants

use crate::ir::hir::HirType;
use serde::{Deserialize, Serialize};

pub type BlockId = usize;
pub type RegId = usize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirModule {
    pub name: String,
    pub functions: Vec<MirFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirFunction {
    pub name: String,
    pub blocks: Vec<MirBasicBlock>,
    pub return_type: HirType,
    pub local_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirBasicBlock {
    pub id: BlockId,
    pub instructions: Vec<MirInstruction>,
    pub terminator: MirTerminator,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MirInstruction {
    Assign {
        dest: RegId,
        rvalue: MirRvalue,
    },
    Store {
        dest_addr: RegId,
        src: RegId,
    },
    Load {
        dest: RegId,
        src_addr: RegId,
    },
    RegionEnter {
        region_id: String,
    },
    RegionExit {
        region_id: String,
    },
    Drop {
        reg: RegId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MirRvalue {
    Use(RegId),
    ConstantInt(i64),
    ConstantFloat(f64),
    ConstantStr(String),
    ConstantBool(bool),
    BinaryOp(String, RegId, RegId),
    UnaryOp(String, RegId),
    Call(String, Vec<RegId>),
    Alloc(HirType, RegId, Option<String>),
    AddressOf(RegId),
    Deref(RegId),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MirTerminator {
    Return(Option<RegId>),
    Goto(BlockId),
    Branch {
        cond: RegId,
        then_block: BlockId,
        else_block: BlockId,
    },
    Panic(String),
}
