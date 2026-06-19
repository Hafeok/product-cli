//! The top-level `Commands` subcommand enum.

use clap::Subcommand;
use std::path::PathBuf;

use super::*;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // subcommand enums vary widely in size; inherent spread
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
    /// Archetype (How + layout + cells) — assemble and validate
    Archetype {
        #[command(subcommand)]
        command: ArchetypeCommands,
    },
    /// Start a graph-aware authoring session
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
    },
    /// Build a deliverable — assemble the SPMC context, dispatch a worker, report done (§5)
    Build {
        /// The deliverable id
        deliverable: String,
        /// The worker role to resolve to a capability (default: implementer)
        #[arg(long, default_value = "implementer")]
        role: String,
        /// Max work units to dispatch concurrently (the §5 parallel unit)
        #[arg(long, default_value_t = 1)]
        jobs: usize,
        /// Assemble + show the context, worker, and gate status without dispatching
        #[arg(long)]
        dry_run: bool,
        /// Diagnose + fix the worker's Rust output with rust-analyzer (clippy) before gating
        #[arg(long)]
        lsp: bool,
        /// Skip the §6 verify step (running each acceptance criterion's runner)
        #[arg(long)]
        no_verify: bool,
        /// Max diagnose→fix rounds per gate before recording what stands
        #[arg(long, default_value_t = 3)]
        max_rounds: usize,
        /// Token budget — escalation stops once total tokens reach it
        #[arg(long)]
        budget: Option<u64>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Task types (cells) — validate, show, and list against What + How
    Cell {
        #[command(subcommand)]
        command: CellCommands,
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
    /// Two Pillars conformance — check the graph against spec clauses
    Conformance {
        #[command(subcommand)]
        command: ConformanceCommands,
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
    /// Decider (§3.3) — derive an aggregate's executable signature, validate it
    Decider {
        #[command(subcommand)]
        command: DeciderCommands,
    },
    /// Delivery feature — one slice plus its acceptance (§7.1)
    Deliverable {
        #[command(subcommand)]
        command: DeliverableCommands,
    },
    /// Dependency management (ADR-030)
    Dep {
        #[command(subcommand)]
        command: DepCommands,
    },
    /// Domain (What) graph — list, show, and CRUD over captured artifacts
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
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
    /// Onboarding — where you are in the framework journey and the next step
    Guide,
    /// Content hash operations (ADR-032)
    Hash {
        #[command(subcommand)]
        command: HashCommands,
    },
    /// How contract (§4 architecture) — validate, show, and project
    How {
        #[command(subcommand)]
        command: HowCommands,
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
    /// rust-analyzer code intelligence — diagnostics, symbols, references
    Lsp {
        #[command(subcommand)]
        command: LspCommands,
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
    Primitive {
        #[command(subcommand)]
        command: PrimitiveCommands,
    },
    Projector {
        #[command(subcommand)]
        command: ProjectorCommands,
    },
    /// Manage authoring session prompts (ADR-022)
    Prompts {
        #[command(subcommand)]
        command: PromptsCommands,
    },
    /// Release — a coherent set of delivery features (§7.1)
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
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
    /// Delivery slice — a saved pointer to a section of the event model
    Slice {
        #[command(subcommand)]
        command: SliceCommands,
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
    /// Verify test criteria — six-stage pipeline (FT-044) when no feature ID is supplied, else per-feature (ADR-021)
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
    /// Work units (SPMC) — validate, show, and scaffold
    WorkUnit {
        #[command(subcommand)]
        command: WorkUnitCommands,
    },
    /// Worker capability catalog — list, resolve a role, validate, scaffold
    Worker {
        #[command(subcommand)]
        command: WorkerCommands,
    },
}
