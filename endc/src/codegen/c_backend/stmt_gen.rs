use super::state::CBackend;
use crate::ast::Statement;

impl CBackend {
    pub(crate) fn gen_statement(&mut self, stmt: &Statement) {
        let span = stmt.span();
        let clean_file = span.file.replace('\\', "/");
        self.output.push_str(&format!("{}#line {} \"{}\"\n", self.indent(), span.line, clean_file));

        if self.gen_control_flow_statement(stmt) {
            return;
        }
        if self.gen_memory_and_regions_statement(stmt) {
            return;
        }
        if self.gen_sla_and_concurrency_statement(stmt) {
            return;
        }
        if self.gen_ops_and_events_statement(stmt) {
            return;
        }
        if self.gen_architecture_and_agents_statement(stmt) {
            return;
        }
        // Extensibility & Architectural Declarations (compile-time semantics & verification)
    }
}
