use crate::ast::*;
use std::collections::HashSet;

pub struct TreeShaker;

impl TreeShaker {
    pub fn prune_unreachable(module: &Module) -> (Module, usize) {
        let mut reachable_functions = HashSet::new();
        let mut reachable_structs = HashSet::new();

        // Entry points: 'main', test functions, or @export / pub functions
        for func in &module.functions {
            if func.name == "main"
                || func.is_pub
                || func.directives.iter().any(|d| d.name == "@test" || d.name == "@export" || d.name == "@c_export" || d.name == "@bench" || d.name == "@patrol")
            {
                reachable_functions.insert(func.name.clone());
                Self::visit_function_body(func, &mut reachable_functions, &mut reachable_structs, module);
            }
        }

        let initial_count = module.functions.len();
        let pruned_functions: Vec<FunctionDef> = module
            .functions
            .iter()
            .filter(|f| reachable_functions.contains(&f.name))
            .cloned()
            .collect();

        let removed = initial_count.saturating_sub(pruned_functions.len());

        let pruned_module = Module {
            name: module.name.clone(),
            imports: module.imports.clone(),
            enums: module.enums.clone(),
            structs: module.structs.clone(),
            traits: module.traits.clone(),
            impls: module.impls.clone(),
            functions: pruned_functions,
            modules: module.modules.clone(),
            extensions: module.extensions.clone(),
            span: module.span.clone(),
        };

        (pruned_module, removed)
    }

    fn visit_function_body(
        func: &FunctionDef,
        reachable_fn: &mut HashSet<String>,
        reachable_st: &mut HashSet<String>,
        module: &Module,
    ) {
        for stmt in &func.body.statements {
            Self::visit_statement(stmt, reachable_fn, reachable_st, module);
        }
    }

    fn visit_statement(
        stmt: &Statement,
        reachable_fn: &mut HashSet<String>,
        reachable_st: &mut HashSet<String>,
        module: &Module,
    ) {
        match stmt {
            Statement::VarDecl { initializer, .. } => {
                if let Some(init) = initializer {
                    Self::visit_expression(init, reachable_fn, reachable_st, module);
                }
            }
            Statement::Assignment { target, value, .. } => {
                Self::visit_expression(target, reachable_fn, reachable_st, module);
                Self::visit_expression(value, reachable_fn, reachable_st, module);
            }
            Statement::Return { value, .. } => {
                if let Some(val) = value {
                    Self::visit_expression(val, reachable_fn, reachable_st, module);
                }
            }
            Statement::Expression(expr) => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
            }
            Statement::If { condition, then_block, else_block, .. } => {
                Self::visit_expression(condition, reachable_fn, reachable_st, module);
                for s in &then_block.statements {
                    Self::visit_statement(s, reachable_fn, reachable_st, module);
                }
                if let Some(eb) = else_block {
                    for s in &eb.statements {
                        Self::visit_statement(s, reachable_fn, reachable_st, module);
                    }
                }
            }
            Statement::While { condition, body, .. } => {
                Self::visit_expression(condition, reachable_fn, reachable_st, module);
                for s in &body.statements {
                    Self::visit_statement(s, reachable_fn, reachable_st, module);
                }
            }
            Statement::ForIn { iterable, body, .. } | Statement::ParallelFor { iterable, body, .. } => {
                Self::visit_expression(iterable, reachable_fn, reachable_st, module);
                for s in &body.statements {
                    Self::visit_statement(s, reachable_fn, reachable_st, module);
                }
            }
            Statement::RegionBlock { body, .. } => {
                for s in &body.statements {
                    Self::visit_statement(s, reachable_fn, reachable_st, module);
                }
            }
            Statement::Match { expr, arms, .. } => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
                for arm in arms {
                    if let Some(guard) = &arm.guard {
                        Self::visit_expression(guard, reachable_fn, reachable_st, module);
                    }
                    for s in &arm.body.statements {
                        Self::visit_statement(s, reachable_fn, reachable_st, module);
                    }
                }
            }
            Statement::Defer { expr, .. } => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
            }
            Statement::Spawn { call, .. } => {
                Self::visit_expression(call, reachable_fn, reachable_st, module);
            }
            _ => {}
        }
    }

    fn visit_expression(
        expr: &Expression,
        reachable_fn: &mut HashSet<String>,
        reachable_st: &mut HashSet<String>,
        module: &Module,
    ) {
        match expr {
            Expression::Call { callee, args, .. } => {
                if let Expression::Ident(name, _) = callee.as_ref() {
                    if !reachable_fn.contains(name) {
                        reachable_fn.insert(name.clone());
                        if let Some(called_fn) = module.functions.iter().find(|f| &f.name == name) {
                            Self::visit_function_body(called_fn, reachable_fn, reachable_st, module);
                        }
                    }
                } else {
                    Self::visit_expression(callee, reachable_fn, reachable_st, module);
                }
                for a in args {
                    Self::visit_expression(a, reachable_fn, reachable_st, module);
                }
            }
            Expression::FieldAccess { object, .. } => {
                Self::visit_expression(object, reachable_fn, reachable_st, module);
            }
            Expression::Index { array, index, .. } => {
                Self::visit_expression(array, reachable_fn, reachable_st, module);
                Self::visit_expression(index, reachable_fn, reachable_st, module);
            }
            Expression::Cast { expr, .. } => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
            }
            Expression::Binary { left, right, .. } => {
                Self::visit_expression(left, reachable_fn, reachable_st, module);
                Self::visit_expression(right, reachable_fn, reachable_st, module);
            }
            Expression::Unary { expr, .. } => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
            }
            Expression::StructInit { name, fields, .. } => {
                reachable_st.insert(name.clone());
                for (_, fv) in fields {
                    Self::visit_expression(fv, reachable_fn, reachable_st, module);
                }
            }
            Expression::EnumInit { payload, .. } => {
                if let Some(p) = payload {
                    Self::visit_expression(p, reachable_fn, reachable_st, module);
                }
            }
            Expression::Alloc { allocator, .. } => {
                Self::visit_expression(allocator, reachable_fn, reachable_st, module);
            }
            Expression::Promote { expr, .. } => {
                Self::visit_expression(expr, reachable_fn, reachable_st, module);
            }
            _ => {}
        }
    }
}
