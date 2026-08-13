//! Subcommand surface for the `ledger` binary.
//!
//! L0 was the format plus the gate; L1 adds the authoring verbs and L2 the
//! graph. Every authoring verb is a thin shell over the same `ledger-core`
//! validators the gate calls — a verb refuses at write exactly what
//! `verify` would fail one command later, and no second copy of the rules
//! exists on this side of the crate boundary.

mod add;
mod basis_text;
mod common;
mod declare;
mod diff_cmd;
mod evolve;
mod graph_cmds;
mod init;
mod inspect;
mod merge_cmd;
mod resolve_cmd;
mod sign;
mod verify;

use std::path::PathBuf;

use clap::Subcommand;

/// Exit codes, CI-conventional: a gate that says no is a different outcome
/// from a gate that could not run, and CI has to be able to tell them apart.
pub const EXIT_OK: i32 = 0;
pub const EXIT_VIOLATIONS: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

/// The command surface. Keep the variant list sorted.
#[derive(Subcommand)]
pub enum Commands {
    /// Sign the latest version of a decision (identity from git config)
    Accept {
        /// The decision id, `dec:<namespace>/<ulid>`
        decision: String,
        /// When this signature goes stale (YYYY-MM-DD)
        #[arg(long, value_name = "DATE")]
        expires: Option<String>,
    },
    /// Mint a decision and file its first version into a set
    Add {
        #[arg(long, value_name = "SET")]
        set: String,
        #[arg(long)]
        statement: String,
        /// Owning scope of the new id; inferred when the store speaks one
        #[arg(long, value_name = "NS")]
        namespace: Option<String>,
        /// Allocation store: constraint | criterion | judgment
        #[arg(long, value_name = "STORE")]
        store: Option<String>,
        /// Discharge pointer(s), e.g. analyzer:DEC001 (repeatable)
        #[arg(long, value_name = "REF")]
        discharge: Vec<String>,
        /// Criterion stage: pr | dev | staging | prod
        #[arg(long, value_name = "STAGE")]
        stage: Option<String>,
        /// Expectation, required when a discharge is otel:
        #[arg(long)]
        expectation: Option<String>,
        /// The named actor, for a judgment
        #[arg(long, value_name = "IDENTITY")]
        actor: Option<String>,
        /// Up-only tolerance override, strictly above the set floor
        #[arg(long, value_name = "TIER")]
        tolerance_override: Option<String>,
        /// Basis pointer(s) this decision rests on (repeatable)
        #[arg(long, value_name = "BASIS")]
        based_on: Vec<String>,
        /// Claim(s) whose death reopens this decision — never ground
        #[arg(long, value_name = "REF")]
        revisit_if: Vec<String>,
        /// What this act was, recorded on the change-set
        #[arg(long)]
        note: Option<String>,
    },
    /// Allocate (or re-allocate) a decision by filing its next version
    Allocate {
        decision: String,
        /// Allocation store: constraint | criterion | judgment
        #[arg(long, value_name = "STORE")]
        store: String,
        #[arg(long, value_name = "REF")]
        discharge: Vec<String>,
        #[arg(long, value_name = "STAGE")]
        stage: Option<String>,
        #[arg(long)]
        expectation: Option<String>,
        #[arg(long, value_name = "IDENTITY")]
        actor: Option<String>,
    },
    /// Who signed which version of a decision, and who committed it
    Blame {
        decision: String,
    },
    /// Disposition coverage per set, per namespace, with supersession chains
    Coverage {
        /// Limit to one set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
        /// Judge expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Declare a decision set: floor, ground, owner
    Declare {
        #[arg(long, value_name = "ID")]
        set: String,
        /// Human title; defaults to the set id
        #[arg(long)]
        title: Option<String>,
        #[arg(long, value_name = "TIER")]
        tolerance_floor: String,
        /// characterised | uncharacterised
        #[arg(long, default_value = "characterised")]
        ground: String,
        /// Defaults to the git-config identity
        #[arg(long, value_name = "IDENTITY")]
        owner: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    /// Decision-level change between two revisions (semantic, never textual)
    Diff {
        /// `<ref>..<ref>`, or `<ref>` to compare against the working tree
        spec: String,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
    },
    /// Price an escape: exposure stated, review date set, acceptor claimed
    Escape {
        decision: String,
        #[arg(long)]
        exposure: String,
        #[arg(long, value_name = "DATE")]
        review_by: String,
    },
    /// Scaffold the .decisions/ store: the §5 layout plus the ignore line
    Init,
    /// Walk the change-sets in creation order
    Log {
        /// Only change-sets touching this set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
    },
    /// Reconcile divergent version chains: plan, arbitrate, or install
    Merge {
        /// Revision to plan a merge with (read-only; exit 1 on conflicts)
        rev: Option<String>,
        /// Present each conflict and record the arbitration as a ledger act
        #[arg(long)]
        resolve: bool,
        /// Register the merge driver and .decisions/.gitattributes
        #[arg(long)]
        install: bool,
        /// Emit the plan as JSON
        #[arg(long)]
        json: bool,
    },
    /// The git merge driver: three-way judge one .decisions/ file
    #[command(hide = true)]
    MergeDriver {
        /// %O — the merge ancestor's version of the file
        base: PathBuf,
        /// %A — ours; the result is left here
        ours: PathBuf,
        /// %B — theirs
        theirs: PathBuf,
        /// %P — the repo-relative pathname
        path: String,
    },
    /// Rebuild the RDF index under .decisions/index/ from the log
    Reindex,
    /// Restate a decision as a new version; prior acceptances become history
    Revise {
        decision: String,
        #[arg(long)]
        statement: String,
        #[arg(long, value_name = "BASIS")]
        based_on: Vec<String>,
        /// Reopen edge(s) this version states, replacing the inherited set
        #[arg(long, value_name = "REF")]
        revisit_if: Vec<String>,
        /// Drop every inherited reopen edge (the decision was reopened)
        #[arg(long)]
        no_revisit_if: bool,
        /// What this act was, recorded on the change-set
        #[arg(long)]
        note: Option<String>,
        /// The hash believed to be the tip; refused when stale (merge is L3)
        #[arg(long, value_name = "HASH")]
        parent: Option<String>,
    },
    /// Reverse a prior acceptance, with the reason on record
    Revoke {
        acceptance: String,
        #[arg(long)]
        reason: String,
    },
    /// Decisions on screen: content, edges with their arguments, filing,
    /// acceptance. One id, or a whole set or group in one pass.
    Show {
        decision: Option<String>,
        /// Every decision in one set
        #[arg(long, value_name = "SET")]
        set: Option<String>,
        /// Every decision in one weight class: mechanical | individual
        #[arg(long, value_name = "GROUP")]
        group: Option<String>,
        /// Emit the screens as JSON
        #[arg(long)]
        json: bool,
        /// Judge acceptance expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Every decision's disposition state, grouped by set
    Status {
        /// Judge expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
    },
    /// Record that one decision supersedes another (never amend)
    Supersede {
        /// The decision being superseded
        decision: String,
        /// The successor decision
        #[arg(long, value_name = "DEC")]
        by: String,
        #[arg(long)]
        reason: Option<String>,
    },
    /// Run the gate over the log (exit 1 on findings, 2 on failure to run)
    Verify {
        /// Limit to one gate: `readiness` blocks produce, `completeness`
        /// blocks release. Default runs every class.
        #[arg(long, value_name = "GATE")]
        gate: Option<String>,
        /// Emit the report as JSON
        #[arg(long)]
        json: bool,
        /// Judge acceptance expiry against this date instead of today
        #[arg(long, value_name = "DATE")]
        today: Option<String>,
        /// Skip the git blame pass (class L009)
        #[arg(long)]
        no_blame: bool,
    },
}

/// Dispatch, mapping every outcome to an exit code.
pub fn run(command: Commands, root: Option<PathBuf>) -> i32 {
    let result = dispatch(command, root);
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("error: {message}");
            EXIT_ERROR
        }
    }
}

/// Dispatch. Split in two along the line the store itself draws: verbs
/// that append to the log, and reads that never touch it.
fn dispatch(command: Commands, root: Option<PathBuf>) -> Result<i32, String> {
    match command {
        Commands::Accept { decision, expires } => sign::accept(root, &decision, expires.as_deref()),
        Commands::Add {
            set, statement, namespace, store, discharge, stage, expectation, actor,
            tolerance_override, based_on, revisit_if, note,
        } => add::run(root, add::Flags {
            set, statement, namespace, store, discharge, stage, expectation, actor,
            tolerance_override, based_on, revisit_if, note,
        }),
        Commands::Allocate { decision, store, discharge, stage, expectation, actor } => {
            evolve::allocate(root, &decision, &store, &discharge, stage.as_deref(), expectation, actor.as_deref())
        }
        Commands::Declare { set, title, tolerance_floor, ground, owner, notes } => {
            declare::run(root, declare::Flags { set, title, tolerance_floor, ground, owner, notes })
        }
        Commands::Escape { decision, exposure, review_by } => {
            evolve::escape(root, &decision, exposure, &review_by)
        }
        Commands::Init => init::run(root),
        Commands::Merge { rev, resolve, install, json } => {
            merge_cmd::run(root, merge_cmd::Flags { rev, resolve, install, json })
        }
        Commands::MergeDriver { base, ours, theirs, path } => {
            merge_cmd::driver(&base, &ours, &theirs, &path)
        }
        Commands::Reindex => graph_cmds::reindex(root),
        Commands::Revise {
            decision, statement, based_on, revisit_if, no_revisit_if, note, parent
        } => {
            let edges = evolve::ReviseEdges {
                based_on: &based_on, revisit_if: &revisit_if, no_revisit_if, note,
            };
            evolve::revise(root, &decision, statement, edges, parent.as_deref())
        }
        Commands::Revoke { acceptance, reason } => sign::revoke(root, &acceptance, reason),
        Commands::Supersede { decision, by, reason } => {
            evolve::supersede(root, &decision, &by, reason)
        }
        read => dispatch_read(read, root),
    }
}

/// The read-only projections: nothing here appends to the log.
fn dispatch_read(command: Commands, root: Option<PathBuf>) -> Result<i32, String> {
    match command {
        Commands::Blame { decision } => inspect::blame(root, &decision),
        Commands::Coverage { set, json, today } => {
            graph_cmds::coverage(root, set.as_deref(), json, today.as_deref())
        }
        Commands::Diff { spec, json } => diff_cmd::run(root, &spec, json),
        Commands::Log { set } => inspect::log(root, set.as_deref()),
        Commands::Show { decision, set, group, json, today } => {
            inspect::show(root, inspect::ShowFlags { decision, set, group, json, today })
        }
        Commands::Status { today } => inspect::status(root, today.as_deref()),
        Commands::Verify { gate, json, today, no_blame } => {
            verify::run(root, verify::Args { gate, json, today, blame: !no_blame })
        }
        // Every writing verb is handled by `dispatch`, which routes here
        // only for what is left. Reaching this arm means a variant was
        // added to the enum and to neither match — reported, never panicked.
        _ => Err("internal: this subcommand is not wired into either dispatch half".to_string()),
    }
}

/// Resolve the repo root: the flag, else the first ancestor with a store.
pub fn resolve_root(root: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(explicit) = root {
        return Ok(explicit);
    }
    let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
    ledger_core::store::find_root(&cwd).ok_or_else(|| {
        format!(
            "no {}/ found here or in any parent — run `ledger init` to scaffold one",
            ledger_core::STORE_DIR
        )
    })
}
