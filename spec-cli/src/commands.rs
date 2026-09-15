//! Subcommand surface, mirroring the flow's verbs.
//!
//! Only the non-delegable half lives here. `import` and the drafting side of
//! `implement` are the agent host's, reached over MCP; this binary is the half
//! a model may not call, which is why the process boundary is the
//! accountability boundary.

use std::path::Path;

use clap::Subcommand;

use crate::{exit, render, verbs};

#[derive(Subcommand)]
pub enum Commands {
    /// Ratify a candidate as an act. Names a principal.
    Accept(verbs::AcceptArgs),
    /// List candidates with what was observed and what is unfilled.
    Candidates(verbs::CandidatesArgs),
    /// The CI gate: judge the store against the closed class set.
    Check(verbs::CheckArgs),
    /// Close an act-time record. Names a principal; a machine cannot be one.
    Close(verbs::CloseArgs),
    /// Open an act-time record for a slice built against the specification.
    Implement(verbs::ImplementArgs),
    /// Join acts to entry points and report the disagreements.
    Map(verbs::MapArgs),
    /// Show or file the check policy.
    #[command(subcommand)]
    Policy(PolicyCommands),
    /// List act-time records.
    Records(verbs::RecordsArgs),
    /// Refuse a candidate, filing the reason. Names a principal.
    Reject(verbs::RejectArgs),
    /// Trusted signing keys.
    #[command(subcommand)]
    Trust(TrustCommands),
}

/// The trust family.
#[derive(Subcommand)]
pub enum TrustCommands {
    /// Generate a keypair, trust the public half, write the secret outside the repo.
    Generate(verbs::TrustGenerateArgs),
    /// List trusted keys.
    List(verbs::TrustListArgs),
}

/// The policy family.
#[derive(Subcommand)]
pub enum PolicyCommands {
    /// File a new policy version. A change is a supersession, never an edit.
    Set(verbs::PolicySetArgs),
    /// Show the policy in force.
    Show(verbs::PolicyShowArgs),
}

/// Run one subcommand, returning its process exit code.
pub fn run(command: Commands, root: &Path) -> i32 {
    let outcome = match command {
        Commands::Accept(args) => verbs::accept(root, &args),
        Commands::Candidates(args) => verbs::candidates(root, &args),
        Commands::Check(args) => verbs::check(root, &args),
        Commands::Close(args) => verbs::close(root, &args),
        Commands::Implement(args) => verbs::implement(root, &args),
        Commands::Map(args) => verbs::map(root, &args),
        Commands::Policy(PolicyCommands::Set(args)) => verbs::policy_set(root, &args),
        Commands::Policy(PolicyCommands::Show(args)) => verbs::policy_show(root, &args),
        Commands::Records(args) => verbs::records(root, &args),
        Commands::Reject(args) => verbs::reject(root, &args),
        Commands::Trust(TrustCommands::Generate(args)) => verbs::trust_generate(root, &args),
        Commands::Trust(TrustCommands::List(args)) => verbs::trust_list(root, &args),
    };
    match outcome {
        Ok(report) => {
            render::emit(&report);
            report.code
        }
        Err(e) => {
            eprintln!("{e}");
            exit::COULD_NOT_RUN
        }
    }
}
