use super::types::OwnershipState;
use crate::ast::Type;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct VarSymbol {
    pub name: String,
    pub var_type: Type,
    pub line_def: usize,
    pub is_mut: bool,
    pub initialized: bool,
    pub ownership: OwnershipState,
}

#[derive(Debug, Clone)]
pub struct TypeEnv {
    pub scopes: Vec<HashMap<String, VarSymbol>>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn declare(&mut self, name: &str, var_type: Type, line_def: usize, is_mut: bool) {
        if let Some(current) = self.scopes.last_mut() {
            current.insert(
                name.to_string(),
                VarSymbol {
                    name: name.to_string(),
                    var_type,
                    line_def,
                    is_mut,
                    initialized: true,
                    ownership: OwnershipState::Owned,
                },
            );
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&VarSymbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut VarSymbol> {
        for scope in self.scopes.iter_mut().rev() {
            if let Some(sym) = scope.get_mut(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn is_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().map(|s| s.contains_key(name)).unwrap_or(false)
    }

    pub fn get_ownership(&self, name: &str) -> Option<OwnershipState> {
        self.lookup(name).map(|s| s.ownership.clone())
    }

    pub fn set_ownership(&mut self, name: &str, new_state: OwnershipState) {
        if let Some(sym) = self.lookup_mut(name) {
            sym.ownership = new_state;
        }
    }

    pub fn all_symbols(&self) -> impl Iterator<Item = &VarSymbol> {
        self.scopes.iter().flat_map(|s| s.values())
    }
}
