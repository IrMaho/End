use super::SemanticAnalyzer;
use crate::ast::*;

impl SemanticAnalyzer {
    pub(crate) fn analyze_block(&mut self, block: &Block) {
        self.push_scope();
        for stmt in &block.statements {
            self.analyze_statement(stmt);
        }
        self.pop_scope();
    }

    pub(crate) fn analyze_statement(&mut self, stmt: &Statement) {
        if !self.analyze_control_flow_statement(stmt) {
            self.analyze_architectural_statement(stmt);
        }
    }
}
