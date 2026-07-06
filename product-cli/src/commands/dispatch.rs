//! Subcommand dispatch — match a parsed `Commands` value to its handler.

use clap::Command as ClapCommand;

use super::{
    author, blueprint, build, cell, codegen, completions, decider, deliverable, deployable_unit,
    design_system, domain,
    guide, hooks, how, init, lsp, mcp_cmd, preview, primitive, product, projector, release, render,
    scope, seam, session, skills, feature, target, verdict, work_unit, worker, BoxResult, Commands,
};

pub(crate) fn dispatch(command: Commands, fmt: &str, cli_command: &mut ClapCommand) -> BoxResult {
    match command {
        Commands::Author { command } => author::handle_author(command),
        Commands::Build { .. } => dispatch_build(command),
        Commands::Completions { shell } => completions::handle_completions(&shell, cli_command),
        Commands::Guide => render(guide::handle_guide(), fmt),
        Commands::Init { .. } => dispatch_init(command),
        Commands::InstallHooks => hooks::handle_install_hooks(),
        Commands::Lsp { command } => lsp::handle_lsp(command),
        Commands::Mcp { .. } => dispatch_mcp(command),
        Commands::Scope { command } => render(scope::handle_scope(command), fmt),
        Commands::Session { command } => session::handle_session(command),
        Commands::Skills { command } => skills::handle_skills(command),
        Commands::Seam { id, product } => seam::handle_seam(id, product),
        Commands::Verdict { file } => verdict::handle_verdict(file),
        // Product-Framework families route through a sub-dispatcher (keeps this match small).
        c @ (Commands::Blueprint { .. } | Commands::Cell { .. } | Commands::Decider { .. } | Commands::Projector { .. } | Commands::Primitive { .. } | Commands::Product { .. } | Commands::Deliverable { .. } | Commands::DeployableUnit { .. } | Commands::DesignSystem { .. } | Commands::Domain { .. } | Commands::How { .. } | Commands::Preview { .. } | Commands::Codegen { .. } | Commands::Release { .. } | Commands::Feature { .. } | Commands::Target { .. } | Commands::WorkUnit { .. } | Commands::Worker { .. }) => dispatch_pf(c),
    }
}

/// Sub-dispatcher for the Product-Framework command families.
fn dispatch_pf(command: Commands) -> BoxResult {
    match command {
        Commands::Blueprint { command } => blueprint::handle_blueprint(command),
        Commands::Cell { command } => cell::handle_cell(command),
        Commands::Decider { command } => decider::handle_decider(command),
        Commands::Projector { command } => projector::handle_projector(command),
        Commands::Primitive { command } => primitive::handle_primitive(command),
        Commands::Product { command } => product::handle_product(command),
        Commands::Deliverable { command } => deliverable::handle_deliverable(command),
        Commands::DeployableUnit { command } => deployable_unit::handle_deployable_unit(command),
        Commands::DesignSystem { command } => design_system::handle_design_system(command),
        Commands::Domain { command } => domain::handle_domain_cmd(command),
        Commands::How { command } => how::handle_how(command),
        Commands::Preview { command } => preview::handle_preview(command),
        Commands::Codegen { command } => codegen::handle_reify(command),
        Commands::Release { command } => release::handle_release(command),
        Commands::Feature { command } => feature::handle_feature(command),
        Commands::Target { command } => target::handle_target(command),
        Commands::WorkUnit { command } => work_unit::handle_work_unit(command),
        Commands::Worker { command } => worker::handle_worker(command),
        _ => unreachable!("dispatch_pf called with non-pf variant"),
    }
}

fn dispatch_build(command: Commands) -> BoxResult {
    let Commands::Build { deliverable, role, jobs, dry_run, lsp, no_verify, max_rounds, budget, emit_spmc, emit_seam, out, product } = command else {
        unreachable!("dispatch_build called with non-Build variant")
    };
    let gates = build::Gates { lsp, verify: !no_verify, max_rounds, budget };
    build::handle_build(&deliverable, &role, jobs, dry_run, gates, emit_spmc, emit_seam, out, product)
}

fn dispatch_init(command: Commands) -> BoxResult {
    let Commands::Init {
        yes,
        force,
        name,
        port,
        write_tools,
        legacy_layout,
        path,
        demo,
        no_skills,
        cli,
    } = command
    else {
        unreachable!("dispatch_init called with non-Init variant")
    };
    init::handle_init(yes, force, name, port, write_tools, legacy_layout, path, demo, no_skills, cli)
}

fn dispatch_mcp(command: Commands) -> BoxResult {
    let Commands::Mcp { http, port, bind, token, repo, write, workflow, session } = command else {
        unreachable!("dispatch_mcp called with non-Mcp variant")
    };
    mcp_cmd::handle_mcp(http, port, &bind, token, repo, write, workflow, session)
}
