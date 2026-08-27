use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use super::sampler::ProfilerSession;
use super::types::ProfilingReport;
use crate::codegen::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

pub struct EndProfiler;

impl EndProfiler {
    pub fn profile_execution(target: &str) -> ProfilingReport {
        let path = Path::new(target);
        if target.ends_with(".end") || path.extension().map_or(false, |ext| ext == "end") {
            match std::fs::read_to_string(path) {
                Ok(source) => {
                    match Self::profile_source(&source, target) {
                        Ok(report) => return report,
                        Err(e) => {
                            eprintln!("Error executing profiled source: {}", e);
                            let mut empty_session = ProfilerSession::new(target);
                            return empty_session.finish();
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Profiler target file not found: {} ({})", target, e);
                    let mut empty_session = ProfilerSession::new(target);
                    return empty_session.finish();
                }
            }
        }

        // If target is an executable file or binary
        if path.exists() {
            let start = Instant::now();
            let mut session = ProfilerSession::new(target);
            session.enter_function("process_spawn");
            
            let cmd_res = std::process::Command::new(target).output();
            let elapsed_us = start.elapsed().as_micros() as u64;
            session.exit_function("process_spawn", elapsed_us, elapsed_us, 1024);

            match cmd_res {
                Ok(_out) => {
                    session.enter_function("main");
                    session.exit_function("main", elapsed_us, elapsed_us, 2048);
                    return session.finish();
                }
                Err(e) => {
                    eprintln!("Error executing process: {}", e);
                    return session.finish();
                }
            }
        }

        // Fallback: try parsing target as inline source code
        if let Ok(report) = Self::profile_source(target, "inline_snippet") {
            return report;
        }

        eprintln!("Profiler target not recognized or not found: {}", target);
        let mut session = ProfilerSession::new(target);
        session.finish()
    }

    pub fn profile_source(source: &str, target_name: &str) -> Result<ProfilingReport, String> {
        let mut lexer = Lexer::new(target_name, source);
        let tokens = lexer.tokenize_all().map_err(|e| format!("Lexer error: {}", e))?;
        let mut parser = Parser::new(target_name, tokens);
        let module = parser.parse_module(target_name).map_err(|e| format!("Parser error: {:?}", e))?;

        let session = Arc::new(Mutex::new(ProfilerSession::new(target_name)));

        let mut interp = Interpreter::new();
        interp.profiler_session = Some(session.clone());

        // Run statements and entry main
        let _ = interp.run(&module);

        let report = session.lock().unwrap().finish();
        Ok(report)
    }
}
