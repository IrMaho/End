use super::state::CBackend;
use crate::ast::Statement;

impl CBackend {
    pub(crate) fn gen_sla_and_concurrency_statement(&mut self, stmt: &Statement) -> bool {
        match stmt {
            Statement::BudgetBlock { specs, body, .. } => {
                let spec_str = specs.iter().map(|(k, v)| format!("{}: {}", k, v)).collect::<Vec<_>>().join(", ");
                self.output.push_str(&format!("{}/* ⏱️ [BUDGET SLA]: {} (runtime enforced) */\n", self.indent(), spec_str));
                if let Some(b) = body {
                    self.output.push_str(&format!("{}{{\n", self.indent()));
                    self.indent_level += 1;
                    self.output.push_str(&format!("{}struct timespec __budget_start, __budget_now;\n{}clock_gettime(CLOCK_MONOTONIC, &__budget_start);\n", self.indent(), self.indent()));
                    self.gen_block_statements(&b.statements);
                    self.output.push_str(&format!("{}clock_gettime(CLOCK_MONOTONIC, &__budget_now);\n", self.indent()));
                    self.output.push_str(&format!("{}int64_t __elapsed = (__budget_now.tv_sec - __budget_start.tv_sec) * 1000 + (__budget_now.tv_nsec - __budget_start.tv_nsec) / 1000000;\n", self.indent()));
                    self.output.push_str(&format!("{}if (__elapsed > 100) {{\n{}    fprintf(stderr, \"[END SLA BUDGET VIOLATION] Elapsed %lld ms exceeded budget in %s:%d\\n\", (long long)__elapsed, __FILE__, __LINE__);\n{}}}\n", self.indent(), self.indent(), self.indent()));
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::DeadlineBlock { duration, body, .. } => {
                self.output.push_str(&format!("{}/* ⏱️ [DEADLINE ENFORCEMENT]: {} */\n", self.indent(), duration));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}struct timespec __dl_start, __dl_now;\n{}clock_gettime(CLOCK_MONOTONIC, &__dl_start);\n", self.indent(), self.indent()));
                self.gen_block_statements(&body.statements);
                self.output.push_str(&format!("{}clock_gettime(CLOCK_MONOTONIC, &__dl_now);\n", self.indent()));
                self.output.push_str(&format!("{}int64_t __elapsed = (__dl_now.tv_sec - __dl_start.tv_sec) * 1000 + (__dl_now.tv_nsec - __dl_start.tv_nsec) / 1000000;\n", self.indent()));
                self.output.push_str(&format!("{}if (__elapsed > 100) {{\n{}    fprintf(stderr, \"[END SLA DEADLINE VIOLATION] Elapsed %lld ms exceeded deadline in %s:%d\\n\", (long long)__elapsed, __FILE__, __LINE__);\n{}}}\n", self.indent(), self.indent(), self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::PriorityBlock { level, body, .. } => {
                self.output.push_str(&format!("{}/* ⚡ [PRIORITY LEVEL: {}] */\n", self.indent(), level));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::QualityBlock { min_metric, max_latency, body, .. } => {
                self.output.push_str(&format!("{}/* 📊 [QUALITY CONSTRAINT]: min={}, max_latency={} */\n", self.indent(), min_metric, max_latency));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::TradeoffBlock { prefer, sacrifice, body, .. } => {
                self.output.push_str(&format!("{}/* ⚖️ [TRADEOFF]: prefer={}, sacrifice={} */\n", self.indent(), prefer, sacrifice));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&body.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::AdaptBlock { branches, .. } => {
                self.output.push_str(&format!("{}/* 🔄 [ADAPTIVE DISPATCH] */\n", self.indent()));
                let mut first = true;
                for (cond, blk) in branches {
                    let cond_str = self.gen_expression(cond);
                    if first {
                        self.output.push_str(&format!("{}if ({}) {{\n", self.indent(), cond_str));
                        first = false;
                    } else {
                        self.output.push_str(&format!("{}}} else if ({}) {{\n", self.indent(), cond_str));
                    }
                    self.indent_level += 1;
                    self.gen_block_statements(&blk.statements);
                    self.indent_level -= 1;
                }
                if !first {
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                true
            }
            Statement::Observe { metrics, .. } => {
                self.output.push_str(&format!("{}/* 👁️ [OBSERVE TELEMETRY]: {} */\n", self.indent(), metrics.join(", ")));
                true
            }
            Statement::WatchBlock { target, event, handler, .. } => {
                self.output.push_str(&format!("{}/* 👁️ [WATCH '{}' ON {}] */\n", self.indent(), target, event));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&handler.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ReactBlock { event, handler, .. } => {
                let event_str = self.gen_expression(event);
                self.output.push_str(&format!("{}/* ⚡ [REACT TO: {}] */\n", self.indent(), event_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&handler.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::StreamBlock { source, operations, .. } => {
                let src_str = self.gen_expression(source);
                self.output.push_str(&format!("{}/* 🌊 [STREAM PIPELINE: {}] */\n", self.indent(), src_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for op in operations {
                    let op_str = self.gen_expression(op);
                    self.output.push_str(&format!("{}{};\n", self.indent(), op_str));
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::FlowBlock { steps, .. } => {
                self.output.push_str(&format!("{}/* 🌊 [DATA FLOW PIPELINE] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                for step in steps {
                    let step_str = self.gen_expression(step);
                    self.output.push_str(&format!("{}{};\n", self.indent(), step_str));
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::ParallelChoose { branches, .. } => {
                self.output.push_str(&format!("{}/* 🔀 [PARALLEL CHOOSE DISPATCH] */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}volatile int __chosen_winner = -1;\n", self.indent()));
                self.output.push_str(&format!("{}#ifdef _OPENMP\n{}#pragma omp parallel num_threads({})\n{}{{\n", self.indent(), self.indent(), branches.len().max(1), self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}int __tid = omp_get_thread_num();\n", self.indent()));
                for (idx, (_b_name, blk)) in branches.iter().enumerate() {
                    self.output.push_str(&format!("{}if (__tid == {} && __chosen_winner < 0) {{\n", self.indent(), idx));
                    self.indent_level += 1;
                    self.gen_block_statements(&blk.statements);
                    self.output.push_str(&format!("{}int __exp = -1;\n{}__atomic_compare_exchange_n(&__chosen_winner, &__exp, {}, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);\n", self.indent(), self.indent(), idx));
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n{}#else\n", self.indent(), self.indent()));
                if let Some((_, first_blk)) = branches.first() {
                    self.gen_block_statements(&first_blk.statements);
                }
                self.output.push_str(&format!("{}#endif\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::RaceBlock { branches, .. } => {
                self.output.push_str(&format!("{}/* 🏁 [CONCURRENT RACE REGION] First completing branch wins */\n", self.indent()));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}volatile int __race_winner = -1;\n", self.indent()));
                self.output.push_str(&format!("{}#ifdef _OPENMP\n{}#pragma omp parallel num_threads({})\n{}{{\n", self.indent(), self.indent(), branches.len().max(1), self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}int __tid = omp_get_thread_num();\n", self.indent()));
                for (idx, blk) in branches.iter().enumerate() {
                    self.output.push_str(&format!("{}if (__tid == {} && __race_winner < 0) {{\n", self.indent(), idx));
                    self.indent_level += 1;
                    self.gen_block_statements(&blk.statements);
                    self.output.push_str(&format!("{}int __exp = -1;\n{}__atomic_compare_exchange_n(&__race_winner, &__exp, {}, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);\n", self.indent(), self.indent(), idx));
                    self.indent_level -= 1;
                    self.output.push_str(&format!("{}}}\n", self.indent()));
                }
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n{}#else\n", self.indent(), self.indent()));
                if let Some(first_blk) = branches.first() {
                    self.gen_block_statements(&first_blk.statements);
                }
                self.output.push_str(&format!("{}#endif\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            Statement::HedgeBlock { delay_ms, primary, fallback, .. } => {
                let delay_str = self.gen_expression(delay_ms);
                self.output.push_str(&format!("{}/* 🛡️ [TRUE LATENCY HEDGING: Primary at t=0, Fallback after {}ms delay] */\n", self.indent(), delay_str));
                self.output.push_str(&format!("{}{{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}volatile int __hedge_winner __attribute__((unused)) = -1;\n", self.indent()));
                self.output.push_str(&format!("{}#ifdef _OPENMP\n{}#pragma omp parallel num_threads(2)\n{}{{\n", self.indent(), self.indent(), self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}int __tid = omp_get_thread_num();\n", self.indent()));
                self.output.push_str(&format!("{}if (__tid == 0) {{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}/* Primary path starts immediately at t=0 */\n", self.indent()));
                self.gen_block_statements(&primary.statements);
                self.output.push_str(&format!("{}int __exp = -1;\n{}__atomic_compare_exchange_n(&__hedge_winner, &__exp, 0, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);\n", self.indent(), self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}} else if (__tid == 1) {{\n", self.indent()));
                self.indent_level += 1;
                self.output.push_str(&format!("{}/* Hedged secondary path delays by {}ms, then races if primary is not complete */\n", self.indent(), delay_str));
                self.output.push_str(&format!("{}END_CPU_SLEEP({});\n", self.indent(), delay_str));
                self.output.push_str(&format!("{}if (__hedge_winner < 0) {{\n", self.indent()));
                self.indent_level += 1;
                self.gen_block_statements(&fallback.statements);
                self.output.push_str(&format!("{}int __exp = -1;\n{}__atomic_compare_exchange_n(&__hedge_winner, &__exp, 1, 0, __ATOMIC_SEQ_CST, __ATOMIC_SEQ_CST);\n", self.indent(), self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n{}#else\n", self.indent(), self.indent()));
                self.output.push_str(&format!("{}struct timespec __h_start, __h_now;\n{}clock_gettime(CLOCK_MONOTONIC, &__h_start);\n", self.indent(), self.indent()));
                self.gen_block_statements(&primary.statements);
                self.output.push_str(&format!("{}clock_gettime(CLOCK_MONOTONIC, &__h_now);\n", self.indent()));
                self.output.push_str(&format!("{}int64_t __elapsed = (__h_now.tv_sec - __h_start.tv_sec) * 1000 + (__h_now.tv_nsec - __h_start.tv_nsec) / 1000000;\n", self.indent()));
                self.output.push_str(&format!("{}if (__elapsed >= {}) {{\n", self.indent(), delay_str));
                self.indent_level += 1;
                self.gen_block_statements(&fallback.statements);
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                self.output.push_str(&format!("{}#endif\n", self.indent()));
                self.indent_level -= 1;
                self.output.push_str(&format!("{}}}\n", self.indent()));
                true
            }
            _ => false,
        }
    }
}
