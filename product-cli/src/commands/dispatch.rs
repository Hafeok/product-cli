//! Subcommand dispatch — match a parsed `Commands` value to its handler.

use clap::Command as ClapCommand;

use super::{
    adr, agent_init, author, cell, checklist, completions, conformance, context, cycle_times, dep,
    domain, drift, feature, gap, graph_cmd, hash, hooks, how, implement, init, mcp_cmd,
    metrics_cmd, migrate, onboard, pattern, preflight, prompts_cmd, render, request_cmd, schema,
    status, tags, test_cmd, BoxResult, Commands,
};

pub(crate) fn dispatch(command: Commands, fmt: &str, cli_command: &mut ClapCommand) -> BoxResult {
    match command {
        Commands::Adr { command } => adr::handle_adr(command, fmt),
        Commands::AgentInit { watch } => agent_init::handle_agent_init(watch),
        Commands::Author { command } => author::handle_author(command),
        Commands::Cell { command } => cell::handle_cell(command),
        Commands::Checklist { command } => checklist::handle_checklist(command),
        Commands::Completions { shell } => completions::handle_completions(&shell, cli_command),
        Commands::Conformance { command } => conformance::handle_conformance(command, fmt),
        Commands::Context { .. } => dispatch_context(command),
        Commands::CycleTimes { .. } => dispatch_cycle_times(command, fmt),
        Commands::Dep { command } => dep::handle_dep(command, fmt),
        Commands::Domain { command } => domain::handle_domain_cmd(command),
        Commands::Drift { command } => drift::handle_drift(command, fmt),
        Commands::Feature { command } => feature::handle_feature(command, fmt),
        Commands::Forecast { .. } => dispatch_forecast(command, fmt),
        Commands::Gap { command } => gap::handle_gap(command, fmt),
        Commands::Graph { command } => graph_cmd::handle_graph(command, fmt),
        Commands::Hash { command } => hash::handle_hash(command),
        Commands::How { command } => how::handle_how(command),
        Commands::Impact { id } => render(status::handle_impact(&id, fmt), fmt),
        Commands::Implement { .. } => dispatch_implement(command),
        Commands::Init { .. } => dispatch_init(command),
        Commands::InstallHooks => hooks::handle_install_hooks(),
        Commands::Mcp { .. } => dispatch_mcp(command),
        Commands::Metrics { command } => metrics_cmd::handle_metrics(command),
        Commands::Migrate { command } => migrate::handle_migrate(command),
        Commands::Onboard { command } => onboard::handle_onboard(command),
        Commands::Pattern { command } => pattern::handle_pattern(command, fmt),
        Commands::Preflight { id } => preflight::handle_preflight(&id, fmt),
        Commands::Prompts { command } => prompts_cmd::handle_prompts(command),
        Commands::Request { command } => request_cmd::handle_request(command, fmt),
        Commands::Schema { artifact_type, type_flag, all } => {
            schema::handle_schema(type_flag.or(artifact_type), all)
        }
        Commands::Status { phase, untested, failing } => {
            render(status::handle_status(phase, untested, failing, fmt), fmt)
        }
        Commands::Tags { command } => tags::handle_tags(command, fmt),
        Commands::Test { command } => test_cmd::handle_test(command, fmt),
        Commands::Verify { .. } => dispatch_verify(command, fmt),
    }
}

fn dispatch_context(command: Commands) -> BoxResult {
    let Commands::Context {
        id,
        depth,
        phase,
        adrs_only,
        order,
        measure,
        measure_all,
        target,
        for_llm,
        show,
        where_flag,
        reset,
    } = command
    else {
        unreachable!("dispatch_context called with non-Context variant")
    };
    context::handle_context(context::ContextArgs {
        id: id.as_deref(),
        depth,
        phase,
        adrs_only,
        order,
        measure,
        measure_all,
        target,
        for_llm,
        show,
        where_flag,
        reset,
    })
}

fn dispatch_cycle_times(command: Commands, fmt: &str) -> BoxResult {
    let Commands::CycleTimes { recent, phase, in_progress, format } = command else {
        unreachable!("dispatch_cycle_times called with non-CycleTimes variant")
    };
    let effective_fmt = format.as_deref().unwrap_or(fmt);
    render(
        cycle_times::handle_cycle_times(recent, phase, in_progress, effective_fmt),
        effective_fmt,
    )
}

fn dispatch_forecast(command: Commands, fmt: &str) -> BoxResult {
    let Commands::Forecast { id, phase, naive, sample_size } = command else {
        unreachable!("dispatch_forecast called with non-Forecast variant")
    };
    cycle_times::handle_forecast(id.as_deref(), phase, naive, sample_size, fmt)
}

fn dispatch_implement(command: Commands) -> BoxResult {
    let Commands::Implement {
        id,
        dry_run,
        no_verify,
        headless,
        no_auto_runners,
        target,
    } = command
    else {
        unreachable!("dispatch_implement called with non-Implement variant")
    };
    implement::handle_implement(
        &id,
        dry_run,
        no_verify,
        headless,
        no_auto_runners,
        target.as_deref(),
    )
}

fn dispatch_init(command: Commands) -> BoxResult {
    let Commands::Init {
        yes,
        force,
        name,
        description,
        domains,
        port,
        write_tools,
        legacy_layout,
        path,
    } = command
    else {
        unreachable!("dispatch_init called with non-Init variant")
    };
    init::handle_init(
        yes,
        force,
        name,
        description,
        domains,
        port,
        write_tools,
        legacy_layout,
        path,
    )
}

fn dispatch_mcp(command: Commands) -> BoxResult {
    let Commands::Mcp { http, port, bind, token, repo, write } = command else {
        unreachable!("dispatch_mcp called with non-Mcp variant")
    };
    mcp_cmd::handle_mcp(http, port, &bind, token, repo, write)
}

fn dispatch_verify(command: Commands, fmt: &str) -> BoxResult {
    let Commands::Verify { id, platform, skip_adr_check, phase, ci } = command else {
        unreachable!("dispatch_verify called with non-Verify variant")
    };
    implement::handle_verify(id.as_deref(), platform, skip_adr_check, phase, ci, fmt)
}
