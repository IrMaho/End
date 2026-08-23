pub mod agent_memory;
pub mod agent_ops;
pub mod build;
pub mod dev_tools;
pub mod exec;
pub mod package_gen;
pub mod security_verify;
pub mod semantic_query;
pub mod test_sim;

use crate::cli::Commands;

pub fn dispatch_command(command: Commands) {
    match command {
        Commands::Build(args) => build::handle_build(args),
        Commands::Run(args) => exec::handle_run(args),
        Commands::Check(args) => exec::handle_check(args),
        Commands::Version => exec::handle_version(),
        Commands::Lsp => exec::handle_lsp(),
        Commands::Dap => exec::handle_dap(),
        Commands::Repl => exec::handle_repl(),

        Commands::Test(args) => test_sim::handle_test(args),
        Commands::Simulate(args) => test_sim::handle_simulate(args),
        Commands::Stress(args) => test_sim::handle_stress(args),
        Commands::Fuzz(args) => test_sim::handle_fuzz(args),
        Commands::Profile(args) => test_sim::handle_profile(args),

        Commands::Dev(args) => dev_tools::handle_dev(args),
        Commands::Patrol(args) => dev_tools::handle_patrol(args),
        Commands::Watch(args) => dev_tools::handle_watch(args),
        Commands::Fmt(args) => dev_tools::handle_fmt(args),
        Commands::Lint(args) => dev_tools::handle_lint(args),
        Commands::Explore(args) => dev_tools::handle_explore(args),

        Commands::New(args) => package_gen::handle_new(args),
        Commands::Init => package_gen::handle_init(),
        Commands::Add(args) => package_gen::handle_add(args),
        Commands::Publish(args) => package_gen::handle_publish(args),
        Commands::Install => package_gen::handle_install(),
        Commands::Doc(args) => package_gen::handle_doc(args),
        Commands::Bindgen(args) => package_gen::handle_bindgen(args),
        Commands::CBindgen(args) => package_gen::handle_cbindgen(args),
        Commands::Gen(args) => package_gen::handle_gen(args),
        Commands::ConfigInit => package_gen::handle_config_init(),
        Commands::Mobile(args) => package_gen::handle_mobile(args),
        Commands::Flutter(args) => package_gen::handle_flutter(args),

        Commands::Inspect(args) => semantic_query::handle_inspect(args),
        Commands::Explain(args) => semantic_query::handle_explain(args),
        Commands::Trace(args) => semantic_query::handle_trace(args),
        Commands::Effects(args) => semantic_query::handle_effects(args),
        Commands::Impact(args) => semantic_query::handle_impact(args),
        Commands::Graph(args) => semantic_query::handle_graph(args),
        Commands::Query(args) => semantic_query::handle_query(args),
        Commands::Slice(args) => semantic_query::handle_slice(args),
        Commands::Patch(args) => semantic_query::handle_patch(args),
        Commands::Eval(args) => semantic_query::handle_eval(args),
        Commands::Arch(args) => semantic_query::handle_arch(args),
        Commands::Fix(args) => semantic_query::handle_fix(args),

        Commands::Ui(args) => agent_ops::handle_ui(args),
        Commands::Agent(args) => agent_ops::handle_agent(args),
        Commands::Skill(args) => agent_ops::handle_skill(args),
        Commands::Dna(args) => agent_ops::handle_dna(args),
        Commands::Context(args) => agent_ops::handle_context(args),
        Commands::Precheck(args) => agent_ops::handle_precheck(args),

        Commands::Memory(args) => agent_memory::handle_memory(args),
        Commands::Scope(args) => agent_memory::handle_scope(args),
        Commands::SemanticGit(args) => agent_memory::handle_semantic_git(args),
        Commands::SemanticIr(args) => agent_memory::handle_semantic_ir(args),
        Commands::AgentRun(args) => agent_memory::handle_agent_run(args),
        Commands::Evolve(args) => agent_memory::handle_evolve(args),

        Commands::IntentVerify(args) => security_verify::handle_intent_verify(args),
        Commands::Verify(args) => security_verify::handle_verify(args),
        Commands::Security(args) => security_verify::handle_security(args),
        Commands::Attest(args) => security_verify::handle_attest(args),
        Commands::Api(args) => security_verify::handle_api(args),
        Commands::Feature(args) => security_verify::handle_feature(args),
    }
}
