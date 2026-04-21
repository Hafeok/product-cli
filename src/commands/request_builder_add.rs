//! CLI adapter — `product request add …` subcommand dispatch.

use clap::Subcommand;
use product_lib::config::ProductConfig;
use product_lib::graph::KnowledgeGraph;
use product_lib::parser;
use product_lib::request::builder::{add, draft::Draft, render};
use product_lib::request::Finding;

use super::BoxResult;

#[derive(Subcommand)]
pub enum AddCommands {
    /// Append a new feature artifact (create-mode drafts)
    Feature {
        #[arg(long)] title: String,
        #[arg(long)] phase: u32,
        /// Comma-separated list of domains
        #[arg(long, default_value = "")] domains: String,
        #[arg(long = "ref")] ref_name: Option<String>,
    },
    /// Append a new ADR artifact (create-mode drafts)
    Adr {
        #[arg(long)] title: String,
        #[arg(long, default_value = "")] domains: String,
        #[arg(long)] scope: Option<String>,
        #[arg(long, default_value = "")] governs: String,
        #[arg(long = "ref")] ref_name: Option<String>,
    },
    /// Append a new test-criterion artifact (create-mode drafts)
    Tc {
        #[arg(long)] title: String,
        #[arg(long = "tc-type")] tc_type: String,
        #[arg(long, default_value = "")] validates_features: String,
        #[arg(long, default_value = "")] validates_adrs: String,
        #[arg(long = "ref")] ref_name: Option<String>,
    },
    /// Append a new dependency artifact (create-mode drafts)
    Dep {
        #[arg(long)] title: String,
        #[arg(long = "dep-type")] dep_type: String,
        #[arg(long)] version: Option<String>,
        /// `--adr new` to author a governing ADR in the same step, or an ID / ref name
        #[arg(long)] adr: Option<String>,
        #[arg(long = "adr-title")] adr_title: Option<String>,
        #[arg(long = "ref")] ref_name: Option<String>,
    },
    /// Append a documentation artifact (structural convenience — maps to an ADR)
    Doc {
        #[arg(long)] title: String,
        #[arg(long, default_value = "")] domains: String,
        #[arg(long = "ref")] ref_name: Option<String>,
    },
    /// Add a change block targeting an existing artifact (change-mode drafts)
    Target {
        /// Target artifact ID (e.g. FT-001)
        id: String,
    },
    /// Add a domain acknowledgement shortcut (change-mode drafts)
    Acknowledgement {
        /// Target artifact ID (e.g. FT-001)
        id: String,
        /// Domain name
        domain: String,
        /// Reason (non-empty)
        reason: String,
    },
}

pub(crate) fn handle_add(cmd: AddCommands) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let mut draft = load_draft(&root)?;
    let graph = build_graph(&config, &root)?;
    let outcome = dispatch_add(cmd, &mut draft, &config, &graph);
    finalize_add(outcome, &draft)
}

fn dispatch_add(
    cmd: AddCommands,
    draft: &mut product_lib::request::builder::draft::Draft,
    config: &ProductConfig,
    graph: &KnowledgeGraph,
) -> Result<add::AddedArtifact, Vec<Finding>> {
    match cmd {
        AddCommands::Feature { title, phase, domains, ref_name } => add::add_feature(
            draft,
            add::AddFeatureArgs { title, phase, domains: split_csv(&domains), ref_name },
            config, graph,
        ),
        AddCommands::Adr { title, domains, scope, governs, ref_name } => add::add_adr(
            draft,
            add::AddAdrArgs {
                title,
                domains: split_csv(&domains),
                scope,
                governs: split_csv(&governs),
                ref_name,
            },
            config, graph,
        ),
        AddCommands::Tc { title, tc_type, validates_features, validates_adrs, ref_name } => {
            add::add_tc(
                draft,
                add::AddTcArgs {
                    title, tc_type,
                    validates_features: split_csv(&validates_features),
                    validates_adrs: split_csv(&validates_adrs),
                    ref_name,
                },
                config, graph,
            )
        }
        AddCommands::Dep { title, dep_type, version, adr, adr_title, ref_name } => add::add_dep(
            draft,
            add::AddDepArgs { title, dep_type, version, adr, adr_title, ref_name },
            config, graph,
        ),
        AddCommands::Doc { title, domains, ref_name } => add::add_doc(
            draft,
            add::AddDocArgs { title, domains: split_csv(&domains), ref_name },
            config, graph,
        ),
        AddCommands::Target { id } => add::add_target(
            draft,
            add::AddTargetArgs { target: id, mutations: Vec::new() },
            config, graph,
        ),
        AddCommands::Acknowledgement { id, domain, reason } => add::add_acknowledgement(
            draft,
            add::AddAckArgs { target: id, domain, reason },
            config, graph,
        ),
    }
}

fn finalize_add(
    outcome: Result<add::AddedArtifact, Vec<Finding>>,
    draft: &product_lib::request::builder::draft::Draft,
) -> BoxResult {
    match outcome {
        Ok(added) => {
            draft.save()?;
            let warn_codes: Vec<String> = added
                .findings
                .iter()
                .filter(|f| !f.is_error())
                .map(|f| f.code.clone())
                .collect();
            print!(
                "{}",
                render::render_added(&added.refs, added.note.as_deref(), &warn_codes)
            );
            Ok(())
        }
        Err(findings) => {
            for f in &findings {
                eprintln!("{f}\n");
            }
            std::process::exit(1);
        }
    }
}

fn split_csv(s: &str) -> Vec<String> {
    s.split(',')
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn load_draft(root: &std::path::Path) -> Result<Draft, Box<dyn std::error::Error>> {
    match Draft::load(root) {
        None => Err("no active draft — run `product request new create|change`".into()),
        Some(Ok(d)) => Ok(d),
        Some(Err(e)) => Err(format!("failed to load draft: {e}").into()),
    }
}

fn build_graph(
    config: &ProductConfig,
    root: &std::path::Path,
) -> Result<KnowledgeGraph, Box<dyn std::error::Error>> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let deps_dir = config.resolve_path(root, &config.paths.dependencies);
    let loaded =
        parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;
    Ok(KnowledgeGraph::build_with_deps(
        loaded.features, loaded.adrs, loaded.tests, loaded.dependencies,
    ))
}

/// Re-exported for finding display.
#[allow(dead_code)]
pub fn emit_findings(findings: &[Finding]) {
    for f in findings {
        eprintln!("{f}\n");
    }
}
