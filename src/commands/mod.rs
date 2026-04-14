//! Command dispatch module — subcommand enums, run(), shared helpers.

mod adr;
mod agent_init;
mod author;
mod checklist;
mod completions;
mod context;
mod dep;
mod drift;
mod feature;
mod feature_write;
mod gap;
mod graph_cmd;
mod hash;
mod hooks;
mod implement;
mod init;
mod mcp_cmd;
mod metrics_cmd;
mod migrate;
mod onboard;
mod preflight;
mod prompts_cmd;
mod schema;
mod status;
mod test_cmd;

use clap::Subcommand;
use product_lib::{config::ProductConfig, fileops, graph::KnowledgeGraph, parser};
use std::path::PathBuf;

pub use self::adr::AdrCommands;
pub use self::author::AuthorCommands;
pub use self::checklist::ChecklistCommands;
pub use self::dep::DepCommands;
pub use self::drift::DriftCommands;
pub use self::feature::FeatureCommands;
pub use self::gap::GapCommands;
pub use self::graph_cmd::GraphCommands;
pub use self::hash::HashCommands;
pub use self::metrics_cmd::MetricsCommands;
pub use self::migrate::MigrateCommands;
pub use self::onboard::OnboardCommands;
pub use self::prompts_cmd::PromptsCommands;

#[derive(Subcommand)]
pub enum Commands {
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
    /// Dependency management (ADR-030)
    Dep {
        #[command(subcommand)]
        command: DepCommands,
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
        /// Feature ID (required unless --platform is used)
        id: Option<String>,
        /// Run all TCs linked to cross-cutting ADRs, regardless of feature
        #[arg(long)]
        platform: bool,
    },
    /// Start a graph-aware authoring session
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
    },
    /// Install git hooks and scaffolding
    InstallHooks,
    /// Manage authoring session prompts (ADR-022)
    Prompts {
        #[command(subcommand)]
        command: PromptsCommands,
    },
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
        /// Add a domain (repeatable): --domain security="Auth, secrets"
        #[arg(long = "domain", value_name = "K=V")]
        domains: Vec<String>,
        /// MCP HTTP port (default: 7777)
        #[arg(long, default_value = "7777")]
        port: u16,
        /// Enable MCP write tools by default
        #[arg(long)]
        write_tools: bool,
        /// Target directory (default: current directory)
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
    },
    /// Content hash operations (ADR-032)
    Hash {
        #[command(subcommand)]
        command: HashCommands,
    },
    /// Display front-matter schemas for artifact types (ADR-031)
    Schema {
        /// Artifact type: feature, adr, test, dep
        artifact_type: Option<String>,
        /// Show all schemas in a single document
        #[arg(long)]
        all: bool,
    },
    /// Generate AGENT.md from current repository state (ADR-031)
    AgentInit {
        /// Watch for changes and regenerate automatically
        #[arg(long)]
        watch: bool,
    },
}

pub use self::test_cmd::TestCommands;

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
    let deps_dir = config.resolve_path(&root, &config.paths.dependencies);

    let loaded = parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;

    // Print parse errors to stderr so they are visible for all commands (ADR-013)
    for e in &loaded.parse_errors {
        eprintln!("{}", e);
    }

    let graph = KnowledgeGraph::build_with_deps(loaded.features, loaded.adrs, loaded.tests, loaded.dependencies)
        .with_parse_errors(loaded.parse_errors);
    Ok((config, root, graph))
}

pub fn run(command: Commands, format: &str, cli_command: &mut clap::Command) -> BoxResult {
    cleanup_stale_tmp_files();
    dispatch(command, format, cli_command)
}

fn cleanup_stale_tmp_files() {
    // Clean up any leftover tmp files from prior crashes (ADR-015)
    if let Ok((config, root)) = ProductConfig::discover() {
        let dirs = [
            config.resolve_path(&root, &config.paths.features),
            config.resolve_path(&root, &config.paths.adrs),
            config.resolve_path(&root, &config.paths.tests),
            config.resolve_path(&root, &config.paths.dependencies),
        ];
        for dir in &dirs {
            fileops::cleanup_tmp_files(dir);
        }
    }
}

fn dispatch(command: Commands, fmt: &str, cli_command: &mut clap::Command) -> BoxResult {
    match command {
        Commands::Feature { command } => feature::handle_feature(command, fmt),
        Commands::Adr { command } => adr::handle_adr(command, fmt),
        Commands::Test { command } => test_cmd::handle_test(command, fmt),
        Commands::Dep { command } => dep::handle_dep(command, fmt),
        Commands::Context {
            id,
            depth,
            phase,
            adrs_only,
            order,
            measure,
        } => context::handle_context(&id, depth, phase, adrs_only, order, measure),
        Commands::Graph { command } => graph_cmd::handle_graph(command, fmt),
        Commands::Impact { id } => status::handle_impact(&id, fmt),
        Commands::Status {
            phase,
            untested,
            failing,
        } => status::handle_status(phase, untested, failing, fmt),
        Commands::Checklist { command } => checklist::handle_checklist(command),
        Commands::Completions { shell } => completions::handle_completions(&shell, cli_command),
        Commands::Migrate { command } => migrate::handle_migrate(command),
        Commands::Gap { command } => gap::handle_gap(command, fmt),
        Commands::Implement { id, dry_run, no_verify, headless } => implement::handle_implement(&id, dry_run, no_verify, headless),
        Commands::Verify { id, platform } => implement::handle_verify(id.as_deref(), platform),
        Commands::Author { command } => author::handle_author(command),
        Commands::Mcp { http, port, bind, token, repo, write } => mcp_cmd::handle_mcp(http, port, &bind, token, repo, write),
        Commands::InstallHooks => hooks::handle_install_hooks(),
        Commands::Prompts { command } => prompts_cmd::handle_prompts(command),
        Commands::Drift { command } => drift::handle_drift(command, fmt),
        Commands::Metrics { command } => metrics_cmd::handle_metrics(command),
        Commands::Preflight { id } => preflight::handle_preflight(&id),
        Commands::Onboard { command } => onboard::handle_onboard(command),
        Commands::Init { yes, force, name, domains, port, write_tools, path } => init::handle_init(yes, force, name, domains, port, write_tools, path),
        Commands::Hash { command } => hash::handle_hash(command),
        Commands::Schema { artifact_type, all } => schema::handle_schema(artifact_type, all),
        Commands::AgentInit { watch } => agent_init::handle_agent_init(watch),
    }
}
