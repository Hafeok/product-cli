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
mod cycle_times;
mod dep;
mod dispatch;
mod drift;
mod drift_diff;
mod feature;
mod feature_fields;
mod feature_write;
mod gap;
mod graph_autolink;
mod graph_cmd;
mod hash;
mod hooks;
mod implement;
mod init;
mod init_helpers;
mod mcp_cmd;
mod metrics_cmd;
mod migrate;
mod onboard;
mod output;
mod pattern;
mod preflight;
mod prompts_cmd;
mod request_builder_add;
mod request_builder_cmd;
mod request_cmd;
mod request_cmd_helpers;
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
pub use self::pattern::PatternCommands;
pub use self::prompts_cmd::PromptsCommands;

#[derive(Subcommand)]
pub enum Commands {
    /// ADR navigation and management
    Adr {
        #[command(subcommand)]
        command: AdrCommands,
    },
    /// Generate AGENTS.md from current repository state (ADR-031)
    AgentInit {
        /// Watch for changes and regenerate automatically
        #[arg(long)]
        watch: bool,
    },
    /// Start a graph-aware authoring session
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
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
    /// Assemble context bundles for LLM agents
    Context {
        /// Feature or ADR ID, OR the literal "templates" subcommand
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
        /// Per-model template name (FT-063); falls back to [context].default-target
        #[arg(long, value_name = "NAME")]
        target: Option<String>,
        /// Deprecated alias for --target claude-opus (FT-063)
        #[arg(long = "for-llm")]
        for_llm: bool,
        /// `templates --show NAME` — print template TOML to stdout
        #[arg(long, value_name = "NAME")]
        show: Option<String>,
        /// `templates --where` — show resolution path for each template
        #[arg(long = "where")]
        where_flag: bool,
        /// `templates --reset NAME` — remove user override
        #[arg(long, value_name = "NAME")]
        reset: Option<String>,
    },
    /// Historical cycle times (FT-054, ADR-046)
    CycleTimes {
        /// Recent-N sample window (default: [cycle-times].recent-window)
        #[arg(long)]
        recent: Option<usize>,
        /// Restrict to a phase
        #[arg(long)]
        phase: Option<u32>,
        /// Show in-progress elapsed-so-far table instead
        #[arg(long = "in-progress")]
        in_progress: bool,
        /// Output format override: text | json | csv
        #[arg(long = "format", value_name = "FMT")]
        format: Option<String>,
    },
    /// Dependency management (ADR-030)
    Dep {
        #[command(subcommand)]
        command: DepCommands,
    },
    /// Drift detection — spec vs implementation
    Drift {
        #[command(subcommand)]
        command: DriftCommands,
    },
    /// Feature navigation and management
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    /// Naive completion forecast (FT-054, ADR-046)
    Forecast {
        /// Feature ID (for single-feature forecast)
        id: Option<String>,
        /// Phase number (for phase forecast)
        #[arg(long)]
        phase: Option<u32>,
        /// Required flag — opts into a rough estimate labelled as such
        #[arg(long)]
        naive: bool,
        /// Override `[cycle-times].recent-window` for this invocation
        #[arg(long = "sample-size")]
        sample_size: Option<usize>,
    },
    /// Gap analysis between ADRs, features, and tests
    Gap {
        #[command(subcommand)]
        command: GapCommands,
    },
    /// Graph operations
    Graph {
        #[command(subcommand)]
        command: GraphCommands,
    },
    /// Content hash operations (ADR-032)
    Hash {
        #[command(subcommand)]
        command: HashCommands,
    },
    /// Impact analysis
    Impact {
        /// Artifact ID (feature, ADR, or test)
        id: String,
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
        /// Disable Step 0a auto-fill of TC runner config (FT-068)
        #[arg(long = "no-auto-runners")]
        no_auto_runners: bool,
        /// Prompt template profile (FT-074). Use "legacy-template" for the
        /// pre-FT-074 bundle shape without Patterns / observes inline /
        /// ADR-051 hard-constraint line.
        #[arg(long)]
        target: Option<String>,
    },
    /// Initialize a new Product repository (ADR-033, ADR-048)
    Init {
        /// Accept all defaults without prompting
        #[arg(short = 'y', long)]
        yes: bool,
        /// Overwrite existing config file
        #[arg(long)]
        force: bool,
        /// Project name (default: directory name)
        #[arg(long)]
        name: Option<String>,
        /// Product responsibility — single statement of what the product is and is not (FT-039)
        #[arg(long, visible_alias = "responsibility", value_name = "TEXT")]
        description: Option<String>,
        /// Add a domain (repeatable): --domain security="Auth, secrets"
        #[arg(long = "domain", value_name = "K=V")]
        domains: Vec<String>,
        /// MCP HTTP port (default: 7777)
        #[arg(long, default_value = "7777")]
        port: u16,
        /// Enable MCP write tools by default
        #[arg(long)]
        write_tools: bool,
        /// Use the pre-FT-057 root-based layout (`product.toml` + `docs/...`).
        /// Default is the canonical `.product/` layout (ADR-048).
        #[arg(long)]
        legacy_layout: bool,
        /// Target directory (default: current directory)
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
    },
    /// Install git hooks and scaffolding
    InstallHooks,
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
    /// Architectural fitness functions
    Metrics {
        #[command(subcommand)]
        command: MetricsCommands,
    },
    /// Migration from monolithic documents
    Migrate {
        #[command(subcommand)]
        command: MigrateCommands,
    },
    /// Codebase onboarding — discover decisions from existing code (ADR-027)
    Onboard {
        #[command(subcommand)]
        command: OnboardCommands,
    },
    /// Pattern artifact management (FT-070, ADR-050)
    Pattern {
        #[command(subcommand)]
        command: PatternCommands,
    },
    /// Pre-flight analysis — check domain and cross-cutting coverage
    Preflight {
        /// Feature ID
        id: String,
    },
    /// Manage authoring session prompts (ADR-022)
    Prompts {
        #[command(subcommand)]
        command: PromptsCommands,
    },
    /// Unified atomic write interface (FT-041, ADR-038)
    Request {
        #[command(subcommand)]
        command: request_cmd::RequestCommands,
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
    /// Tag lifecycle — browse product/* git tags (ADR-036)
    Tags {
        #[command(subcommand)]
        command: tags::TagsCommands,
    },
    /// Test criterion navigation and management
    Test {
        #[command(subcommand)]
        command: TestCommands,
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
}

pub use self::test_cmd::TestCommands;

type BoxResult = Result<(), Box<dyn std::error::Error>>;

pub fn run(command: Commands, format: &str, cli_command: &mut clap::Command) -> BoxResult {
    shared::run_startup_hooks()?;
    dispatch::dispatch(command, format, cli_command)
}
