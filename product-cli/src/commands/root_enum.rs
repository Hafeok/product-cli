//! The top-level `Commands` subcommand enum.

use clap::Subcommand;
use std::path::PathBuf;

use super::*;

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)] // subcommand enums vary widely in size; inherent spread
pub enum Commands {
    /// Start a graph-aware authoring session
    Author {
        #[command(subcommand)]
        command: AuthorCommands,
    },
    /// Blueprint (How + layout + cells) — assemble and validate
    #[command(visible_alias = "archetype")]
    Blueprint {
        #[command(subcommand)]
        command: BlueprintCommands,
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
        /// Emit a self-contained SPMC prompt for a Claude Code -p session instead of dispatching
        #[arg(long = "emit-spmc")]
        emit_spmc: bool,
        /// Emit the work units as §5.1 build-seam envelopes (by value + content-hash) instead of dispatching
        #[arg(long = "emit-seam")]
        emit_seam: bool,
        /// Where to write the SPMC (`-` for stdout); default .product/build/<id>.spmc.md
        #[arg(long)]
        out: Option<std::path::PathBuf>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Task types (cells) — validate, show, and list against What + How
    Cell {
        #[command(subcommand)]
        command: CellCommands,
    },
    /// Generate shell completions
    Completions {
        /// Shell: bash, zsh, fish
        shell: String,
    },
    /// Decider (§3.3) — derive an aggregate's executable signature, validate it
    Decider {
        #[command(subcommand)]
        command: DeciderCommands,
    },
    /// Deliverable — one feature plus its acceptance (§7.1)
    Deliverable {
        #[command(subcommand)]
        command: DeliverableCommands,
    },
    /// DeployableUnit (§4/§4.2) — the concrete artifact a blueprint produces for a system
    DeployableUnit {
        #[command(subcommand)]
        command: DeployableUnitCommands,
    },
    /// Design system (§11) — vendor a manifest, validate its bundle, bind it to the How
    DesignSystem {
        #[command(subcommand)]
        command: DesignSystemCommands,
    },
    /// Domain (What) graph — list, show, and CRUD over captured artifacts
    Domain {
        #[command(subcommand)]
        command: DomainCommands,
    },
    /// Delivery feature — a saved pointer to a subgraph of the event model (§7.1)
    Feature {
        #[command(subcommand)]
        command: FeatureCommands,
    },
    /// Onboarding — where you are in the framework journey and the next step
    Guide,
    /// How contract (§4 architecture) — validate, show, and project
    How {
        #[command(subcommand)]
        command: HowCommands,
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
        /// MCP HTTP port (default: 7777)
        #[arg(long, default_value = "7777")]
        port: u16,
        /// Enable MCP write tools by default
        #[arg(long)]
        write_tools: bool,
        /// Pre-FT-057 root layout (`product.toml` + `docs/...`); default is canonical `.product/` (ADR-048)
        #[arg(long)]
        legacy_layout: bool,
        /// Target directory (default: current directory)
        #[arg(long, value_name = "DIR")]
        path: Option<PathBuf>,
        /// Seed a worked example (the bookstore What model) for demos and workshops
        #[arg(long)]
        demo: bool,
        /// Skip writing the bundled What→How→Build skills into `.claude/skills/`
        #[arg(long)]
        no_skills: bool,
        /// Default agent CLI for `product session start`: claude | copilot
        /// (writes `[author].cli`)
        #[arg(long, value_name = "CLI")]
        cli: Option<String>,
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
        /// Phase-gate the tool surface for a What→How→Build session
        #[arg(long)]
        workflow: bool,
        /// Drive the named workflow session (implies --workflow)
        #[arg(long)]
        session: Option<String>,
    },
    /// Preview profiles (§11/§12) — validate a design-system or content-store manifest
    Preview {
        #[command(subcommand)]
        command: PreviewCommands,
    },
    Primitive {
        #[command(subcommand)]
        command: PrimitiveCommands,
    },
    /// Products — list, add, and inspect the homes under .product/products/
    Product {
        #[command(subcommand)]
        command: ProductCommands,
    },
    Projector {
        #[command(subcommand)]
        command: ProjectorCommands,
    },
    /// Reify (§4.5) — project the What graph into a verifiable implementation frame
    Reify {
        #[command(subcommand)]
        command: ReifyCommands,
    },
    /// Release — a coherent set of delivery features (§7.1)
    Release {
        #[command(subcommand)]
        command: ReleaseCommands,
    },
    /// Seam verification (§6.3) — verify a UI step's screen and its What agree
    Seam {
        /// The UI step id
        id: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// What→How→Build session — start, list, show, and resume a guided session
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    /// Claude Code skills — install the bundled What→How→Build phase guides
    Skills {
        #[command(subcommand)]
        command: SkillsCommands,
    },
    /// Target version (§7.3) — a future partition of features + its gap
    Target {
        #[command(subcommand)]
        command: TargetCommands,
    },
    /// Verdict (§5.1) — validate a build-seam verdict event the executor emitted
    Verdict {
        /// Path to the verdict event JSON
        file: PathBuf,
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
