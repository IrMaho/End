// ? HIR -> MIR Control Flow Graph (CFG) Lowering Pass

use crate::ir::hir::*;
use crate::ir::mir::*;

pub struct HirToMirLowering {
    current_reg: RegId,
    current_block: BlockId,
    blocks: Vec<MirBasicBlock>,
}

impl HirToMirLowering {
    pub fn new() -> Self {
        Self {
            current_reg: 1,
            current_block: 0,
            blocks: Vec::new(),
        }
    }

    fn next_reg(&mut self) -> RegId {
        let r = self.current_reg;
        self.current_reg += 1;
        r
    }

    fn next_block(&mut self) -> BlockId {
        let id = self.blocks.len();
        self.blocks.push(MirBasicBlock {
            id,
            instructions: Vec::new(),
            terminator: MirTerminator::Return(None),
        });
        id
    }

    pub fn lower_module(hir: &HirModule) -> MirModule {
        let mut functions = Vec::new();
        for f in &hir.functions {
            let mut lowering = Self::new();
            let mir_fn = lowering.lower_function(f);
            functions.push(mir_fn);
        }
        MirModule {
            name: hir.name.clone(),
            functions,
        }
    }

    fn lower_function(&mut self, f: &HirFunction) -> MirFunction {
        self.current_reg = 1;
        self.blocks.clear();

        let entry_id = self.next_block();
        self.current_block = entry_id;

        for stmt in &f.body {
            self.lower_statement(stmt);
        }

        MirFunction {
            name: f.name.clone(),
            blocks: self.blocks.clone(),
            return_type: f.return_type.clone(),
            local_count: self.current_reg,
        }
    }

    fn lower_statement(&mut self, stmt: &HirStatement) {
        match stmt {
            HirStatement::VarDecl { init, .. } => {
                let dest_reg = self.next_reg();
                if let Some(in_expr) = init {
                    let rval = self.lower_expression(in_expr);
                    self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                        dest: dest_reg,
                        rvalue: rval,
                    });
                }
            }
            HirStatement::Assign { value, .. } => {
                let dest_reg = self.next_reg();
                let rval = self.lower_expression(value);
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: dest_reg,
                    rvalue: rval,
                });
            }
            HirStatement::Return { val, .. } => {
                let ret_reg = val.as_ref().map(|v| {
                    let r = self.next_reg();
                    let rval = self.lower_expression(v);
                    self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                        dest: r,
                        rvalue: rval,
                    });
                    r
                });
                self.blocks[self.current_block].terminator = MirTerminator::Return(ret_reg);
            }
            HirStatement::Expression(expr) => {
                let r = self.next_reg();
                let rval = self.lower_expression(expr);
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: r,
                    rvalue: rval,
                });
            }
            HirStatement::RegionBlock { name, body, .. } => {
                self.blocks[self.current_block].instructions.push(MirInstruction::RegionEnter {
                    region_id: name.clone(),
                });
                for s in body {
                    self.lower_statement(s);
                }
                self.blocks[self.current_block].instructions.push(MirInstruction::RegionExit {
                    region_id: name.clone(),
                });
            }
            HirStatement::If { cond, then_branch, else_branch, .. } => {
                let cond_reg = self.next_reg();
                let cond_rval = self.lower_expression(cond);
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: cond_reg,
                    rvalue: cond_rval,
                });

                let then_bb = self.next_block();
                let else_bb = self.next_block();
                let merge_bb = self.next_block();

                self.blocks[self.current_block].terminator = MirTerminator::Branch {
                    cond: cond_reg,
                    then_block: then_bb,
                    else_block: if else_branch.is_some() { else_bb } else { merge_bb },
                };

                // Then Block
                self.current_block = then_bb;
                for s in then_branch {
                    self.lower_statement(s);
                }
                self.blocks[self.current_block].terminator = MirTerminator::Goto(merge_bb);

                // Else Block
                if let Some(eb) = else_branch {
                    self.current_block = else_bb;
                    for s in eb {
                        self.lower_statement(s);
                    }
                    self.blocks[self.current_block].terminator = MirTerminator::Goto(merge_bb);
                }

                self.current_block = merge_bb;
            }
            HirStatement::While { cond, body, .. } => {
                let cond_bb = self.next_block();
                let body_bb = self.next_block();
                let end_bb = self.next_block();

                self.blocks[self.current_block].terminator = MirTerminator::Goto(cond_bb);

                // Condition Block
                self.current_block = cond_bb;
                let cond_reg = self.next_reg();
                let cond_rval = self.lower_expression(cond);
                self.blocks[cond_bb].instructions.push(MirInstruction::Assign {
                    dest: cond_reg,
                    rvalue: cond_rval,
                });
                self.blocks[cond_bb].terminator = MirTerminator::Branch {
                    cond: cond_reg,
                    then_block: body_bb,
                    else_block: end_bb,
                };

                // Body Block
                self.current_block = body_bb;
                for s in body {
                    self.lower_statement(s);
                }
                self.blocks[self.current_block].terminator = MirTerminator::Goto(cond_bb);

                self.current_block = end_bb;
            }
            HirStatement::Drop { var_name: _, .. } => {
                let r = self.next_reg();
                self.blocks[self.current_block].instructions.push(MirInstruction::Drop { reg: r });
            }
            _ => {}
        }
    }

    fn lower_expression(&mut self, expr: &HirExpression) -> MirRvalue {
        match expr {
            HirExpression::LitInt(n, _) => MirRvalue::ConstantInt(*n),
            HirExpression::LitFloat(f, _) => MirRvalue::ConstantFloat(*f),
            HirExpression::LitStr(s) => MirRvalue::ConstantStr(s.clone()),
            HirExpression::LitBool(b) => MirRvalue::ConstantBool(*b),
            HirExpression::Var(_, _) => {
                let r = self.next_reg();
                MirRvalue::Use(r)
            }
            HirExpression::Binary { op, left, right, .. } => {
                let l_rval = self.lower_expression(left);
                let l_reg = self.next_reg();
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: l_reg,
                    rvalue: l_rval,
                });

                let r_rval = self.lower_expression(right);
                let r_reg = self.next_reg();
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: r_reg,
                    rvalue: r_rval,
                });

                MirRvalue::BinaryOp(op.clone(), l_reg, r_reg)
            }
            HirExpression::Unary { op, expr, .. } => {
                let inner_rval = self.lower_expression(expr);
                let inner_reg = self.next_reg();
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: inner_reg,
                    rvalue: inner_rval,
                });
                MirRvalue::UnaryOp(op.clone(), inner_reg)
            }
            HirExpression::Call { callee, args, .. } => {
                let mut arg_regs = Vec::new();
                for a in args {
                    let a_rval = self.lower_expression(a);
                    let a_reg = self.next_reg();
                    self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                        dest: a_reg,
                        rvalue: a_rval,
                    });
                    arg_regs.push(a_reg);
                }
                MirRvalue::Call(callee.clone(), arg_regs)
            }
            HirExpression::Alloc { element_type, count, region_name, .. } => {
                let c_rval = self.lower_expression(count);
                let c_reg = self.next_reg();
                self.blocks[self.current_block].instructions.push(MirInstruction::Assign {
                    dest: c_reg,
                    rvalue: c_rval,
                });
                MirRvalue::Alloc(element_type.clone(), c_reg, region_name.clone())
            }
            _ => MirRvalue::ConstantBool(true),
        }
    }
}
