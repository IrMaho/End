use super::state::Interpreter;
use super::value::Value;
use crate::ast::Statement;

impl Interpreter {
    pub fn eval_statement(&mut self, stmt: &Statement) -> Result<Option<Value>, String> {
        if let Some(res) = self.eval_control_flow_statement(stmt)? {
            return Ok(res);
        }
        if let Some(res) = self.eval_architectural_block_statement(stmt)? {
            return Ok(res);
        }
        if let Some(res) = self.eval_operations_and_events_statement(stmt)? {
            return Ok(res);
        }
        self.eval_agent_contracts_and_oop_statement(stmt)
    }
}
