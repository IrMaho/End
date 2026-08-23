use super::state::Interpreter;
use super::value::Value;
use crate::ast::Statement;

impl Interpreter {
    pub(crate) fn eval_architectural_block_statement(&mut self, stmt: &Statement) -> Result<Option<Option<Value>>, String> {
        match stmt {
            Statement::LeaseBlock { name, initializer, condition, body, .. } => {
                let init_val = self.eval_expression(initializer)?;
                if let Some(cond_expr) = condition {
                    let cond_val = self.eval_expression(cond_expr)?;
                    if let Value::Bool(false) = cond_val {
                        return Ok(Some(None));
                    }
                }
                self.push_scope();
                self.set_var(name, init_val);
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::LeaseCpu { body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::LeaseEvent { condition, body, .. } => {
                if let Some(cond_expr) = condition {
                    let cond_val = self.eval_expression(cond_expr)?;
                    if let Value::Bool(false) = cond_val {
                        return Ok(Some(None));
                    }
                }
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::LeaseLoop { item_name, iterable, body, .. } => {
                let iter_val = self.eval_expression(iterable)?;
                let count = match iter_val {
                    Value::Int(n) => n,
                    _ => 0,
                };
                self.push_scope();
                for i in 0..count {
                    self.set_var(item_name, Value::Int(i));
                    for s in &body.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::RegionBlock { name, body, .. } => {
                self.push_scope();
                self.set_var(&format!("region_{}", name), Value::String(format!("Region<{}>", name)));
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::AsmBlock { arch, .. } => {
                let _ = arch;
                Ok(Some(None))
            }
            Statement::TargetBlock { body, .. } => {
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        return Ok(Some(Some(ret)));
                    }
                }
                Ok(Some(None))
            }
            Statement::Defer { expr, .. } => {
                let _ = expr;
                Ok(Some(None))
            }
            Statement::Skip { .. } => Ok(Some(None)),
            Statement::InlineC { .. } => Ok(Some(None)),
            Statement::Spawn { call, .. } => {
                let _ = self.eval_expression(call)?;
                Ok(Some(None))
            }
            Statement::QuantumUnwrap { name, expr, fallback, .. } => {
                let val = self.eval_expression(expr)?;
                let final_val = match &val {
                    Value::Void => self.eval_expression(fallback)?,
                    Value::Int(0) => self.eval_expression(fallback)?,
                    _ => val,
                };
                self.set_var(name, final_val);
                Ok(Some(None))
            }
            Statement::AtomicOp { target, value, .. } => {
                let add_val = self.eval_expression(value)?;
                let mut is_updated = false;
                {
                    if let Some(Value::Int(curr)) = self.get_var(target) {
                        if let Value::Int(inc) = add_val {
                            let _ = self.update_var(target, Value::Int(curr + inc));
                            is_updated = true;
                        }
                    }
                }
                if !is_updated {
                    return Err(format!("AtomicOp failed: target '{}' is not an integer", target));
                }
                Ok(Some(None))
            }
            Statement::Owned { name, initializer, .. } => {
                let val = self.eval_expression(initializer)?;
                self.set_var(name, val);
                Ok(Some(None))
            }
            Statement::Intent { body, .. } => {
                if let Some(b) = body {
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            return Ok(Some(Some(ret)));
                        }
                    }
                }
                Ok(Some(None))
            }
            Statement::Prove { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Formal proof obligation failed at line {}", span.line)),
                    _ => Ok(Some(None)),
                }
            }
            Statement::Assume { condition, .. } => {
                let _ = self.eval_expression(condition)?;
                Ok(Some(None))
            }
            Statement::Guarantee { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Postcondition guarantee failed at line {}", span.line)),
                    _ => Ok(Some(None)),
                }
            }
            Statement::Invariant { condition, span } => {
                let cond_val = self.eval_expression(condition)?;
                match cond_val {
                    Value::Bool(false) => Err(format!("Invariant violation at line {}", span.line)),
                    _ => Ok(Some(None)),
                }
            }
            Statement::VerifyBlock { invariants, span } => {
                for inv in invariants {
                    let val = self.eval_expression(inv)?;
                    if let Value::Bool(false) = val {
                        return Err(format!("Verify contract failed at line {}", span.line));
                    }
                }
                Ok(Some(None))
            }
            Statement::Checkpoint { state_name, .. } => {
                self.snapshots.insert(state_name.clone(), self.variables.clone());
                Ok(Some(None))
            }
            Statement::Rollback { checkpoint_name, span } => {
                if let Some(snap) = self.snapshots.get(checkpoint_name).cloned() {
                    self.variables = snap;
                    Ok(Some(None))
                } else {
                    Err(format!("Rollback failed at line {}: checkpoint '{}' does not exist", span.line, checkpoint_name))
                }
            }
            Statement::TransactionBlock { body, .. } => {
                let pre_txn = self.variables.clone();
                self.push_scope();
                let mut txn_err = None;
                for s in &body.statements {
                    match self.eval_statement(s) {
                        Ok(Some(ret)) => {
                            self.pop_scope();
                            return Ok(Some(Some(ret)));
                        }
                        Ok(None) => {}
                        Err(e) => {
                            txn_err = Some(e);
                            break;
                        }
                    }
                }
                self.pop_scope();
                if let Some(err) = txn_err {
                    self.variables = pre_txn;
                    return Err(format!("Transaction aborted and rolled back: {}", err));
                }
                Ok(Some(None))
            }
            Statement::Handoff { resource, target_domain, .. } => {
                self.domain_ownership.insert(resource.clone(), target_domain.clone());
                Ok(Some(None))
            }
            Statement::ReturnTo { source_domain, resource, .. } => {
                if source_domain == "cpu" || source_domain == "host" {
                    self.domain_ownership.remove(resource);
                } else {
                    self.domain_ownership.insert(resource.clone(), source_domain.clone());
                }
                Ok(Some(None))
            }
            Statement::ProtectBlock { body, .. }
            | Statement::DeterministicBlock { body, .. }
            | Statement::ReplayBlock { body, .. }
            | Statement::SpeculativeBlock { body, .. }
            | Statement::FallbackBlock { body, .. }
            | Statement::CancelSafeBlock { body, .. }
            | Statement::TaskDecl { body, .. }
            | Statement::PatchDecl { body, .. }
            | Statement::RaceFreeBlock { body, .. }
            | Statement::DeadlineBlock { body, .. }
            | Statement::PriorityBlock { body, .. }
            | Statement::QualityBlock { body, .. }
            | Statement::TradeoffBlock { body, .. }
            | Statement::WatchBlock { handler: body, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::ComputeBlock { body, fallback, .. } => {
                self.push_scope();
                for s in &body.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                if let Some(fb) = fallback {
                    let _ = fb;
                }
                Ok(Some(None))
            }
            Statement::BudgetBlock { body, .. }
            | Statement::ContextBlock { body, .. }
            | Statement::AgentContract { body, .. }
            | Statement::EvolveBlock { body, .. } => {
                if let Some(b) = body {
                    self.push_scope();
                    for s in &b.statements {
                        if let Some(ret) = self.eval_statement(s)? {
                            self.pop_scope();
                            return Ok(Some(Some(ret)));
                        }
                    }
                    self.pop_scope();
                }
                Ok(Some(None))
            }
            Statement::AdaptBlock { branches, .. } => {
                for (cond, blk) in branches {
                    let cond_val = self.eval_expression(cond)?;
                    let is_match = match cond_val {
                        Value::Bool(b) => b,
                        Value::Int(n) => n != 0,
                        _ => false,
                    };
                    if is_match {
                        self.push_scope();
                        for s in &blk.statements {
                            if let Some(ret) = self.eval_statement(s)? {
                                self.pop_scope();
                                return Ok(Some(Some(ret)));
                            }
                        }
                        self.pop_scope();
                        break;
                    }
                }
                Ok(Some(None))
            }
            Statement::ReactBlock { event, handler, .. } => {
                let _ = self.eval_expression(event)?;
                self.push_scope();
                for s in &handler.statements {
                    if let Some(ret) = self.eval_statement(s)? {
                        self.pop_scope();
                        return Ok(Some(Some(ret)));
                    }
                }
                self.pop_scope();
                Ok(Some(None))
            }
            Statement::StreamBlock { source, operations, .. } => {
                let _ = self.eval_expression(source)?;
                for op in operations {
                    let _ = self.eval_expression(op)?;
                }
                Ok(Some(None))
            }
            Statement::FlowBlock { steps, .. } => {
                for step in steps {
                    let _ = self.eval_expression(step)?;
                }
                Ok(Some(None))
            }
            Statement::ParallelChoose { branches, .. } => {
                let (tx, rx) = std::sync::mpsc::channel();
                for (branch_name, blk) in branches {
                    let blk_c = blk.clone();
                    let mut interp_c = self.clone();
                    let tx_c = tx.clone();
                    let b_name = branch_name.clone();
                    std::thread::spawn(move || {
                        interp_c.push_scope();
                        let mut res = None;
                        for s in &blk_c.statements {
                            if let Ok(Some(ret)) = interp_c.eval_statement(s) {
                                res = Some(ret);
                                break;
                            }
                        }
                        let _ = tx_c.send((b_name, res, interp_c.variables));
                    });
                }
                drop(tx);
                if let Ok((_chosen_name, maybe_val, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                    if let Some(top_scope) = updated_vars.last() {
                        for (k, v) in top_scope {
                            self.set_var(k, v.clone());
                        }
                    }
                    if let Some(v) = maybe_val {
                        return Ok(Some(Some(v)));
                    }
                }
                Ok(Some(None))
            }
            Statement::RaceBlock { branches, .. } => {
                let (tx, rx) = std::sync::mpsc::channel();
                for (idx, blk) in branches.iter().enumerate() {
                    let blk_c = blk.clone();
                    let mut interp_c = self.clone();
                    let tx_c = tx.clone();
                    std::thread::spawn(move || {
                        interp_c.push_scope();
                        let mut res = None;
                        for s in &blk_c.statements {
                            if let Ok(Some(ret)) = interp_c.eval_statement(s) {
                                res = Some(ret);
                                break;
                            }
                        }
                        let _ = tx_c.send((idx, res, interp_c.variables));
                    });
                }
                drop(tx);
                if let Ok((_winner_idx, maybe_val, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                    if let Some(top_scope) = updated_vars.last() {
                        for (k, v) in top_scope {
                            self.set_var(k, v.clone());
                        }
                    }
                    if let Some(v) = maybe_val {
                        return Ok(Some(Some(v)));
                    }
                }
                Ok(Some(None))
            }
            Statement::HedgeBlock { delay_ms, primary, fallback, .. } => {
                let delay_ms_val = match self.eval_expression(delay_ms)? {
                    Value::Int(n) if n > 0 => n as u64,
                    _ => 20,
                };
                let (tx, rx) = std::sync::mpsc::channel();
                let prim_blk = primary.clone();
                let mut prim_interp = self.clone();
                let tx_prim = tx.clone();
                std::thread::spawn(move || {
                    prim_interp.push_scope();
                    let mut res = None;
                    for s in &prim_blk.statements {
                        if let Ok(Some(ret)) = prim_interp.eval_statement(s) {
                            res = Some(ret);
                            break;
                        }
                    }
                    let _ = tx_prim.send(("primary", res, prim_interp.variables));
                });

                match rx.recv_timeout(std::time::Duration::from_millis(delay_ms_val)) {
                    Ok((_, maybe_v, updated_vars)) => {
                        if let Some(top_scope) = updated_vars.last() {
                            for (k, v) in top_scope {
                                self.set_var(k, v.clone());
                            }
                        }
                        Ok(Some(maybe_v))
                    }
                    Err(_) => {
                        let fb_blk = fallback.clone();
                        let mut fb_interp = self.clone();
                        let tx_fb = tx.clone();
                        std::thread::spawn(move || {
                            fb_interp.push_scope();
                            let mut res = None;
                            for s in &fb_blk.statements {
                                if let Ok(Some(ret)) = fb_interp.eval_statement(s) {
                                    res = Some(ret);
                                    break;
                                }
                            }
                            let _ = tx_fb.send(("fallback", res, fb_interp.variables));
                        });
                        drop(tx);
                        if let Ok((_, maybe_v, updated_vars)) = rx.recv_timeout(std::time::Duration::from_millis(500)) {
                            if let Some(top_scope) = updated_vars.last() {
                                for (k, v) in top_scope {
                                    self.set_var(k, v.clone());
                                }
                            }
                            Ok(Some(maybe_v))
                        } else {
                            Ok(Some(None))
                        }
                    }
                }
            }
            _ => Ok(None),
        }
    }
}
