//! Command dispatch module — subcommand enums, run(), shared helpers.

mod adr;
mod adr_conflicts;
mod adr_seal;
mod adr_write;
mod agent_init;
mod author;
mod checklist;
mod completions;
mod context;
mod dep;
mod drift;
mod drift_diff;
mod feature;
mod feature_write;
mod gap;
mod graph_autolink;
mod graph_cmd;
mod hash;
mod hooks;
mod implement;
mod init;
mod mcp_cmd;
mod metrics_cmd;
mod migrate;
mod onboard;
mod output;
mod preflight;
mod prompts_cmd;
mod request_cmd;
mod request_log_cmd;
mod schema;
mod shared;
mod status;
mod tags;
mod test_cmd;

pub(crate) use self::output::{render_result as render, CmdResult, Output};
pub(crate) use self::shared::{acquire_write_lock, acquire_write_lock_typed, load_graph, load_graph_typed};

use clap::Subcommand;
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
        /// Feature or ADR ID to bundle (not required with --measure-all)
        #[arg(required_unless_present = "measure_all")]
        id: Option<String>,
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
        /// Measure every feature in one pass, printing only the aggregate summary
        #[arg(long = "measure-all")]
        measure_all: bool,
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
    /// Verify test criteria — unified six-stage pipeline (FT-044) when no
    /// feature ID is supplied, or per-feature (ADR-021) otherwise.
    Verify {
        /// Feature ID (optional — if omitted, runs the full pipeline)
        id: Option<String>,
        /// Run all TCs linked to cross-cutting ADRs, regardless of feature
        #[arg(long)]
        platform: bool,
        /// Skip ADR lifecycle check (bypass E016 for migration scenarios)
        #[arg(long)]
        skip_adr_check: bool,
        /// Scope the pipeline's stage 5 (feature TCs) to a phase
        #[arg(long)]
        phase: Option<u32>,
        /// Emit single-document JSON to stdout for CI pipelines (no colour)
        #[arg(long)]
        ci: bool,
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
    /// Tag lifecycle — browse product/* git tags (ADR-036)
    Tags {
        #[command(subcommand)]
        command: tags::TagsCommands,
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
    /// Display front-matter schemas for artifact types (ADR-031, FT-049)
    Schema {
        /// Artifact type: feature, adr, test, dep, formal
        artifact_type: Option<String>,
        /// Artifact type as a named flag (alternative to the positional).
        /// Example: `product schema --type formal`.
        #[arg(long = "type", value_name = "TYPE")]
        type_flag: Option<String>,
        /// Show all schemas in a single document
        #[arg(long)]
        all: bool,
    },
    /// Generate AGENTS.md from current repository state (ADR-031)
    AgentInit {
        /// Watch for changes and regenerate automatically
        #[arg(long)]
        watch: bool,
    },
    /// Unified atomic write interface (FT-041, ADR-038)
    Request {
        #[command(subcommand)]
        command: request_cmd::RequestCommands,
    },
}

pub use self::test_cmd::TestCommands;

type BoxResult = Result<(), Box<dyn std::error::Error>>;

pub fn run(command: Commands, format: &str, cli_command: &mut clap::Command) -> BoxResult {
    shared::run_startup_hooks()?;
    dispatch(command, format, cli_command)
}

fn dispatch(command: Commands, fmt: &str, cli_command: &mut clap::Command) -> BoxResult {
    match command {
        Commands::Feature { command } => feature::handle_feature(command, fmt),
        Commands::Adr { command } => adr::handle_adr(command, fmt),
        Commands::Test { command } => test_cmd::handle_test(command, fmt),
        Commands::Dep { command } => dep::handle_dep(command, fmt),
        Commands::Context { id, depth, phase, adrs_only, order, measure, measure_all } =>
            context::handle_context(id.as_deref(), depth, phase, adrs_only, order, measure, measure_all),
        Commands::Graph { command } => graph_cmd::handle_graph(command, fmt),
        Commands::Impact { id } => render(status::handle_impact(&id, fmt), fmt),
        Commands::Status {
            phase,
            untested,
            failing,
        } => render(status::handle_status(phase, untested, failing, fmt), fmt),
        Commands::Checklist { command } => checklist::handle_checklist(command),
        Commands::Completions { shell } => completions::handle_completions(&shell, cli_command),
        Commands::Migrate { command } => migrate::handle_migrate(command),
        Commands::Gap { command } => gap::handle_gap(command, fmt),
        Commands::Implement { id, dry_run, no_verify, headless } => implement::handle_implement(&id, dry_run, no_verify, headless),
        Commands::Verify { id, platform, skip_adr_check, phase, ci } =>
            implement::handle_verify(id.as_deref(), platform, skip_adr_check, phase, ci),
        Commands::Author { command } => author::handle_author(command),
        Commands::Mcp { http, port, bind, token, repo, write } => mcp_cmd::handle_mcp(http, port, &bind, token, repo, write),
        Commands::InstallHooks => hooks::handle_install_hooks(),
        Commands::Prompts { command } => prompts_cmd::handle_prompts(command),
        Commands::Drift { command } => drift::handle_drift(command, fmt),
        Commands::Tags { command } => tags::handle_tags(command, fmt),
        Commands::Metrics { command } => metrics_cmd::handle_metrics(command),
        Commands::Preflight { id } => preflight::handle_preflight(&id),
        Commands::Onboard { command } => onboard::handle_onboard(command),
        Commands::Init { yes, force, name, domains, port, write_tools, path } => init::handle_init(yes, force, name, domains, port, write_tools, path),
        Commands::Hash { command } => hash::handle_hash(command),
        Commands::Schema { artifact_type, type_flag, all } => {
            // `--type` wins over the positional so the idealised invocation
            // `product schema --type formal` works even when both are given.
            schema::handle_schema(type_flag.or(artifact_type), all)
        }
        Commands::AgentInit { watch } => agent_init::handle_agent_init(watch),
        Commands::Request { command } => request_cmd::handle_request(command, fmt),
    }
}
