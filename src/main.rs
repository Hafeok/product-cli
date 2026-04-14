//! Product CLI — knowledge graph for features, ADRs, and test criteria
//!
//! See product-prd.md for full requirements.
//! See product-adrs.md for all architectural decisions.

#![deny(clippy::unwrap_used)]

use product_lib::{author, checklist, config, context, domains, drift, error, fileops, gap, graph, implement, mcp, metrics, migrate, onboard, parser, rdf, types};

use clap::{Parser, Subcommand};
use config::ProductConfig;
use error::ProductError;
use graph::KnowledgeGraph;
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(
    name = "product",
    about = "Knowledge graph CLI for managing features, ADRs, and test criteria",
    version
)]
struct Cli {
    /// Output format: text (default) or json
    #[arg(long, global = true, default_value = "text")]
    format: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Feature navigation and management
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    /// ADR navigation and management
    Adr {
        #[command(subcommand)]
        command: AdrCommands,
    },
    /// Test criterion navigation and management
    Test {
        #[command(subcommand)]
        command: TestCommands,
    },
    /// Assemble context bundles for LLM agents
    Context {
        /// Feature or ADR ID to bundle
        id: String,
        /// BFS traversal depth (default: 1)
        #[arg(long, default_value = "1")]
        depth: usize,
        /// Scope to a phase (bundles all features in that phase)
        #[arg(long)]
        phase: Option<u32>,
        /// Include only ADRs (no test criteria) when using --phase
        #[arg(long)]
        adrs_only: bool,
        /// Order ADRs by ID instead of betweenness centrality
        #[arg(long, value_name = "ORDER")]
        order: Option<String>,
        /// Measure bundle dimensions and write to feature front-matter + metrics.jsonl
        #[arg(long)]
        measure: bool,
    },
    /// Graph operations
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Impact analysis
    Impact {
        /// Artifact ID (feature, ADR, or test)
        id: String,
    },
    /// Status summary
    Status {
        /// Filter to a specific phase
        #[arg(long)]
        phase: Option<u32>,
        /// Show only features with no linked tests
        #[arg(long)]
        untested: bool,
        /// Show only features with failing tests
        #[arg(long)]
        failing: bool,
    },
    /// Checklist generation
    Checklist {
        #[command(subcommand)]
        command: ChecklistCommands,
    },
    /// Generate shell completions
    Completions {
        /// Shell: bash, zsh, fish
        shell: String,
    },
    /// Migration from monolithic documents
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    /// Gap analysis between ADRs, features, and tests
    Gap {
        #[command(subcommand)]
        command: GapCommands,
    },
    /// MCP server (stdio or HTTP transport)
    Mcp {
        /// Use HTTP transport instead of stdio
        #[arg(long)]
        http: bool,
        /// HTTP port (default: 7777)
        #[arg(long, default_value = "7777")]
        port: u16,
        /// HTTP bind address
        #[arg(long, default_value = "127.0.0.1")]
        bind: String,
        /// Bearer token for HTTP auth
        #[arg(long, env = "PRODUCT_MCP_TOKEN")]
        token: Option<String>,
        /// Explicit repo path
        #[arg(long)]
        repo: Option<String>,
        /// Enable write tools (overrides product.toml mcp.write)
        #[arg(long)]
        write: bool,
    },
    /// Implement a feature (gap gate, context assembly, agent invocation)
    Implement {
        /// Feature ID
        id: String,
        /// Inspect context without invoking agent
        #[arg(long)]
        dry_run: bool,
        /// Skip auto-verify after agent completion
        #[arg(long)]
        no_verify: bool,
        /// Run non-interactively via claude -p (no human in the loop)
        #[arg(long)]
        headless: bool,
    },
    /// Verify test criteria for a feature
    Verify {
        /// Feature ID
        id: String,
    },
    /// Start a graph-aware authoring session
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
    },
    /// Install git hooks and scaffolding
    InstallHooks,
    /// Drift detection — spec vs implementation
    Drift {
        #[command(subcommand)]
        command: DriftCommands,
    },
    /// Architectural fitness functions
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
    /// Pre-flight analysis — check domain and cross-cutting coverage
    Preflight {
        /// Feature ID
        id: String,
    },
    /// Codebase onboarding — discover decisions from existing code (ADR-027)
    Onboard {
        #[command(subcommand)]
        command: OnboardCommands,
    },
    /// Initialize a new Product repository (ADR-033)
    Init {
        /// Accept all defaults without prompting
        #[arg(short = 'y', long)]
        yes: bool,
        /// Overwrite existing product.toml
        #[arg(long)]
        force: bool,
        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,
    },
}

#[derive(Subcommand)]
enum GapCommands {
    /// Check for gaps (optionally for a single ADR, or only changed ADRs)
    Check {
        /// ADR ID to check (omit for all)
        adr_id: Option<String>,
        /// Only check ADRs changed in the last commit
        #[arg(long)]
        changed: bool,
        /// Output format: text or json
        #[arg(long, default_value = "json")]
        format: String,
    },
    /// Print a human-readable gap report for all ADRs
    Report,
    /// Suppress a gap finding
    Suppress {
        /// Gap finding ID to suppress
        gap_id: String,
        /// Reason for suppression
        #[arg(long)]
        reason: String,
    },
    /// Remove suppression for a gap finding
    Unsuppress {
        /// Gap finding ID to unsuppress
        gap_id: String,
    },
    /// Print gap analysis statistics
    Stats,
}

#[derive(Subcommand)]
enum AuthorCommands {
    /// Start a feature authoring session
    Feature {
        /// Feature ID (optional — enables preflight gate)
        #[arg(long)]
        feature: Option<String>,
    },
    /// Start an ADR authoring session
    Adr,
    /// Start a spec review session
    Review,
}

#[derive(Subcommand)]
enum FeatureCommands {
    /// List all features
    List {
        #[arg(long)]
        phase: Option<u32>,
        #[arg(long)]
        status: Option<String>,
    },
    /// Show a feature's details
    Show { id: String },
    /// List ADRs linked to a feature
    Adrs { id: String },
    /// List test criteria for a feature
    Tests { id: String },
    /// Show the full dependency tree for a feature
    Deps { id: String },
    /// Show the next feature to implement (topological order)
    Next {
        /// Skip phase gate checks (allow phase-2+ features even if prior gates fail)
        #[arg(long)]
        ignore_phase_gate: bool,
    },
    /// Create a new feature file
    New {
        /// Feature title
        title: String,
        /// Phase number
        #[arg(long, default_value = "1")]
        phase: u32,
    },
    /// Link a feature to an ADR, test, or dependency
    Link {
        /// Feature ID
        id: String,
        /// ADR ID to link
        #[arg(long)]
        adr: Option<String>,
        /// Test ID to link
        #[arg(long)]
        test: Option<String>,
        /// Feature ID this feature depends on
        #[arg(long)]
        dep: Option<String>,
    },
    /// Set feature status
    Status {
        /// Feature ID
        id: String,
        /// New status: planned, in-progress, complete, abandoned
        new_status: String,
    },
    /// Acknowledge a domain or ADR gap with reasoning
    Acknowledge {
        /// Feature ID
        id: String,
        /// Domain to acknowledge
        #[arg(long)]
        domain: Option<String>,
        /// ADR to acknowledge
        #[arg(long)]
        adr: Option<String>,
        /// Reasoning (required)
        #[arg(long)]
        reason: String,
    },
}

#[derive(Subcommand)]
enum AdrCommands {
    /// List all ADRs
    List {
        #[arg(long)]
        status: Option<String>,
    },
    /// Show an ADR's details
    Show { id: String },
    /// List features that reference this ADR
    Features { id: String },
    /// List tests that validate this ADR
    Tests { id: String },
    /// Create a new ADR file
    New {
        /// ADR title
        title: String,
    },
    /// Set ADR status
    Status {
        /// ADR ID
        id: String,
        /// New status: proposed, accepted, superseded, abandoned
        new_status: String,
        /// When setting to superseded, specify the replacement ADR
        #[arg(long)]
        by: Option<String>,
    },
    /// Review staged ADR files
    Review {
        /// Only review staged files (for pre-commit hook)
        #[arg(long)]
        staged: bool,
    },
}

#[derive(Subcommand)]
enum TestCommands {
    /// List all test criteria
    List {
        #[arg(long)]
        phase: Option<u32>,
        #[arg(long = "type")]
        test_type: Option<String>,
        #[arg(long)]
        status: Option<String>,
        /// Show only failing tests
        #[arg(long)]
        failing: bool,
    },
    /// Show a test criterion's details
    Show { id: String },
    /// List features with no linked test criteria
    Untested,
    /// Create a new test criterion file
    New {
        /// Test title
        title: String,
        /// Test type: scenario, invariant, chaos, exit-criteria
        #[arg(long = "type", default_value = "scenario")]
        test_type: String,
    },
    /// Set test criterion status
    Status {
        /// Test ID
        id: String,
        /// New status: unimplemented, implemented, passing, failing
        new_status: String,
    },
}

#[derive(Subcommand)]
enum GraphCommands {
    /// Validate all links and report errors/warnings
    Check {
        /// Output as JSON (for CI)
        #[arg(long)]
        format: Option<String>,
    },
    /// Regenerate index.ttl from all front-matter
    Rebuild,
    /// Execute a SPARQL query over the graph
    Query {
        /// SPARQL query string
        query: String,
    },
    /// Show graph statistics
    Stats,
    /// Show top ADRs by betweenness centrality
    Central {
        /// Number of results
        #[arg(long, default_value = "10")]
        top: usize,
        /// Show all ADRs
        #[arg(long)]
        all: bool,
    },
    /// Auto-link TCs to features via shared ADRs
    Autolink {
        /// Only show what would be linked (don't write)
        #[arg(long)]
        dry_run: bool,
    },
    /// Show feature x domain coverage matrix
    Coverage {
        /// Filter to a specific domain
        #[arg(long)]
        domain: Option<String>,
        /// Output as JSON
        #[arg(long)]
        format: Option<String>,
    },
}

#[derive(Subcommand)]
enum ChecklistCommands {
    /// Regenerate checklist.md from feature files
    Generate,
}

#[derive(Subcommand)]
enum MigrateCommands {
    /// Parse a monolithic PRD into feature files
    FromPrd {
        /// Path to the source PRD markdown file
        source: String,
        /// Only show what would be created, don't write files
        #[arg(long)]
        validate: bool,
        /// Write files (default: dry-run)
        #[arg(long)]
        execute: bool,
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Review each artifact before writing
        #[arg(long)]
        interactive: bool,
    },
    /// Parse a monolithic ADR file into ADR + test files
    FromAdrs {
        /// Path to the source ADR markdown file
        source: String,
        /// Only show what would be created
        #[arg(long)]
        validate: bool,
        /// Write files
        #[arg(long)]
        execute: bool,
        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
        /// Review each artifact before writing
        #[arg(long)]
        interactive: bool,
    },
    /// Upgrade front-matter schema to current version
    Schema {
        /// Show what would change without writing
        #[arg(long)]
        dry_run: bool,
    },
    /// Report what migration would produce without writing
    Validate,
}

#[derive(Subcommand)]
enum DriftCommands {
    /// Check for drift between ADRs and source code
    Check {
        /// ADR ID (optional — checks all if omitted)
        adr_id: Option<String>,
        /// Explicit source files to check
        #[arg(long)]
        files: Vec<String>,
    },
    /// Scan a source file to find governing ADRs
    Scan {
        /// Source file path
        path: String,
    },
    /// Suppress a drift finding
    Suppress {
        drift_id: String,
        #[arg(long)]
        reason: String,
    },
    /// Unsuppress a drift finding
    Unsuppress {
        drift_id: String,
    },
}

#[derive(Subcommand)]
enum MetricsCommands {
    /// Record a metric snapshot to metrics.jsonl
    Record,
    /// Check current metrics against thresholds
    Threshold,
    /// Show metric trends
    Trend {
        /// Metric name (optional — shows all if omitted)
        #[arg(long)]
        metric: Option<String>,
    },
}

#[derive(Subcommand)]
enum OnboardCommands {
    /// Scan a codebase for decision candidates
    Scan {
        /// Path to the source directory to scan
        source: String,
        /// Output file for candidates JSON
        #[arg(long, default_value = "candidates.json")]
        output: String,
        /// Maximum number of candidates to produce
        #[arg(long)]
        max_candidates: Option<usize>,
        /// Disable evidence validation
        #[arg(long)]
        no_validate: bool,
    },
    /// Triage decision candidates (confirm, reject, merge)
    Triage {
        /// Path to candidates.json from scan phase
        source: String,
        /// Interactive triage (reads actions from stdin)
        #[arg(long)]
        interactive: bool,
        /// Output file for triaged candidates
        #[arg(long, default_value = "triaged.json")]
        output: String,
    },
    /// Seed the knowledge graph from triaged candidates
    Seed {
        /// Path to triaged.json from triage phase
        source: String,
        /// Show what would be created without writing files
        #[arg(long)]
        dry_run: bool,
    },
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

fn main() {
    // Handle SIGPIPE gracefully — exit silently when piped to `head` etc.
    #[cfg(unix)]
    {
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_DFL);
        }
    }

    let cli = Cli::parse();

    let result = run(cli);
    if let Err(e) = result {
        eprintln!("{}", e);
        process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    // Clean up any leftover tmp files from prior crashes (ADR-015)
    if let Ok((config, root)) = ProductConfig::discover() {
        let dirs = [
            config.resolve_path(&root, &config.paths.features),
            config.resolve_path(&root, &config.paths.adrs),
            config.resolve_path(&root, &config.paths.tests),
        ];
        for dir in &dirs {
            fileops::cleanup_tmp_files(dir);
        }
    }

    let fmt = &cli.format;
    match cli.command {
        Commands::Feature { command } => handle_feature(command, fmt),
        Commands::Adr { command } => handle_adr(command, fmt),
        Commands::Test { command } => handle_test(command, fmt),
        Commands::Context {
            id,
            depth,
            phase,
            adrs_only,
            order,
            measure,
        } => handle_context(&id, depth, phase, adrs_only, order, measure),
        Commands::Graph { command } => handle_graph(command, fmt),
        Commands::Impact { id } => handle_impact(&id, fmt),
        Commands::Status {
            phase,
            untested,
            failing,
        } => handle_status(phase, untested, failing, fmt),
        Commands::Checklist { command } => handle_checklist(command),
        Commands::Completions { shell } => handle_completions(&shell),
        Commands::Migrate { command } => handle_migrate(command),
        Commands::Gap { command } => handle_gap(command, fmt),
        Commands::Implement { id, dry_run, no_verify, headless } => handle_implement(&id, dry_run, no_verify, headless),
        Commands::Verify { id } => handle_verify(&id),
        Commands::Author { command } => handle_author(command),
        Commands::Mcp { http, port, bind, token, repo, write } => handle_mcp(http, port, &bind, token, repo, write),
        Commands::InstallHooks => handle_install_hooks(),
        Commands::Drift { command } => handle_drift(command, fmt),
        Commands::Metrics { command } => handle_metrics(command),
        Commands::Preflight { id } => handle_preflight(&id),
        Commands::Onboard { command } => handle_onboard(command),
        Commands::Init { yes, force, name } => handle_init(yes, force, name),
    }
}

type BoxResult = Result<(), Box<dyn std::error::Error>>;

/// Acquire the advisory lock for write operations (ADR-015).
/// Returns a `RepoLock` that must be held for the duration of the write.
fn acquire_write_lock() -> Result<fileops::RepoLock, Box<dyn std::error::Error>> {
    let (_, root) = ProductConfig::discover()?;
    Ok(fileops::RepoLock::acquire(&root)?)
}

fn load_graph() -> Result<(ProductConfig, PathBuf, KnowledgeGraph), Box<dyn std::error::Error>> {
    let (config, root) = ProductConfig::discover()?;

    // Check schema version
    for warning in config.check_schema_version()? {
        eprintln!("{}", warning);
    }

    let features_dir = config.resolve_path(&root, &config.paths.features);
    let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
    let tests_dir = config.resolve_path(&root, &config.paths.tests);

    let loaded = parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;

    // Print parse errors to stderr so they are visible for all commands (ADR-013)
    for e in &loaded.parse_errors {
        eprintln!("{}", e);
    }

    let graph = KnowledgeGraph::build(loaded.features, loaded.adrs, loaded.tests)
        .with_parse_errors(loaded.parse_errors);
    Ok((config, root, graph))
}

// ---------------------------------------------------------------------------
// Feature commands
// ---------------------------------------------------------------------------

fn handle_feature(cmd: FeatureCommands, fmt: &str) -> BoxResult {
    match cmd {
        FeatureCommands::List { phase, status } => {
            let (_, _, graph) = load_graph()?;
            let mut features: Vec<&types::Feature> = graph.features.values().collect();
            features.sort_by_key(|f| &f.front.id);

            if let Some(p) = phase {
                features.retain(|f| f.front.phase == p);
            }
            if let Some(ref s) = status {
                let target: types::FeatureStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
                features.retain(|f| f.front.status == target);
            }

            if fmt == "json" {
                let arr: Vec<serde_json::Value> = features
                    .iter()
                    .map(|f| {
                        serde_json::json!({
                            "id": f.front.id,
                            "phase": f.front.phase,
                            "status": f.front.status.to_string(),
                            "title": f.front.title,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
            } else {
                println!("{:<10} {:<8} {:<15} TITLE", "ID", "PHASE", "STATUS");
                println!("{}", "-".repeat(60));
                for f in &features {
                    println!(
                        "{:<10} {:<8} {:<15} {}",
                        f.front.id, f.front.phase, f.front.status, f.front.title
                    );
                }
            }
        }
        FeatureCommands::Show { id } => {
            let (_, _, graph) = load_graph()?;
            let f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
            if fmt == "json" {
                let obj = serde_json::json!({
                    "id": f.front.id,
                    "title": f.front.title,
                    "phase": f.front.phase,
                    "status": f.front.status.to_string(),
                    "depends_on": f.front.depends_on,
                    "adrs": f.front.adrs,
                    "tests": f.front.tests,
                    "body": f.body,
                });
                println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
            } else {
                println!("# {} — {}\n", f.front.id, f.front.title);
                println!("Phase:      {}", f.front.phase);
                println!("Status:     {}", f.front.status);
                println!("Depends-on: {}", if f.front.depends_on.is_empty() { "(none)".to_string() } else { f.front.depends_on.join(", ") });
                println!("ADRs:       {}", if f.front.adrs.is_empty() { "(none)".to_string() } else { f.front.adrs.join(", ") });
                println!("Tests:      {}", if f.front.tests.is_empty() { "(none)".to_string() } else { f.front.tests.join(", ") });
                println!("\n{}", f.body);
            }
        }
        FeatureCommands::Adrs { id } => {
            let (_, _, graph) = load_graph()?;
            let f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
            println!("ADRs linked to {}:", id);
            for adr_id in &f.front.adrs {
                if let Some(adr) = graph.adrs.get(adr_id.as_str()) {
                    println!("  {} — {} ({})", adr.front.id, adr.front.title, adr.front.status);
                } else {
                    println!("  {} (broken link)", adr_id);
                }
            }
        }
        FeatureCommands::Tests { id } => {
            let (_, _, graph) = load_graph()?;
            let f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
            println!("Tests linked to {}:", id);
            for test_id in &f.front.tests {
                if let Some(tc) = graph.tests.get(test_id.as_str()) {
                    println!(
                        "  {} — {} ({}, {})",
                        tc.front.id, tc.front.title, tc.front.test_type, tc.front.status
                    );
                } else {
                    println!("  {} (broken link)", test_id);
                }
            }
        }
        FeatureCommands::Deps { id } => {
            let (_, _, graph) = load_graph()?;
            let _f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;
            println!("Dependency tree for {}:", id);
            print_dep_tree(&graph, &id, 0, &mut std::collections::HashSet::new());
        }
        FeatureCommands::Next { ignore_phase_gate } => {
            let (_, _, graph) = load_graph()?;
            match graph.feature_next_with_gate(ignore_phase_gate)? {
                graph::FeatureNextResult::Ready(id) => {
                    if ignore_phase_gate {
                        eprintln!("warning: --ignore-phase-gate: phase gate checks skipped");
                    }
                    let f = &graph.features[&id];
                    println!("{} — {} (phase {}, {})", f.front.id, f.front.title, f.front.phase, f.front.status);
                }
                graph::FeatureNextResult::Blocked { candidate, blocked_phase, exit_criteria } => {
                    let f = &graph.features[&candidate];
                    println!(
                        "  Next candidate: {} — {}  [phase {}, {}]",
                        f.front.id, f.front.title, f.front.phase, f.front.status
                    );
                    let failing: Vec<&graph::PhaseGateTC> = exit_criteria.iter().filter(|tc| !tc.passing).collect();
                    eprintln!(
                        "  ✗ Phase {} locked — Phase {} exit criteria not all passing:",
                        f.front.phase, blocked_phase
                    );
                    eprintln!();
                    for tc in &exit_criteria {
                        let mark = if tc.passing { "passing  ✓" } else { "failing  ✗" };
                        eprintln!("    {}  {}  [{}]", tc.id, tc.title, mark);
                    }
                    eprintln!();
                    let failing_ids: Vec<&str> = failing.iter().map(|tc| tc.id.as_str()).collect();
                    eprintln!("  Fix {} to unlock Phase {}.", failing_ids.join(" and "), f.front.phase);
                    eprintln!("  To skip the gate:  product feature next --ignore-phase-gate");
                }
                graph::FeatureNextResult::AllDone => {
                    println!("All features are complete or have incomplete dependencies.");
                }
            }
        }
        FeatureCommands::New { title, phase } => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            let existing: Vec<String> = graph.features.keys().cloned().collect();
            let id = parser::next_id(&config.prefixes.feature, &existing);
            let filename = parser::id_to_filename(&id, &title);
            let dir = config.resolve_path(&root, &config.paths.features);
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(&filename);

            let front = types::FeatureFrontMatter {
                id: id.clone(),
                title: title.clone(),
                phase,
                status: types::FeatureStatus::Planned,
                depends_on: vec![],
                adrs: vec![],
                tests: vec![],
                domains: vec![],
                domains_acknowledged: std::collections::HashMap::new(),
                bundle: None,
            };
            let body = format!("## Description\n\n[Describe {} here.]\n", title);
            let content = parser::render_feature(&front, &body);
            fileops::write_file_atomic(&path, &content)?;
            println!("Created: {} at {}", id, path.display());
        }
        FeatureCommands::Link { id, adr, test, dep } => {
            let _lock = acquire_write_lock()?;
            let (_config, _root, graph) = load_graph()?;
            let f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

            let mut front = f.front.clone();
            let mut changed = false;

            if let Some(adr_id) = adr {
                if !front.adrs.contains(&adr_id) {
                    front.adrs.push(adr_id.clone());
                    changed = true;
                    println!("Linked {} -> {}", id, adr_id);
                } else {
                    println!("{} already linked to {}", id, adr_id);
                }
            }
            if let Some(test_id) = test {
                if !front.tests.contains(&test_id) {
                    front.tests.push(test_id.clone());
                    changed = true;
                    println!("Linked {} -> {}", id, test_id);
                } else {
                    println!("{} already linked to {}", id, test_id);
                }
            }
            if let Some(dep_id) = dep {
                if !graph.features.contains_key(&dep_id) {
                    return Err(Box::new(ProductError::NotFound(format!("feature {}", dep_id))));
                }
                if !front.depends_on.contains(&dep_id) {
                    // Validate no cycle would be introduced
                    front.depends_on.push(dep_id.clone());
                    let mut test_features: Vec<types::Feature> = graph.features.values().cloned().collect();
                    // Replace the feature with our modified version for cycle check
                    test_features.retain(|tf| tf.front.id != id);
                    test_features.push(types::Feature {
                        front: front.clone(),
                        body: f.body.clone(),
                        path: f.path.clone(),
                    });
                    let test_graph = graph::KnowledgeGraph::build(test_features, vec![], vec![]);
                    if let Err(ProductError::DependencyCycle { cycle }) = test_graph.topological_sort() {
                        front.depends_on.retain(|d| d != &dep_id);
                        return Err(Box::new(ProductError::DependencyCycle { cycle }));
                    }
                    changed = true;
                    println!("Linked {} depends-on {}", id, dep_id);
                } else {
                    println!("{} already depends on {}", id, dep_id);
                }
            }

            if changed {
                let content = parser::render_feature(&front, &f.body);
                fileops::write_file_atomic(&f.path, &content)?;
            }
        }
        FeatureCommands::Status { id, new_status } => {
            let _lock = acquire_write_lock()?;
            let (_, _, graph) = load_graph()?;
            let f = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

            let status: types::FeatureStatus = new_status
                .parse()
                .map_err(|e: String| ProductError::ConfigError(e))?;

            let mut front = f.front.clone();
            front.status = status;
            let content = parser::render_feature(&front, &f.body);
            fileops::write_file_atomic(&f.path, &content)?;
            println!("{} status -> {}", id, status);

            // ADR-010: Auto-orphan tests on feature abandonment
            if status == types::FeatureStatus::Abandoned {
                println!("Auto-orphaning test criteria linked to abandoned feature:");
                for test_id in &f.front.tests {
                    if let Some(tc) = graph.tests.get(test_id.as_str()) {
                        let mut test_front = tc.front.clone();
                        test_front.validates.features.retain(|fid| fid != &id);
                        let test_content = parser::render_test(&test_front, &tc.body);
                        fileops::write_file_atomic(&tc.path, &test_content)?;
                        println!("  {} — removed {} from validates.features", test_id, id);
                    }
                }
            }
        }
        FeatureCommands::Acknowledge { id, domain, adr, reason } => {
            let _lock = acquire_write_lock()?;
            let (_, _, graph) = load_graph()?;
            let feature = graph
                .features
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("feature {}", id)))?;

            let updated_front = if let Some(ref domain_name) = domain {
                domains::acknowledge_domain(feature, domain_name, &reason)?
            } else if let Some(ref adr_id) = adr {
                let adr_obj = graph
                    .adrs
                    .get(adr_id.as_str())
                    .ok_or_else(|| ProductError::NotFound(format!("ADR {}", adr_id)))?;
                domains::acknowledge_adr(feature, adr_obj, &reason)?
            } else {
                return Err(Box::new(ProductError::ConfigError(
                    "must specify --domain or --adr".to_string(),
                )));
            };

            let content = parser::render_feature(&updated_front, &feature.body);
            fileops::write_file_atomic(&feature.path, &content)?;
            if let Some(ref d) = domain {
                println!("{} acknowledged domain '{}': {}", id, d, reason);
            } else if let Some(ref a) = adr {
                println!("{} acknowledged ADR '{}': {}", id, a, reason);
            }
        }
    }
    Ok(())
}

fn print_dep_tree(
    graph: &KnowledgeGraph,
    id: &str,
    indent: usize,
    visited: &mut std::collections::HashSet<String>,
) {
    if visited.contains(id) {
        println!("{}  {} (circular)", "  ".repeat(indent), id);
        return;
    }
    visited.insert(id.to_string());

    if let Some(f) = graph.features.get(id) {
        let marker = match f.front.status {
            types::FeatureStatus::Complete => "[x]",
            types::FeatureStatus::InProgress => "[~]",
            types::FeatureStatus::Planned => "[ ]",
            types::FeatureStatus::Abandoned => "[-]",
        };
        println!(
            "{}{} {} — {}",
            "  ".repeat(indent),
            marker,
            f.front.id,
            f.front.title
        );
        for dep in &f.front.depends_on {
            print_dep_tree(graph, dep, indent + 1, visited);
        }
    }
}

// ---------------------------------------------------------------------------
// ADR commands
// ---------------------------------------------------------------------------

fn handle_adr(cmd: AdrCommands, fmt: &str) -> BoxResult {
    match cmd {
        AdrCommands::List { status } => {
            let (_, _, graph) = load_graph()?;
            let mut adrs: Vec<&types::Adr> = graph.adrs.values().collect();
            adrs.sort_by_key(|a| &a.front.id);

            if let Some(ref s) = status {
                let target: types::AdrStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
                adrs.retain(|a| a.front.status == target);
            }

            if fmt == "json" {
                let arr: Vec<serde_json::Value> = adrs
                    .iter()
                    .map(|a| {
                        serde_json::json!({
                            "id": a.front.id,
                            "status": a.front.status.to_string(),
                            "title": a.front.title,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
            } else {
                println!("{:<10} {:<15} TITLE", "ID", "STATUS");
                println!("{}", "-".repeat(60));
                for a in &adrs {
                    println!("{:<10} {:<15} {}", a.front.id, a.front.status, a.front.title);
                }
            }
        }
        AdrCommands::Show { id } => {
            let (_, _, graph) = load_graph()?;
            let a = graph
                .adrs
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;
            if fmt == "json" {
                let obj = serde_json::json!({
                    "id": a.front.id,
                    "title": a.front.title,
                    "status": a.front.status.to_string(),
                    "features": a.front.features,
                    "supersedes": a.front.supersedes,
                    "superseded_by": a.front.superseded_by,
                    "body": a.body,
                });
                println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
            } else {
                println!("# {} — {}\n", a.front.id, a.front.title);
                println!("Status:        {}", a.front.status);
                println!("Features:      {}", if a.front.features.is_empty() { "(none)".to_string() } else { a.front.features.join(", ") });
                println!("Supersedes:    {}", if a.front.supersedes.is_empty() { "(none)".to_string() } else { a.front.supersedes.join(", ") });
                println!("Superseded-by: {}", if a.front.superseded_by.is_empty() { "(none)".to_string() } else { a.front.superseded_by.join(", ") });
                println!("\n{}", a.body);
            }
        }
        AdrCommands::Features { id } => {
            let (_, _, graph) = load_graph()?;
            println!("Features referencing {}:", id);
            for f in graph.features.values() {
                if f.front.adrs.contains(&id) {
                    println!("  {} — {} ({})", f.front.id, f.front.title, f.front.status);
                }
            }
        }
        AdrCommands::Tests { id } => {
            let (_, _, graph) = load_graph()?;
            println!("Tests validating {}:", id);
            for t in graph.tests.values() {
                if t.front.validates.adrs.contains(&id) {
                    println!(
                        "  {} — {} ({}, {})",
                        t.front.id, t.front.title, t.front.test_type, t.front.status
                    );
                }
            }
        }
        AdrCommands::New { title } => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            let existing: Vec<String> = graph.adrs.keys().cloned().collect();
            let id = parser::next_id(&config.prefixes.adr, &existing);
            let filename = parser::id_to_filename(&id, &title);
            let dir = config.resolve_path(&root, &config.paths.adrs);
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(&filename);

            let front = types::AdrFrontMatter {
                id: id.clone(),
                title: title.clone(),
                status: types::AdrStatus::Proposed,
                features: vec![],
                supersedes: vec![],
                superseded_by: vec![],
                domains: vec![],
                scope: types::AdrScope::Domain,
            };
            let body = "**Status:** Proposed\n\n**Context:**\n\n[Describe the context here.]\n\n**Decision:**\n\n[Describe the decision.]\n\n**Rationale:**\n\n[Explain why.]\n\n**Rejected alternatives:**\n\n- [Alternative 1]\n".to_string();
            let content = parser::render_adr(&front, &body);
            fileops::write_file_atomic(&path, &content)?;
            println!("Created: {} at {}", id, path.display());
        }
        AdrCommands::Status {
            id,
            new_status,
            by,
        } => {
            let _lock = acquire_write_lock()?;
            let (_, _, graph) = load_graph()?;
            let a = graph
                .adrs
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;

            let status: types::AdrStatus = new_status
                .parse()
                .map_err(|e: String| ProductError::ConfigError(e))?;

            // If superseding, show impact first
            if status == types::AdrStatus::Superseded {
                let impact = graph.impact(&id);
                impact.print(&graph);
                println!();
            }

            let mut front = a.front.clone();
            front.status = status;
            if let Some(by_id) = by {
                if !front.superseded_by.contains(&by_id) {
                    front.superseded_by.push(by_id.clone());
                }
                // Also update the successor to list this in supersedes
                if let Some(successor) = graph.adrs.get(&by_id) {
                    let mut succ_front = successor.front.clone();
                    if !succ_front.supersedes.contains(&id) {
                        succ_front.supersedes.push(id.clone());
                    }
                    let succ_content = parser::render_adr(&succ_front, &successor.body);
                    fileops::write_file_atomic(&successor.path, &succ_content)?;
                }
            }

            let content = parser::render_adr(&front, &a.body);
            fileops::write_file_atomic(&a.path, &content)?;
            println!("{} status -> {}", id, status);
        }
        AdrCommands::Review { staged } => {
            if staged {
                let (_, root, _) = load_graph()?;
                let warnings = author::review_staged(&root)?;
                for w in &warnings {
                    eprintln!("{}", w);
                }
                if !warnings.is_empty() {
                    eprintln!("{} ADR review warning(s)", warnings.len());
                }
            } else {
                eprintln!("Use --staged to review staged ADR files.");
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Test commands
// ---------------------------------------------------------------------------

fn handle_test(cmd: TestCommands, fmt: &str) -> BoxResult {
    match cmd {
        TestCommands::List {
            phase,
            test_type,
            status,
            failing,
        } => {
            let (_, _, graph) = load_graph()?;
            let mut tests: Vec<&types::TestCriterion> = graph.tests.values().collect();
            tests.sort_by_key(|t| &t.front.id);

            if let Some(p) = phase {
                tests.retain(|t| t.front.phase == p);
            }
            if let Some(ref tt) = test_type {
                let target: types::TestType = tt.parse().map_err(|e: String| ProductError::ConfigError(e))?;
                tests.retain(|t| t.front.test_type == target);
            }
            if failing {
                tests.retain(|t| t.front.status == types::TestStatus::Failing);
            } else if let Some(ref s) = status {
                let target: types::TestStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
                tests.retain(|t| t.front.status == target);
            }

            if fmt == "json" {
                let arr: Vec<serde_json::Value> = tests
                    .iter()
                    .map(|t| {
                        serde_json::json!({
                            "id": t.front.id,
                            "phase": t.front.phase,
                            "type": t.front.test_type.to_string(),
                            "status": t.front.status.to_string(),
                            "title": t.front.title,
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
            } else {
                println!(
                    "{:<10} {:<8} {:<15} {:<15} TITLE",
                    "ID", "PHASE", "TYPE", "STATUS"
                );
                println!("{}", "-".repeat(70));
                for t in &tests {
                    println!(
                        "{:<10} {:<8} {:<15} {:<15} {}",
                        t.front.id, t.front.phase, t.front.test_type, t.front.status, t.front.title
                    );
                }
            }
        }
        TestCommands::Show { id } => {
            let (_, _, graph) = load_graph()?;
            let t = graph
                .tests
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("test {}", id)))?;
            if fmt == "json" {
                let obj = serde_json::json!({
                    "id": t.front.id,
                    "title": t.front.title,
                    "type": t.front.test_type.to_string(),
                    "status": t.front.status.to_string(),
                    "phase": t.front.phase,
                    "validates": {
                        "features": t.front.validates.features,
                        "adrs": t.front.validates.adrs,
                    },
                    "body": t.body,
                });
                println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
            } else {
                println!("# {} — {}\n", t.front.id, t.front.title);
                println!("Type:     {}", t.front.test_type);
                println!("Status:   {}", t.front.status);
                println!("Phase:    {}", t.front.phase);
                println!(
                    "Features: {}",
                    if t.front.validates.features.is_empty() {
                        "(none)".to_string()
                    } else {
                        t.front.validates.features.join(", ")
                    }
                );
                println!(
                    "ADRs:     {}",
                    if t.front.validates.adrs.is_empty() {
                        "(none)".to_string()
                    } else {
                        t.front.validates.adrs.join(", ")
                    }
                );
                println!("\n{}", t.body);
            }
        }
        TestCommands::Untested => {
            let (_, _, graph) = load_graph()?;
            println!("Features with no linked test criteria:");
            let mut found = false;
            for f in graph.features.values() {
                if f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty() {
                    println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
                    found = true;
                }
            }
            if !found {
                println!("  (none — all features have linked tests)");
            }
        }
        TestCommands::New { title, test_type } => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            let existing: Vec<String> = graph.tests.keys().cloned().collect();
            let id = parser::next_id(&config.prefixes.test, &existing);
            let filename = parser::id_to_filename(&id, &title);
            let dir = config.resolve_path(&root, &config.paths.tests);
            std::fs::create_dir_all(&dir)?;
            let path = dir.join(&filename);

            let tt: types::TestType = test_type
                .parse()
                .map_err(|e: String| ProductError::ConfigError(e))?;

            let front = types::TestFrontMatter {
                id: id.clone(),
                title: title.clone(),
                test_type: tt,
                status: types::TestStatus::Unimplemented,
                validates: types::ValidatesBlock {
                    features: vec![],
                    adrs: vec![],
                },
                phase: 1,
            };

            let body = "## Description\n\n[Describe the test criterion here.]\n".to_string();
            let content = parser::render_test(&front, &body);
            fileops::write_file_atomic(&path, &content)?;
            println!("Created: {} at {}", id, path.display());
        }
        TestCommands::Status { id, new_status } => {
            let _lock = acquire_write_lock()?;
            let (_, _, graph) = load_graph()?;
            let t = graph
                .tests
                .get(&id)
                .ok_or_else(|| ProductError::NotFound(format!("test {}", id)))?;

            let status: types::TestStatus = new_status
                .parse()
                .map_err(|e: String| ProductError::ConfigError(e))?;

            let mut front = t.front.clone();
            front.status = status;
            let content = parser::render_test(&front, &t.body);
            fileops::write_file_atomic(&t.path, &content)?;
            println!("{} status -> {}", id, status);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Context command
// ---------------------------------------------------------------------------

fn handle_context(
    id: &str,
    depth: usize,
    phase: Option<u32>,
    adrs_only: bool,
    order: Option<String>,
    measure: bool,
) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let order_by_centrality = order.as_deref() != Some("id");

    if let Some(p) = phase {
        let bundle = context::bundle_phase(&graph, p, depth, adrs_only, order_by_centrality);
        print!("{}", bundle);
    } else if graph.features.contains_key(id) {
        match context::bundle_feature(&graph, id, depth, order_by_centrality) {
            Some(bundle) => {
                if measure {
                    // Compute bundle metrics
                    let feature = graph.features.get(id).ok_or_else(|| {
                        ProductError::NotFound(format!("feature {}", id))
                    })?;

                    // Count depth-1 ADRs (only direct ADRs)
                    let depth_1_adrs = feature.front.adrs.len();
                    let tcs = feature.front.tests.len();
                    let domains = feature.front.domains.clone();
                    // Approximate token count: ~4 chars per token is a reasonable estimate
                    let tokens_approx = bundle.len() / 4;
                    let measured_at = chrono::Utc::now().to_rfc3339();

                    let bundle_metrics = types::BundleMetrics {
                        depth_1_adrs,
                        tcs,
                        domains: domains.clone(),
                        tokens_approx,
                        measured_at: measured_at.clone(),
                    };

                    // Update feature front-matter with bundle metrics
                    let mut front = feature.front.clone();
                    front.bundle = Some(bundle_metrics.clone());
                    let content = parser::render_feature(&front, &feature.body);
                    fileops::write_file_atomic(&feature.path, &content)?;

                    // Append to metrics.jsonl
                    let (config, root, _) = load_graph()?;
                    let metrics_path = root.join("metrics.jsonl");
                    let entry = serde_json::json!({
                        "feature": id,
                        "depth-1-adrs": bundle_metrics.depth_1_adrs,
                        "tcs": bundle_metrics.tcs,
                        "domains": bundle_metrics.domains,
                        "tokens-approx": bundle_metrics.tokens_approx,
                        "measured-at": bundle_metrics.measured_at,
                    });
                    let mut line = serde_json::to_string(&entry).unwrap_or_default();
                    line.push('\n');
                    let mut file = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&metrics_path)?;
                    std::io::Write::write_all(&mut file, line.as_bytes())?;

                    let _ = config; // suppress unused warning
                }
                print!("{}", bundle);
            }
            None => eprintln!("Feature {} not found", id),
        }
    } else if graph.adrs.contains_key(id) {
        match context::bundle_adr(&graph, id, depth) {
            Some(bundle) => print!("{}", bundle),
            None => eprintln!("ADR {} not found", id),
        }
    } else {
        eprintln!("Artifact {} not found", id);
        process::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Graph commands
// ---------------------------------------------------------------------------

fn handle_graph(cmd: GraphCommands, global_format: &str) -> BoxResult {
    match cmd {
        GraphCommands::Check { format } => {
            let (config, _, graph) = load_graph()?;
            let mut result = graph.check();
            domains::validate_domains(&graph, &config.domains, &mut result.errors, &mut result.warnings);
            let fmt = format.as_deref().unwrap_or(global_format);

            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&result.to_json())?);
                let code = result.exit_code();
                if code != 0 {
                    process::exit(code);
                }
            } else {
                result.print_stderr();
                let code = result.exit_code();
                match code {
                    0 => eprintln!("Graph check: clean (no errors, no warnings)"),
                    1 => eprintln!("Graph check: {} error(s)", result.errors.len()),
                    2 => eprintln!("Graph check: {} warning(s)", result.warnings.len()),
                    _ => {}
                }
                process::exit(code);
            }
        }
        GraphCommands::Rebuild => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            let graph_dir = config.resolve_path(&root, &config.paths.graph);
            std::fs::create_dir_all(&graph_dir)?;
            let path = graph_dir.join("index.ttl");
            rdf::write_index_ttl(&graph, &path)?;
            println!("Wrote {}", path.display());
        }
        GraphCommands::Query { query } => {
            let (_, _, graph) = load_graph()?;
            let result = rdf::sparql_query(&graph, &query)?;
            print!("{}", result);
        }
        GraphCommands::Stats => {
            let start = std::time::Instant::now();
            let (_, _, graph) = load_graph()?;
            let parse_time = start.elapsed();

            let centrality_start = std::time::Instant::now();
            let stats = graph.stats();
            let centrality_time = centrality_start.elapsed();

            let total_time = start.elapsed();

            // Link density: edges / (nodes * (nodes - 1)), 0 if < 2 nodes
            let link_density = if stats.total_nodes > 1 {
                stats.total_edges as f64 / (stats.total_nodes * (stats.total_nodes - 1)) as f64
            } else {
                0.0
            };

            println!("Graph Statistics");
            println!("================");
            println!("  Features:      {}", stats.features);
            println!("  ADRs:          {}", stats.adrs);
            println!("  Tests:         {}", stats.tests);
            println!("  Total nodes:   {}", stats.total_nodes);
            println!("  Total edges:   {}", stats.total_edges);
            println!("  Link density:  {:.3}", link_density);
            println!("  Formal coverage (invariant/chaos): {}%", stats.formal_coverage);
            println!();
            println!("  Timing:");
            println!("    Parse:      {:.1}ms", parse_time.as_secs_f64() * 1000.0);
            println!("    Centrality: {:.1}ms", centrality_time.as_secs_f64() * 1000.0);
            println!("    Total:      {:.1}ms", total_time.as_secs_f64() * 1000.0);

            if !stats.adr_centrality.is_empty() {
                let mut sorted: Vec<_> = stats.adr_centrality.clone();
                sorted.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
                let max = sorted.first().map(|(_, c)| *c).unwrap_or(0.0);
                let min = sorted.last().map(|(_, c)| *c).unwrap_or(0.0);
                let mean: f64 =
                    sorted.iter().map(|(_, c)| c).sum::<f64>() / sorted.len().max(1) as f64;
                println!();
                println!(
                    "  ADR centrality: mean={:.3}, max={:.3}, min={:.3}",
                    mean, max, min
                );

                let hubs: Vec<_> = sorted
                    .iter()
                    .filter(|(_, c)| *c > 0.5)
                    .map(|(id, _)| id.as_str())
                    .collect();
                if !hubs.is_empty() {
                    println!("  Structural hubs (>0.5): {}", hubs.join(", "));
                }
            }
        }
        GraphCommands::Central { top, all } => {
            let (_, _, graph) = load_graph()?;
            let centrality = graph.betweenness_centrality();
            let mut adr_centrality: Vec<(String, f64)> = graph
                .adrs
                .keys()
                .map(|id| (id.clone(), centrality.get(id).copied().unwrap_or(0.0)))
                .collect();
            adr_centrality
                .sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let limit = if all { adr_centrality.len() } else { top.min(adr_centrality.len()) };
            println!(
                "{:<6} {:<10} {:<12} TITLE",
                "RANK", "ID", "CENTRALITY"
            );
            println!("{}", "-".repeat(60));
            for (i, (id, c)) in adr_centrality.iter().take(limit).enumerate() {
                let title = graph
                    .adrs
                    .get(id)
                    .map(|a| a.front.title.as_str())
                    .unwrap_or("");
                println!("{:<6} {:<10} {:<12.3} {}", i + 1, id, c, title);
            }
        }
        GraphCommands::Autolink { dry_run } => {
            let _lock = acquire_write_lock()?;
            let (_, _, graph) = load_graph()?;

            // Build a map: ADR ID -> list of feature IDs that link to it
            let mut adr_to_features: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new();
            for f in graph.features.values() {
                for adr_id in &f.front.adrs {
                    adr_to_features
                        .entry(adr_id.clone())
                        .or_default()
                        .push(f.front.id.clone());
                }
            }

            // For each TC that validates an ADR, find which features share that ADR
            let mut feature_adds: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new(); // feature_id -> new test IDs
            let mut tc_adds: std::collections::HashMap<String, Vec<String>> =
                std::collections::HashMap::new(); // tc_id -> new feature IDs

            for tc in graph.tests.values() {
                for adr_id in &tc.front.validates.adrs {
                    if let Some(feature_ids) = adr_to_features.get(adr_id) {
                        for fid in feature_ids {
                            // Add TC to feature's tests list (if not already there)
                            if let Some(f) = graph.features.get(fid) {
                                if !f.front.tests.contains(&tc.front.id) {
                                    feature_adds
                                        .entry(fid.clone())
                                        .or_default()
                                        .push(tc.front.id.clone());
                                }
                            }
                            // Add feature to TC's validates.features (if not already there)
                            if !tc.front.validates.features.contains(fid) {
                                tc_adds
                                    .entry(tc.front.id.clone())
                                    .or_default()
                                    .push(fid.clone());
                            }
                        }
                    }
                }
            }

            // Deduplicate
            for v in feature_adds.values_mut() {
                v.sort();
                v.dedup();
            }
            for v in tc_adds.values_mut() {
                v.sort();
                v.dedup();
            }

            let total_feature_links: usize = feature_adds.values().map(|v| v.len()).sum();
            let total_tc_links: usize = tc_adds.values().map(|v| v.len()).sum();

            if total_feature_links == 0 && total_tc_links == 0 {
                println!("No new links to add. Graph is already fully connected via ADRs.");
                return Ok(());
            }

            println!(
                "Autolink: {} TC→Feature links, {} Feature→TC links to add",
                total_tc_links, total_feature_links
            );

            if dry_run {
                println!();
                let mut sorted_features: Vec<_> = feature_adds.iter().collect();
                sorted_features.sort_by_key(|(k, _)| (*k).clone());
                for (fid, tcs) in &sorted_features {
                    println!("  {} += tests: [{}]", fid, tcs.join(", "));
                }
                println!();
                println!("Run without --dry-run to write these links.");
                return Ok(());
            }

            // Write feature files
            let mut features_written = 0;
            for (fid, new_tcs) in &feature_adds {
                if let Some(f) = graph.features.get(fid) {
                    let mut front = f.front.clone();
                    for tc_id in new_tcs {
                        if !front.tests.contains(tc_id) {
                            front.tests.push(tc_id.clone());
                        }
                    }
                    front.tests.sort();
                    let content = parser::render_feature(&front, &f.body);
                    fileops::write_file_atomic(&f.path, &content)?;
                    features_written += 1;
                }
            }

            // Write TC files
            let mut tcs_written = 0;
            for (tc_id, new_features) in &tc_adds {
                if let Some(tc) = graph.tests.get(tc_id) {
                    let mut front = tc.front.clone();
                    for fid in new_features {
                        if !front.validates.features.contains(fid) {
                            front.validates.features.push(fid.clone());
                        }
                    }
                    front.validates.features.sort();
                    let content = parser::render_test(&front, &tc.body);
                    fileops::write_file_atomic(&tc.path, &content)?;
                    tcs_written += 1;
                }
            }

            println!(
                "  Updated {} feature files, {} TC files",
                features_written, tcs_written
            );
        }
        GraphCommands::Coverage { domain, format } => {
            let (config, _, graph) = load_graph()?;
            let matrix = domains::build_coverage_matrix(&graph, &config.domains);
            let fmt = format.as_deref().unwrap_or(global_format);
            if fmt == "json" {
                let json = domains::coverage_matrix_to_json(&matrix);
                println!("{}", serde_json::to_string_pretty(&json).unwrap_or_default());
            } else {
                    print!("{}", domains::render_coverage_matrix_filtered(&matrix, &graph, domain.as_deref()));
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Impact command
// ---------------------------------------------------------------------------

fn handle_impact(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    if !graph.all_ids().contains(id) {
        return Err(Box::new(ProductError::NotFound(format!("artifact {}", id))));
    }
    let impact = graph.impact(id);
    if fmt == "json" {
        let obj = serde_json::json!({
            "seed": impact.seed,
            "direct_features": impact.direct_features,
            "direct_tests": impact.direct_tests,
            "transitive_features": impact.transitive_features,
            "transitive_tests": impact.transitive_tests,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        impact.print(&graph);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Status command
// ---------------------------------------------------------------------------

fn handle_status(phase: Option<u32>, untested: bool, failing: bool, fmt: &str) -> BoxResult {
    let (config, _, graph) = load_graph()?;

    if untested {
        let items: Vec<&types::Feature> = graph
            .features
            .values()
            .filter(|f| f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty())
            .collect();
        if fmt == "json" {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "id": f.front.id,
                        "title": f.front.title,
                        "phase": f.front.phase,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
        } else {
            println!("Features with no linked test criteria:");
            for f in &items {
                println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
            }
        }
        return Ok(());
    }

    if failing {
        let items: Vec<&types::Feature> = graph
            .features
            .values()
            .filter(|f| {
                f.front.tests.iter().any(|tid| {
                    graph
                        .tests
                        .get(tid.as_str())
                        .map(|t| t.front.status == types::TestStatus::Failing)
                        .unwrap_or(false)
                })
            })
            .collect();
        if fmt == "json" {
            let arr: Vec<serde_json::Value> = items
                .iter()
                .map(|f| {
                    serde_json::json!({
                        "id": f.front.id,
                        "title": f.front.title,
                        "phase": f.front.phase,
                    })
                })
                .collect();
            println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
        } else {
            println!("Features with failing tests:");
            for f in &items {
                println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
            }
        }
        return Ok(());
    }

    let mut phases: Vec<u32> = graph
        .features
        .values()
        .map(|f| f.front.phase)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();
    phases.sort();

    // Compute topological order for deterministic display (same as `feature next`)
    let topo_order: std::collections::HashMap<String, usize> = graph
        .topological_sort()
        .unwrap_or_else(|_| {
            // Fallback to ID sort if cycle detected
            let mut ids: Vec<String> = graph.features.keys().cloned().collect();
            ids.sort();
            ids
        })
        .into_iter()
        .enumerate()
        .map(|(i, id)| (id, i))
        .collect();

    if fmt == "json" {
        let mut phase_arr: Vec<serde_json::Value> = Vec::new();
        for p in &phases {
            if let Some(filter_phase) = phase {
                if *p != filter_phase {
                    continue;
                }
            }
            let mut phase_features: Vec<&types::Feature> = graph
                .features
                .values()
                .filter(|f| f.front.phase == *p)
                .collect();
            phase_features.sort_by_key(|f| topo_order.get(&f.front.id).copied().unwrap_or(usize::MAX));
            let name = config
                .phases
                .get(&p.to_string())
                .cloned()
                .unwrap_or_else(|| format!("Phase {}", p));
            let complete = phase_features
                .iter()
                .filter(|f| f.front.status == types::FeatureStatus::Complete)
                .count();
            let total = phase_features.len();
            let gate = graph.phase_gate_satisfied(*p);
            let gate_status = if gate.is_open() { "OPEN" } else { "LOCKED" };
            let features_json: Vec<serde_json::Value> = phase_features
                .iter()
                .map(|f| {
                    let test_count = f.front.tests.len();
                    let passing = f
                        .front
                        .tests
                        .iter()
                        .filter(|tid| {
                            graph
                                .tests
                                .get(tid.as_str())
                                .map(|t| t.front.status == types::TestStatus::Passing)
                                .unwrap_or(false)
                        })
                        .count();
                    serde_json::json!({
                        "id": f.front.id,
                        "title": f.front.title,
                        "status": f.front.status.to_string(),
                        "tests_passing": passing,
                        "tests_total": test_count,
                    })
                })
                .collect();
            phase_arr.push(serde_json::json!({
                "phase": p,
                "name": name,
                "complete": complete,
                "total": total,
                "gate": gate_status,
                "features": features_json,
            }));
        }
        let obj = serde_json::json!({
            "project": config.name,
            "phases": phase_arr,
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        println!("Project Status: {}", config.name);
        println!("=================");
        println!();

        for p in &phases {
            if let Some(filter_phase) = phase {
                if *p != filter_phase {
                    continue;
                }
            }

            let mut phase_features: Vec<&types::Feature> = graph
                .features
                .values()
                .filter(|f| f.front.phase == *p)
                .collect();
            phase_features.sort_by_key(|f| topo_order.get(&f.front.id).copied().unwrap_or(usize::MAX));

            let name = config
                .phases
                .get(&p.to_string())
                .cloned()
                .unwrap_or_else(|| format!("Phase {}", p));

            let complete = phase_features
                .iter()
                .filter(|f| f.front.status == types::FeatureStatus::Complete)
                .count();
            let total = phase_features.len();

            // Phase gate status
            let gate = graph.phase_gate_satisfied(*p);
            let gate_label = match &gate {
                graph::PhaseGateStatus::Open { .. } => "[OPEN]".to_string(),
                graph::PhaseGateStatus::Locked { failing, .. } => {
                    format!("[LOCKED — exit criteria not passing: {}]", failing.join(", "))
                }
            };

            println!("Phase {} — {} ({}/{} complete)  {}", p, name, complete, total, gate_label);

            // If --phase N is set, show exit-criteria detail
            if phase.is_some() {
                let exit_criteria = match &gate {
                    graph::PhaseGateStatus::Open { exit_criteria } => exit_criteria,
                    graph::PhaseGateStatus::Locked { exit_criteria, .. } => exit_criteria,
                };
                if !exit_criteria.is_empty() {
                    println!();
                    println!("  Exit criteria:");
                    for tc in exit_criteria {
                        let mark = if tc.passing { "passing  ✓" } else { "failing  ✗" };
                        println!("    {}  {}  [{}]", tc.id, tc.title, mark);
                    }
                    println!();
                }
            }

            for f in &phase_features {
                let test_count = f.front.tests.len();
                let passing = f
                    .front
                    .tests
                    .iter()
                    .filter(|tid| {
                        graph
                            .tests
                            .get(tid.as_str())
                            .map(|t| t.front.status == types::TestStatus::Passing)
                            .unwrap_or(false)
                    })
                    .count();
                println!(
                    "  {} {:<15} {} (tests: {}/{})",
                    match f.front.status {
                        types::FeatureStatus::Complete => "[x]",
                        types::FeatureStatus::InProgress => "[~]",
                        types::FeatureStatus::Planned => "[ ]",
                        types::FeatureStatus::Abandoned => "[-]",
                    },
                    f.front.id,
                    f.front.title,
                    passing,
                    test_count,
                );
            }
            println!();
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Checklist command
// ---------------------------------------------------------------------------

fn handle_checklist(cmd: ChecklistCommands) -> BoxResult {
    match cmd {
        ChecklistCommands::Generate => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            // Git-aware warning: check for uncommitted artifact files
            fileops::warn_uncommitted_changes(&root);
            let content = checklist::generate(&graph);
            let path = config.resolve_path(&root, &config.paths.checklist);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            fileops::write_file_atomic(&path, &content)?;
            println!("Generated: {}", path.display());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Completions command
// ---------------------------------------------------------------------------

fn handle_completions(shell: &str) -> BoxResult {
    use clap::CommandFactory;
    use clap_complete::{generate, Shell};

    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        other => {
            eprintln!("Unknown shell: {}. Use: bash, zsh, fish", other);
            process::exit(1);
        }
    };

    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "product", &mut std::io::stdout());
    Ok(())
}

// ---------------------------------------------------------------------------
// Migrate command
// ---------------------------------------------------------------------------

fn handle_migrate(cmd: MigrateCommands) -> BoxResult {
    match cmd {
        MigrateCommands::FromPrd {
            source,
            validate,
            execute,
            overwrite,
            interactive,
        } => {
            let _lock = acquire_write_lock()?;
            let (config, root, _) = load_graph()?;
            let features_dir = config.resolve_path(&root, &config.paths.features);
            let plan =
                migrate::migrate_from_prd(&PathBuf::from(&source), &features_dir, &config.prefixes.feature)?;
            plan.print_summary();

            if validate || (!execute && !interactive) {
                println!("Run with --execute to create these files.");
            } else {
                std::fs::create_dir_all(&features_dir)?;
                let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
                let tests_dir = config.resolve_path(&root, &config.paths.tests);
                let (written, skipped) =
                    migrate::execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, overwrite, interactive)?;
                println!("\n{} files written, {} skipped", written, skipped);
            }
        }
        MigrateCommands::FromAdrs {
            source,
            validate,
            execute,
            overwrite,
            interactive,
        } => {
            let _lock = acquire_write_lock()?;
            let (config, root, _) = load_graph()?;
            let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
            let tests_dir = config.resolve_path(&root, &config.paths.tests);
            let plan = migrate::migrate_from_adrs(
                &PathBuf::from(&source),
                &adrs_dir,
                &tests_dir,
                &config.prefixes.adr,
                &config.prefixes.test,
            )?;
            plan.print_summary();

            if validate || (!execute && !interactive) {
                println!("Run with --execute to create these files.");
            } else {
                let features_dir = config.resolve_path(&root, &config.paths.features);
                std::fs::create_dir_all(&adrs_dir)?;
                std::fs::create_dir_all(&tests_dir)?;
                let (written, skipped) =
                    migrate::execute_plan(&plan, &features_dir, &adrs_dir, &tests_dir, overwrite, interactive)?;
                println!("\n{} files written, {} skipped", written, skipped);
            }
        }
        MigrateCommands::Schema { dry_run } => {
            let _lock = acquire_write_lock()?;
            let (cfg, root, _) = load_graph()?;
            let version: u32 = cfg.schema_version.parse().unwrap_or(0);
            if version >= config::CURRENT_SCHEMA_VERSION {
                println!("Schema is already at version {} (current)", version);
            } else {
                println!(
                    "Migrating schema from {} to {}{}",
                    version,
                    config::CURRENT_SCHEMA_VERSION,
                    if dry_run { " (dry-run)" } else { "" }
                );
                let (updated, unchanged) = config::migrate_schema(&root, &cfg, dry_run)?;
                println!("{} files updated, {} unchanged", updated, unchanged);
            }
        }
        MigrateCommands::Validate => {
            let (_, _, graph) = load_graph()?;
            let result = graph.check();
            result.print_stderr();
            println!(
                "Validation: {} errors, {} warnings",
                result.errors.len(),
                result.warnings.len()
            );
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Gap commands
// ---------------------------------------------------------------------------

fn handle_gap(cmd: GapCommands, _global_fmt: &str) -> BoxResult {
    let (_, root, graph) = load_graph()?;
    let baseline_path = root.join("gaps.json");
    let mut baseline = gap::GapBaseline::load(&baseline_path);

    match cmd {
        GapCommands::Check { adr_id, changed, format } => {
            // Determine which ADR IDs to check
            let adr_ids_to_check: Vec<String> = if let Some(ref id) = adr_id {
                vec![id.clone()]
            } else if changed {
                // check_changed returns reports; we need IDs for model analysis
                // We'll use the existing check_changed for structural + build reports
                let reports = gap::check_changed(&graph, &baseline, &root);
                // Update resolved in baseline
                let all_finding_ids: Vec<String> = reports
                    .iter()
                    .flat_map(|r| r.findings.iter().map(|f| f.id.clone()))
                    .collect();
                baseline.update_resolved(&all_finding_ids);
                baseline.save(&baseline_path)?;

                if format == "json" {
                    println!("{}", serde_json::to_string_pretty(&reports).unwrap_or_default());
                } else {
                    for report in &reports {
                        if report.findings.is_empty() {
                            continue;
                        }
                        println!("--- {} ---", report.adr);
                        for finding in &report.findings {
                            let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
                            println!(
                                "  [{:>6}] {} — {}{}",
                                finding.severity, finding.code, finding.description, suppressed_tag
                            );
                        }
                    }
                }

                let has_new_high = reports.iter().any(|r| {
                    r.findings.iter().any(|f| f.severity == gap::GapSeverity::High && !f.suppressed)
                });
                if has_new_high {
                    process::exit(1);
                }
                return Ok(());
            } else {
                graph.adrs.keys().cloned().collect()
            };

            // For each ADR: structural checks + model analysis
            let mut reports = Vec::new();
            for id in &adr_ids_to_check {
                let mut findings = gap::check_adr(&graph, id, &baseline);

                // Try model-based analysis (may use injected response in tests)
                match gap::try_model_analysis(id, &baseline) {
                    Ok(model_findings) => {
                        findings.extend(model_findings);
                    }
                    Err(e) => {
                        eprintln!("error: gap analysis model failure for {}: {}", id, e);
                        process::exit(2);
                    }
                }

                let summary = gap::GapSummary {
                    high: findings.iter().filter(|f| f.severity == gap::GapSeverity::High && !f.suppressed).count(),
                    medium: findings.iter().filter(|f| f.severity == gap::GapSeverity::Medium && !f.suppressed).count(),
                    low: findings.iter().filter(|f| f.severity == gap::GapSeverity::Low && !f.suppressed).count(),
                    suppressed: findings.iter().filter(|f| f.suppressed).count(),
                };
                reports.push(gap::GapReport {
                    adr: id.clone(),
                    run_date: chrono::Utc::now().to_rfc3339(),
                    product_version: env!("CARGO_PKG_VERSION").to_string(),
                    findings,
                    summary,
                });
            }

            // Update resolved in baseline
            let all_finding_ids: Vec<String> = reports
                .iter()
                .flat_map(|r| r.findings.iter().map(|f| f.id.clone()))
                .collect();
            baseline.update_resolved(&all_finding_ids);
            baseline.save(&baseline_path)?;

            if format == "json" {
                println!("{}", serde_json::to_string_pretty(&reports).unwrap_or_default());
            } else {
                for report in &reports {
                    if report.findings.is_empty() {
                        continue;
                    }
                    println!("--- {} ---", report.adr);
                    for finding in &report.findings {
                        let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
                        println!(
                            "  [{:>6}] {} — {}{}",
                            finding.severity, finding.code, finding.description, suppressed_tag
                        );
                    }
                }
            }

            // Exit 1 if any new unsuppressed high-severity gaps
            let has_new_high = reports.iter().any(|r| {
                r.findings.iter().any(|f| f.severity == gap::GapSeverity::High && !f.suppressed)
            });
            if has_new_high {
                process::exit(1);
            }
        }
        GapCommands::Report => {
            let reports = gap::check_all(&graph, &baseline);
            let total_findings: usize = reports.iter().map(|r| r.findings.len()).sum();
            let total_high: usize = reports.iter().flat_map(|r| &r.findings)
                .filter(|f| f.severity == gap::GapSeverity::High && !f.suppressed).count();
            let total_medium: usize = reports.iter().flat_map(|r| &r.findings)
                .filter(|f| f.severity == gap::GapSeverity::Medium && !f.suppressed).count();
            let total_low: usize = reports.iter().flat_map(|r| &r.findings)
                .filter(|f| f.severity == gap::GapSeverity::Low && !f.suppressed).count();
            let total_suppressed: usize = reports.iter().flat_map(|r| &r.findings)
                .filter(|f| f.suppressed).count();

            println!("Gap Analysis Report");
            println!("====================");
            println!("ADRs analysed: {}", reports.len());
            println!("Total findings: {} (high: {}, medium: {}, low: {}, suppressed: {})",
                total_findings, total_high, total_medium, total_low, total_suppressed);
            println!();

            for report in &reports {
                if report.findings.is_empty() {
                    continue;
                }
                println!("--- {} ({} findings) ---", report.adr, report.findings.len());
                for finding in &report.findings {
                    let suppressed_tag = if finding.suppressed { " [suppressed]" } else { "" };
                    println!(
                        "  [{:>6}] {} — {}{}",
                        finding.severity, finding.code, finding.description, suppressed_tag
                    );
                    println!("           Action: {}", finding.suggested_action);
                    if !finding.affected_artifacts.is_empty() {
                        println!("           Affects: {}", finding.affected_artifacts.join(", "));
                    }
                }
                println!();
            }
        }
        GapCommands::Suppress { gap_id, reason } => {
            baseline.suppress(&gap_id, &reason);
            baseline.save(&baseline_path)?;
            println!("Suppressed: {}", gap_id);
        }
        GapCommands::Unsuppress { gap_id } => {
            baseline.unsuppress(&gap_id);
            baseline.save(&baseline_path)?;
            println!("Unsuppressed: {}", gap_id);
        }
        GapCommands::Stats => {
            let reports = gap::check_all(&graph, &baseline);
            let stats = gap::gap_stats(&reports, &baseline);
            println!("{}", serde_json::to_string_pretty(&stats).unwrap_or_default());
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Drift commands
// ---------------------------------------------------------------------------

fn handle_drift(cmd: DriftCommands, fmt: &str) -> BoxResult {
    let (_config, root, graph) = load_graph()?;
    let baseline_path = root.join("drift.json");
    let mut baseline = drift::DriftBaseline::load(&baseline_path);

    // Default source roots and ignore patterns
    let source_roots = vec!["src".to_string(), "crates".to_string()];
    let ignore = vec!["target".to_string(), ".git".to_string(), "node_modules".to_string()];

    match cmd {
        DriftCommands::Check { adr_id, files } => {
            let all_findings: Vec<drift::DriftFinding> = if let Some(ref id) = adr_id {
                drift::check_adr(id, &graph, &root, &baseline, &source_roots, &ignore, &files)
            } else {
                let adr_ids: Vec<String> = graph.adrs.keys().cloned().collect();
                let mut combined = Vec::new();
                for id in &adr_ids {
                    combined.extend(drift::check_adr(id, &graph, &root, &baseline, &source_roots, &ignore, &files));
                }
                combined
            };

            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&all_findings).unwrap_or_default());
            } else {
                if all_findings.is_empty() {
                    println!("No drift findings.");
                } else {
                    for f in &all_findings {
                        let suppressed_tag = if f.suppressed { " [suppressed]" } else { "" };
                        println!(
                            "[{:>6}] {} ({}) — {}{}",
                            f.severity, f.id, f.code, f.description, suppressed_tag
                        );
                        println!("         Action: {}", f.suggested_action);
                        if !f.source_files.is_empty() {
                            println!("         Files: {}", f.source_files.join(", "));
                        }
                    }
                }
            }

            // Exit 1 if any unsuppressed high-severity findings
            let has_high = all_findings.iter().any(|f| {
                f.severity == drift::DriftSeverity::High && !f.suppressed
            });
            if has_high {
                process::exit(1);
            }
        }
        DriftCommands::Scan { path } => {
            let source_path = std::path::Path::new(&path);
            let adrs = drift::scan_source(source_path, &graph);
            if fmt == "json" {
                println!("{}", serde_json::to_string_pretty(&adrs).unwrap_or_default());
            } else if adrs.is_empty() {
                println!("No governing ADRs found for {}", path);
            } else {
                println!("Governing ADRs for {}:", path);
                for adr_id in &adrs {
                    let title = graph
                        .adrs
                        .get(adr_id)
                        .map(|a| a.front.title.as_str())
                        .unwrap_or("(unknown)");
                    println!("  {} — {}", adr_id, title);
                }
            }
        }
        DriftCommands::Suppress { drift_id, reason } => {
            baseline.suppress(&drift_id, &reason);
            baseline.save(&baseline_path)?;
            println!("Suppressed: {}", drift_id);
        }
        DriftCommands::Unsuppress { drift_id } => {
            baseline.unsuppress(&drift_id);
            baseline.save(&baseline_path)?;
            println!("Unsuppressed: {}", drift_id);
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Implement command
// ---------------------------------------------------------------------------

fn handle_preflight(id: &str) -> BoxResult {
    let (config, _root, graph) = load_graph()?;
    let result = domains::preflight(&graph, id, &config.domains)?;
    print!("{}", domains::render_preflight(&result));
    if !result.is_clean {
        process::exit(1);
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Onboard command (ADR-027)
// ---------------------------------------------------------------------------

fn handle_onboard(cmd: OnboardCommands) -> BoxResult {
    match cmd {
        OnboardCommands::Scan {
            source,
            output,
            max_candidates,
            no_validate,
        } => {
            let source_path = PathBuf::from(&source);
            let scan_result = onboard::scan(
                &source_path,
                max_candidates,
                !no_validate,
            )?;

            let json = serde_json::to_string_pretty(&scan_result)
                .map_err(|e| ProductError::IoError(format!("failed to serialize scan output: {}", e)))?;

            // Write output file
            let output_path = PathBuf::from(&output);
            if let Some(parent) = output_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        ProductError::IoError(format!("cannot create output directory: {}", e))
                    })?;
                }
            }
            std::fs::write(&output_path, &json).map_err(|e| {
                ProductError::WriteError {
                    path: output_path.clone(),
                    message: e.to_string(),
                }
            })?;

            println!(
                "Scan complete: {} candidates from {} files",
                scan_result.candidates.len(),
                scan_result.scan_metadata.files_scanned
            );
            println!("Output written to {}", output_path.display());

            // Print warnings
            for c in &scan_result.candidates {
                for w in &c.warnings {
                    eprintln!("warning: {} ({}): {}", c.id, c.title, w);
                }
            }

            Ok(())
        }

        OnboardCommands::Triage {
            source,
            interactive,
            output,
        } => {
            let source_path = PathBuf::from(&source);
            let content = std::fs::read_to_string(&source_path).map_err(|e| {
                ProductError::IoError(format!(
                    "cannot read candidates file {}: {}",
                    source_path.display(),
                    e
                ))
            })?;

            let scan_output: onboard::ScanOutput = serde_json::from_str(&content)
                .map_err(|e| {
                    ProductError::IoError(format!(
                        "cannot parse candidates file: {}",
                        e
                    ))
                })?;

            let triage_output = if interactive {
                let stdin = std::io::stdin();
                let mut reader = std::io::BufReader::new(stdin.lock());
                onboard::triage_interactive(&scan_output, &mut reader)?
            } else {
                // Non-interactive: batch confirm all
                onboard::triage_batch_confirm(&scan_output)
            };

            let json = serde_json::to_string_pretty(&triage_output)
                .map_err(|e| ProductError::IoError(format!("failed to serialize triage output: {}", e)))?;

            let output_path = PathBuf::from(&output);
            if let Some(parent) = output_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).map_err(|e| {
                        ProductError::IoError(format!("cannot create output directory: {}", e))
                    })?;
                }
            }
            std::fs::write(&output_path, &json).map_err(|e| {
                ProductError::WriteError {
                    path: output_path.clone(),
                    message: e.to_string(),
                }
            })?;

            let confirmed = triage_output
                .candidates
                .iter()
                .filter(|c| c.triage_status == onboard::TriageStatus::Confirmed)
                .count();
            let rejected = triage_output
                .candidates
                .iter()
                .filter(|c| c.triage_status == onboard::TriageStatus::Rejected)
                .count();
            let merged = triage_output
                .candidates
                .iter()
                .filter(|c| c.triage_status == onboard::TriageStatus::Merged)
                .count();

            println!(
                "Triage complete: {} confirmed, {} rejected, {} merged",
                confirmed, rejected, merged
            );
            println!("Output written to {}", output_path.display());

            Ok(())
        }

        OnboardCommands::Seed { source, dry_run } => {
            let _lock = if !dry_run {
                Some(acquire_write_lock()?)
            } else {
                None
            };

            let (config, root, _graph) = load_graph()?;

            let source_path = PathBuf::from(&source);
            let content = std::fs::read_to_string(&source_path).map_err(|e| {
                ProductError::IoError(format!(
                    "cannot read triaged file {}: {}",
                    source_path.display(),
                    e
                ))
            })?;

            let triage_output: onboard::TriageOutput = serde_json::from_str(&content)
                .map_err(|e| {
                    ProductError::IoError(format!(
                        "cannot parse triaged file: {}",
                        e
                    ))
                })?;

            // Get existing IDs
            let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
            let features_dir = config.resolve_path(&root, &config.paths.features);
            let tests_dir = config.resolve_path(&root, &config.paths.tests);

            let loaded =
                parser::load_all(&features_dir, &adrs_dir, &tests_dir)?;
            let (features_all, adrs_all) = (loaded.features, loaded.adrs);

            let existing_adrs: Vec<String> = adrs_all
                .iter()
                .map(|a| a.front.id.clone())
                .collect();
            let existing_features: Vec<String> = features_all
                .iter()
                .map(|f| f.front.id.clone())
                .collect();

            let seed_result = onboard::plan_seed(
                &triage_output,
                &existing_adrs,
                &existing_features,
                &config.prefixes.adr,
                &config.prefixes.feature,
            );

            if dry_run {
                println!("Dry run — the following files would be created:\n");
                println!("ADR files:");
                for adr in &seed_result.adrs {
                    println!("  {} — {} ({})", adr.id, adr.title, adr.filename);
                }
                println!("\nFeature stubs:");
                for ft in &seed_result.features {
                    println!(
                        "  {} — {} ({}) -> [{}]",
                        ft.id,
                        ft.title,
                        ft.filename,
                        ft.adr_ids.join(", ")
                    );
                }
                println!(
                    "\nTotal: {} ADRs, {} feature stubs",
                    seed_result.adrs.len(),
                    seed_result.features.len()
                );
            } else {
                println!("Seeding knowledge graph...\n");
                onboard::execute_seed(&seed_result, &adrs_dir, &features_dir)?;
                println!(
                    "\nSeed complete: {} ADRs, {} feature stubs created",
                    seed_result.adrs.len(),
                    seed_result.features.len()
                );
            }

            Ok(())
        }
    }
}

fn handle_implement(id: &str, dry_run: bool, no_verify: bool, headless: bool) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    implement::run_implement(id, &config, &root, &graph, dry_run, no_verify, headless)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Verify command
// ---------------------------------------------------------------------------

fn handle_verify(id: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    implement::run_verify(id, &config, &root, &graph)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Author command
// ---------------------------------------------------------------------------

fn handle_author(cmd: AuthorCommands) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let session_type = match &cmd {
        AuthorCommands::Feature { .. } => author::SessionType::Feature,
        AuthorCommands::Adr => author::SessionType::Adr,
        AuthorCommands::Review => author::SessionType::Review,
    };

    // ADR-026: if authoring a feature, run preflight first
    if let AuthorCommands::Feature { feature: Some(ref fid) } = cmd {
        let result = domains::preflight(&graph, fid, &config.domains)?;
        if !result.is_clean {
            eprintln!("{}", domains::render_preflight(&result));
            eprintln!("  Resolve preflight gaps before starting author session.");
            process::exit(1);
        }
    }

    author::start_session(session_type, &config, &root)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// MCP command
// ---------------------------------------------------------------------------

fn handle_mcp(
    http: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    repo: Option<String>,
    write_flag: bool,
) -> BoxResult {
    let repo_root = if let Some(ref path) = repo {
        PathBuf::from(path)
    } else {
        let (_config, root) = ProductConfig::discover()?;
        root
    };

    // --write flag overrides product.toml mcp.write
    let write_enabled = write_flag || {
        let toml_path = repo_root.join("product.toml");
        if toml_path.exists() {
            let cfg = ProductConfig::load(&toml_path)?;
            cfg.mcp.map(|m| m.write).unwrap_or(false)
        } else {
            false
        }
    };

    if http {
        let toml_path = repo_root.join("product.toml");
        let cors_origins = if toml_path.exists() {
            let cfg = ProductConfig::load(&toml_path)?;
            cfg.mcp.map(|m| m.cors_origins).unwrap_or_default()
        } else {
            vec![]
        };
        let rt = tokio::runtime::Runtime::new().map_err(|e| {
            ProductError::IoError(format!("Failed to create tokio runtime: {}", e))
        })?;
        rt.block_on(mcp::run_http(
            repo_root,
            write_enabled,
            port,
            bind,
            token,
            cors_origins,
        ))?;
    } else {
        mcp::run_stdio(repo_root, write_enabled)?;
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Metrics commands
// ---------------------------------------------------------------------------

fn handle_metrics(cmd: MetricsCommands) -> BoxResult {
    let (config, root, graph) = load_graph()?;
    let jsonl_path = root.join("metrics.jsonl");

    match cmd {
        MetricsCommands::Record => {
            let snapshot = metrics::record(&graph, &root);
            metrics::append_snapshot(&snapshot, &jsonl_path)?;
            print!("{}", metrics::render_summary(&snapshot));
            println!("Appended to {}", jsonl_path.display());
        }
        MetricsCommands::Threshold => {
            let snapshot = metrics::record(&graph, &root);
            let thresholds = config
                .metrics
                .as_ref()
                .map(|m| &m.thresholds)
                .cloned()
                .unwrap_or_default();
            let (errors, warnings) = metrics::check_thresholds(&snapshot, &thresholds);

            for w in &warnings {
                eprintln!("warning: {}", w);
            }
            for e in &errors {
                eprintln!("error: {}", e);
            }

            if !errors.is_empty() {
                process::exit(1);
            } else if !warnings.is_empty() {
                process::exit(2);
            }
        }
        MetricsCommands::Trend { metric } => {
            let (snapshots, warnings) = metrics::load_snapshots_with_warnings(&jsonl_path);
            for w in &warnings {
                eprintln!("{}", w);
            }
            if snapshots.is_empty() {
                println!("No snapshots found. Run `product metrics record` first.");
                return Ok(());
            }
            match metric {
                Some(name) => {
                    print!("{}", metrics::render_trend(&snapshots, &name));
                }
                None => {
                    let last = snapshots.last();
                    if let Some(s) = last {
                        print!("{}", metrics::render_summary(s));
                    }
                    println!();
                    for name in &[
                        "spec_coverage",
                        "test_coverage",
                        "exit_criteria_coverage",
                        "phi",
                        "gap_density",
                        "gap_resolution_rate",
                        "drift_density",
                        "centrality_stability",
                    ] {
                        print!("{}", metrics::render_trend(&snapshots, name));
                    }
                }
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// InstallHooks command
// ---------------------------------------------------------------------------

fn handle_install_hooks() -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_config, root) = ProductConfig::discover()?;

    // Write pre-commit hook
    let hooks_dir = root.join(".git").join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let hook_path = hooks_dir.join("pre-commit");
    let hook_content = "#!/bin/sh\n\
        # Installed by `product install-hooks`\n\
        exec product adr review --staged\n";
    fileops::write_file_atomic(&hook_path, hook_content)?;

    // Make executable (Unix)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))?;
    }

    println!("Installed pre-commit hook: {}", hook_path.display());

    // Write .mcp.json
    mcp::scaffold_mcp_json(&root)?;
    println!("Wrote .mcp.json: {}", root.join(".mcp.json").display());

    Ok(())
}

// ---------------------------------------------------------------------------
// Init command (ADR-033)
// ---------------------------------------------------------------------------

fn handle_init(_yes: bool, force: bool, name: Option<String>) -> BoxResult {
    let target_dir = std::env::current_dir().map_err(|e| {
        ProductError::ConfigError(format!("Cannot determine working directory: {}", e))
    })?;
    let toml_path = target_dir.join("product.toml");

    // Determine checklist-in-gitignore setting.
    // If --force and product.toml exists, preserve the existing setting.
    let checklist_in_gitignore = if toml_path.exists() {
        if !force {
            return Err(Box::new(ProductError::ConfigError(format!(
                "product.toml already exists\n  --> {}\n  = hint: use `product init --force` to overwrite, or edit the file directly",
                toml_path.display()
            ))));
        }
        // --force: read existing config to preserve checklist-in-gitignore
        ProductConfig::load(&toml_path)
            .map(|c| c.checklist_in_gitignore)
            .unwrap_or(true)
    } else {
        true
    };

    // Determine project name
    let project_name = name.unwrap_or_else(|| {
        target_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("my-project")
            .to_string()
    });

    // Generate product.toml
    let toml_content = format!(
        r#"name = "{project_name}"
schema-version = "1"
checklist-in-gitignore = {checklist_in_gitignore}

[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"

[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"

[phases]
1 = "Phase 1"

[domains]

[mcp]
write = false
port = 7777
"#
    );
    fileops::write_file_atomic(&toml_path, &toml_content)?;
    println!("  product.toml");

    // Create directory skeleton
    let dirs = ["docs/features", "docs/adrs", "docs/tests", "docs/graph"];
    for d in &dirs {
        let path = target_dir.join(d);
        std::fs::create_dir_all(&path).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", path.display(), e))
        })?;
        println!("  {}/", d);
    }

    // Manage .gitignore
    let gitignore_path = target_dir.join(".gitignore");
    init_manage_gitignore(&gitignore_path, checklist_in_gitignore)?;

    println!("\nRun `product feature new \"My First Feature\"` to get started.");
    Ok(())
}

/// Manage .gitignore entries for generated files (ADR-033, ADR-007).
/// Always adds `docs/graph/`. Adds `docs/checklist.md` only when checklist_in_gitignore is true.
fn init_manage_gitignore(path: &std::path::Path, checklist_in_gitignore: bool) -> BoxResult {
    let mut entries_to_add: Vec<&str> = vec!["docs/graph/"];
    if checklist_in_gitignore {
        entries_to_add.push("docs/checklist.md");
    }

    let existing = if path.exists() {
        std::fs::read_to_string(path).map_err(|e| {
            ProductError::IoError(format!("failed to read {}: {}", path.display(), e))
        })?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = if existing.is_empty() {
        Vec::new()
    } else {
        existing.lines().map(String::from).collect()
    };

    let has_header = lines.iter().any(|l| l.contains("Product CLI"));
    let mut added_any = false;

    for entry in &entries_to_add {
        if !lines.iter().any(|l| l.trim() == *entry) {
            if !added_any && !has_header {
                if !lines.is_empty() && lines.last().map(|l| !l.is_empty()).unwrap_or(false) {
                    lines.push(String::new());
                }
                lines.push("# Product CLI — generated files".to_string());
            }
            lines.push(entry.to_string());
            added_any = true;
        }
    }

    let mut content = lines.join("\n");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    fileops::write_file_atomic(path, &content)?;
    println!("  .gitignore");
    Ok(())
}
