//! `product request log` / `replay` / `undo` (FT-042, ADR-039).

use clap::Subcommand;
use product_lib::config::ProductConfig;
use product_lib::fileops;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum LogCommands {
    /// List log entries (optionally filtered by --type or --feature)
    Show {
        /// Filter by entry type (create, change, verify, etc.)
        #[arg(long, name = "type")]
        type_filter: Option<String>,
        /// Filter by feature ID (only works for verify entries)
        #[arg(long)]
        feature: Option<String>,
        /// Show full detail for a specific REQ-ID
        #[arg(long)]
        show: Option<String>,
    },
    /// Verify the chain and every entry's hash
    Verify {
        /// Cross-reference git tags — detects tail truncation (W021)
        #[arg(long = "against-tags")]
        against_tags: bool,
    },
}

pub fn handle_log(cmd: LogCommands, _fmt: &str) -> BoxResult {
    use product_lib::request_log::{append, entry, log_path, migrate_if_needed};

    let (config, root) = ProductConfig::discover()?;
    let _ = migrate_if_needed(&root, Some(&config.paths.requests));
    let log_p = log_path(&root, Some(&config.paths.requests));

    match cmd {
        LogCommands::Show { type_filter, feature, show } => {
            let entries = match append::load_all_entries(&log_p) {
                Ok(v) => v,
                Err(_) => {
                    println!("No log at {}", log_p.display());
                    return Ok(());
                }
            };
            if let Some(ref req_id) = show {
                for (_, e) in entries.into_iter().flatten() {
                    if e.id == *req_id {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&e.to_value()).unwrap_or_default()
                        );
                        return Ok(());
                    }
                }
                eprintln!("No entry with id {}", req_id);
                std::process::exit(1);
            }
            println!("{:<22} {:<20} reason", "id", "type");
            for (_, e) in entries.into_iter().flatten() {
                if let Some(ref tf) = type_filter {
                    if e.entry_type.as_str() != tf.as_str() {
                        continue;
                    }
                }
                if let Some(ref ft) = feature {
                    if let entry::EntryPayload::Verify { feature: f, .. } = &e.payload {
                        if f != ft {
                            continue;
                        }
                    } else {
                        continue;
                    }
                }
                println!("{:<22} {:<20} {}", e.id, e.entry_type.as_str(), e.reason);
            }
        }
        LogCommands::Verify { against_tags } => {
            run_log_verify(&log_p, &root, against_tags);
        }
    }
    Ok(())
}

fn run_log_verify(log_p: &std::path::Path, root: &std::path::Path, against_tags: bool) {
    use product_lib::request_log::verify::{verify_log, Severity, VerifyOptions};
    if !log_p.exists() {
        println!("No log at {} — nothing to verify.", log_p.display());
        return;
    }
    let outcome = verify_log(log_p, root, &VerifyOptions { against_tags });
    let n = outcome.entry_count;
    println!("Verifying {} ({} entries)...", log_p.display(), n);
    if outcome.findings.is_empty() {
        println!("  \u{2713} Entry hashes valid ({}/{})", outcome.entry_hashes_valid, n);
        println!("  \u{2713} Hash chain intact ({}/{})", outcome.chain_links_valid, n);
        if against_tags {
            println!("  \u{2713} Tag cross-reference clean");
        }
        println!();
        println!("Log is tamper-free.");
        return;
    }
    println!("  Entry hashes valid ({}/{})", outcome.entry_hashes_valid, n);
    println!("  Hash chain intact  ({}/{})", outcome.chain_links_valid, n);
    for f in &outcome.findings {
        let sev = match f.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        eprint!("{}[{}]: ", sev, f.code);
        if let Some(ref id) = f.entry_id {
            eprint!("{} ", id);
        }
        eprintln!("{}", f.message);
        if let Some(line) = f.line {
            eprintln!("  --> {}:{}", log_p.display(), line);
        }
        if let Some(ref d) = f.detail {
            for l in d.lines() {
                eprintln!("  {}", l);
            }
        }
    }
    let code = outcome.exit_code();
    if code != 0 {
        std::process::exit(code);
    }
}

pub fn handle_replay(full: bool, to: Option<String>, output: Option<PathBuf>) -> BoxResult {
    use product_lib::request_log::{log_path, replay};
    let (config, root) = ProductConfig::discover()?;

    let out_dir = match output {
        Some(p) => {
            let canon_out = p.canonicalize().unwrap_or(p.clone());
            let canon_repo = root.canonicalize().unwrap_or(root.clone());
            if canon_out == canon_repo || canon_out.starts_with(&canon_repo) {
                eprintln!(
                    "error: refusing to replay into the working tree (`--output .` or a subdirectory)"
                );
                std::process::exit(1);
            }
            p
        }
        None => {
            let ts = chrono::Utc::now().timestamp();
            std::env::temp_dir().join(format!("product-replay-{}", ts))
        }
    };

    let summary = if full || to.is_none() {
        replay::replay_full(&root, Some(&config.paths.requests), &out_dir)?
    } else if let Some(to_id) = to {
        replay::replay_to(&root, Some(&config.paths.requests), &to_id, &out_dir)?
    } else {
        replay::replay_full(&root, Some(&config.paths.requests), &out_dir)?
    };
    let lp = log_path(&root, Some(&config.paths.requests));
    println!("Replaying {} ...", lp.display());
    println!(
        "  {} entries applied, {} truncated",
        summary.entries_applied, summary.entries_skipped
    );
    println!();
    println!("Replay complete. State written to {}", summary.output.display());
    println!("Run: product graph check --repo {}", summary.output.display());
    Ok(())
}

pub fn handle_undo(req_id: &str, reason: Option<&str>) -> BoxResult {
    use product_lib::request_log::{append, entry::Entry, log_path};
    let (config, root) = ProductConfig::discover()?;
    let _lock = fileops::RepoLock::acquire(&root)?;
    let log_p = log_path(&root, Some(&config.paths.requests));
    if !log_p.exists() {
        eprintln!("No log at {}", log_p.display());
        std::process::exit(1);
    }
    let entries = append::load_all_entries(&log_p)?;
    let mut target: Option<Entry> = None;
    for (_, e) in entries.into_iter().flatten() {
        if e.id == req_id {
            target = Some(e);
        }
    }
    let target = match target {
        Some(e) => e,
        None => {
            eprintln!("No entry with id {}", req_id);
            std::process::exit(1);
        }
    };

    let applied_by = product_lib::request_log::git_identity::resolve_applied_by(&root)
        .unwrap_or_else(|_| "local:unknown".into());
    let commit = product_lib::request_log::git_identity::resolve_commit(&root);
    let reason = reason
        .map(String::from)
        .unwrap_or_else(|| format!("Undo of {}", req_id));
    let inverse = serde_json::json!({
        "note": "inverse of target entry; artifact-level reversal is out of scope for v1 of FT-042",
        "target-type": target.entry_type.as_str(),
    });
    let written = append::append_undo_entry(
        &log_p,
        &applied_by,
        &commit,
        &reason,
        req_id,
        inverse,
    )?;
    println!("  Appended undo entry {} (undoes {})", written.id, req_id);
    Ok(())
}
