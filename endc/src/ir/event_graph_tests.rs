#[cfg(test)]
pub mod tests {
    use crate::lexer::Lexer;
    use crate::parser::Parser;
    use crate::ast::*;
    use crate::codegen::interpreter::{Interpreter, Value};
    use crate::codegen::c_backend::CBackend;

    fn parse_str(code: &str) -> Result<Module, String> {
        let mut lexer = Lexer::new("test.end", code);
        let tokens = lexer.tokenize_all()?;
        let mut parser = Parser::new("test.end", tokens);
        parser.parse_module("test_mod")
    }

    #[test]
    fn test_phase1_event_primitives_channels_subtypes() {
        let code = r#"
        event Heartbeat;
        event UserLogin(user_id: str, ip: str);
        event Stream<T>;
        event ClientDuplex <-> ServerDuplex;
        event ProducerHalf <~> ConsumerHalf;
        event Inbound -> Outbound;
        event SecurityAlert : SystemEvent with ring_buffer(4096), wal_persisted;

        fn main() -> i64 {
            ret 100;
        }
        "#;
        let module = parse_str(code).expect("Phase 1 event primitives must parse cleanly");
        let events: Vec<&EventDef> = module.statements.iter().filter_map(|s| match s {
            Statement::EventDecl(e) => Some(e),
            _ => None,
        }).collect();

        assert!(events.iter().any(|e| e.name == "Heartbeat"));
        assert!(events.iter().any(|e| e.name == "UserLogin" && e.fields.len() == 2));
        assert!(events.iter().any(|e| e.name == "Stream"));
        assert!(events.iter().any(|e| e.name == "ClientDuplex" && e.channel_kind == Some(EventChannelKind::Duplex)));
        assert!(events.iter().any(|e| e.name == "ProducerHalf" && e.channel_kind == Some(EventChannelKind::HalfDuplex)));
        assert!(events.iter().any(|e| e.name == "Inbound" && e.channel_kind == Some(EventChannelKind::SingleDirection)));
        assert!(events.iter().any(|e| e.name == "SecurityAlert" && e.parent_event.as_deref() == Some("SystemEvent")));
    }

    #[test]
    fn test_phase2_event_triggers_guards_filters_projections() {
        let code = r#"
        event OrderPlaced(order_id: i64, total: i64);

        fn main() -> i64 {
            state total = 500;
            emit OrderPlaced(101, 500);

            on OrderPlaced when total > 100 where total < 1000 {
                ret 1;
            }

            once SystemInit {
                ret 2;
            }

            every 100ms {
                ret 3;
            }

            after 500ms {
                ret 4;
            }

            before DatabaseFlush {
                ret 5;
            }

            ret 0;
        }
        "#;
        let module = parse_str(code).expect("Phase 2 event triggers must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter execution must succeed");
        assert_eq!(res, Value::Int(1));
    }

    #[test]
    fn test_phase3_reactive_state_and_derived_computations() {
        let code = r#"
        fn main() -> i64 {
            state count = 10;
            state price = 50;
            derive total from price, count => price * count;

            on count.changed {
                ret 42;
            }

            ret total;
        }
        "#;
        let module = parse_str(code).expect("Phase 3 reactive state must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must evaluate state and derive");
        assert_eq!(res, Value::Int(42));
    }

    #[test]
    fn test_phase4_stream_operators_and_windowing() {
        let code = r#"
        fn main() -> i64 {
            debounce 200ms on MouseMove;
            throttle 16ms on FrameRender;
            sample 10% on SensorTelemetry;
            coalesce on StateUpdate;
            window sliding(10s, 1s) on MetricStream;
            window tumbling(1m) on LogStream {
                ret 77;
            }

            ret 0;
        }
        "#;
        let module = parse_str(code).expect("Phase 4 stream operators must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must execute stream operators");
        assert_eq!(res, Value::Int(77));
    }

    #[test]
    fn test_phase5_execution_placement_and_memory_attributes() {
        let code = r#"
        @cpu(2)
        @local
        @lockfree
        @waitfree
        @pool
        @binary
        @arena(frame)
        @gpu
        @auto
        fn process_event_batch() -> i64 {
            ret 88;
        }

        fn main() -> i64 {
            ret process_event_batch();
        }
        "#;
        let module = parse_str(code).expect("Phase 5 attributes must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must run decorated event functions");
        assert_eq!(res, Value::Int(88));
    }

    #[test]
    fn test_phase6_backpressure_reliability_transactions() {
        let code = r#"
        event HighFrequencyTelemetry with capacity(1024), overflow(drop), require_ack, replayable, durable, event_sourced;

        fn main() -> i64 {
            event_transaction {
                state balance = 1000;
                ack OrderEvent;
            } rollback {
                state error_flag = 1;
            }

            ret 99;
        }
        "#;
        let module = parse_str(code).expect("Phase 6 backpressure and transactions must parse");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must evaluate transaction blocks");
        assert_eq!(res, Value::Int(99));
    }

    #[test]
    fn test_phase7_topologies_and_graph_execution() {
        let code = r#"
        topology DataIngestionGraph {
            Ingest -> Validate -> Enrich -> Persist;
            Validate -> Quarantine;
        }

        fn main() -> i64 {
            fuse StreamA, StreamB as UnifiedStream;
            quarantine PoisonPill;
            publish EventPayload to MessageBroker;
            drain EventStream;
            pause EventPipeline;
            resume EventPipeline;
            circuit_breaker on PaymentService;
            retry_policy 3 times on NetworkCall;
            dead_letter_queue DeadEvents;

            ret 105;
        }
        "#;
        let module = parse_str(code).expect("Phase 7 topologies must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must execute graph control statements");
        assert_eq!(res, Value::Int(105));
    }

    #[test]
    fn test_phase8_formal_invariants_and_evolution() {
        let code = r#"
        topology OrderFlow {
            Receive -> Process -> Fulfill;
        }

        fn main() -> i64 {
            invariant "event_throughput >= 10000";
            evolve event_topology OrderFlow {
                add CacheStage;
                remove RedundantValidation;
            }
            explain event_graph OrderFlow;

            ret 110;
        }
        "#;
        let module = parse_str(code).expect("Phase 8 formal invariants & evolution must parse");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Interpreter must run evolution statements");
        assert_eq!(res, Value::Int(110));
    }

    #[test]
    fn test_phase9_full_pipeline_native_c_codegen() {
        let code = r#"
        event PacketReceived(size: i64);
        event InboundStream -> OutboundStream;

        topology PacketPipeline {
            Ingress -> Parser -> Router -> Egress;
        }

        fn main() -> i64 {
            state total_bytes = 0;
            emit PacketReceived(1024);

            on PacketReceived when size > 0 {
                total_bytes = total_bytes + 1024;
            }

            every 10ms {
                total_bytes = total_bytes + 1;
            }

            circuit_breaker on Router;
            ret total_bytes;
        }
        "#;
        let module = parse_str(code).expect("Phase 9 code must parse cleanly");
        let mut backend = CBackend::new();
        let c_code = backend.generate(&module);

        assert!(c_code.contains("ON EVENT 'PacketReceived'"));
        assert!(c_code.contains("EVERY TICK: 10ms"));
        assert!(c_code.contains("EVENT CONTROL 'circuit_breaker' ON 'Router'"));
        assert!(c_code.contains("total_bytes"));
    }

    #[test]
    fn test_phase10_feature_file_integration() {
        let code = include_str!("../../../tests/features/17_event_primitives_and_execution_graphs.end");
        let module = parse_str(code).expect("Feature 17 .end file must parse cleanly");
        let mut interp = Interpreter::new();
        let res = interp.run(&module).expect("Feature 17 .end file interpreter run");
        assert_eq!(res, Value::Int(0));

        let mut c_gen = CBackend::new();
        let c_code = c_gen.generate(&module);
        assert!(c_code.contains("end_emit_event(\"UserLoggedIn\""));
    }
}

