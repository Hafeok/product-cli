//! `product request` — unified atomic write interface (FT-041, ADR-038).

use clap::Subcommand;
use product_lib::config::ProductConfig;
use product_lib::fileops;
use product_lib::request::{self, ApplyOptions};
use std::path::{Path, PathBuf};

use super::request_builder_add::AddCommands as BuilderAddCommands;
use super::request_builder_cmd::{self, BuilderCommands};
use super::request_cmd_helpers::{
    dedup_findings, print_apply_summary, print_findings, print_json_result, resolve_file_or_draft,
    run_git_commit,
};
use super::request_log_cmd::{self, LogCommands};
use super::BoxResult;

#[derive(Subcommand)]
pub enum RequestCommands {
    /// Open $EDITOR with a create template in .product/requests/
    Create,
    /// Open $EDITOR with a change template in .product/requests/
    Change,
    /// Validate a request YAML without writing — reports every finding
    Validate {
        /// Path to the request YAML file (defaults to the active draft)
        file: Option<PathBuf>,
    },
    /// Validate and apply a request atomically
    Apply {
        /// Path to the request YAML file
        file: PathBuf,
        /// Apply and then commit (reason: used as commit message suffix)
        #[arg(long)]
        commit: bool,
    },
    /// Show what would change without writing
    Diff {
        /// Path to the request YAML file (defaults to the active draft)
        file: Option<PathBuf>,
    },
    /// List draft YAML files under .product/requests/
    Draft,
    /// Start a new interactive draft session (FT-052)
    New {
        /// Draft kind: "create" or "change"
        kind: String,
    },
    /// Resume the active interactive draft session (FT-052)
    #[command(name = "continue")]
    Continue,
    /// Remove the active interactive draft (FT-052)
    Discard {
        /// Skip confirmation
        #[arg(long)]
        force: bool,
    },
    /// Show the active draft's state with per-artifact indicators (FT-052)
    Status,
    /// Print the raw draft YAML to stdout (FT-052)
    Show,
    /// Submit the active draft — validate, apply, archive (FT-052)
    Submit {
        /// Submit through warnings without prompting
        #[arg(long)]
        force: bool,
    },
    /// Open the active draft in `$EDITOR` (FT-052)
    Edit,
    /// Append an artifact or change to the active draft (FT-052)
    Add {
        #[command(subcommand)]
        command: BuilderAddCommands,
    },
    /// View / verify the hash-chained request log (FT-042)
    Log {
        #[command(subcommand)]
        command: LogCommands,
    },
    /// Replay the log into a directory outside the working tree (FT-042)
    Replay {
        /// Replay all entries from genesis to head
        #[arg(long)]
        full: bool,
        /// Stop at this entry ID (inclusive)
        #[arg(long)]
        to: Option<String>,
        /// Output directory — must be outside the working tree
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Append an undo entry that reverses a past request (FT-042)
    Undo {
        /// Request ID to undo
        req_id: String,
        /// Reason for the undo
        #[arg(long)]
        reason: Option<String>,
    },
}

pub(crate) fn handle_request(cmd: RequestCommands, fmt: &str) -> BoxResult {
    match cmd {
        RequestCommands::Create => create_draft("create"),
        RequestCommands::Change => create_draft("change"),
        RequestCommands::Validate { file } => validate(file.as_deref(), fmt),
        RequestCommands::Apply { file, commit } => apply(&file, commit, fmt),
        RequestCommands::Diff { file } => diff(file.as_deref(), fmt),
        RequestCommands::Draft => list_drafts(),
        RequestCommands::New { kind } => {
            request_builder_cmd::handle_builder(BuilderCommands::New { kind })
        }
        RequestCommands::Continue => {
            request_builder_cmd::handle_builder(BuilderCommands::Continue)
        }
        RequestCommands::Discard { force } => {
            request_builder_cmd::handle_builder(BuilderCommands::Discard { force })
        }
        RequestCommands::Status => request_builder_cmd::handle_builder(BuilderCommands::Status),
        RequestCommands::Show => request_builder_cmd::handle_builder(BuilderCommands::Show),
        RequestCommands::Submit { force } => {
            request_builder_cmd::handle_builder(BuilderCommands::Submit { force })
        }
        RequestCommands::Edit => request_builder_cmd::handle_builder(BuilderCommands::Edit),
        RequestCommands::Add { command } => {
            request_builder_cmd::handle_builder(BuilderCommands::Add { command })
        }
        RequestCommands::Log { command } => request_log_cmd::handle_log(command, fmt),
        RequestCommands::Replay { full, to, output } => {
            request_log_cmd::handle_replay(full, to, output)
        }
        RequestCommands::Undo { req_id, reason } => {
            request_log_cmd::handle_undo(&req_id, reason.as_deref())
        }
    }
}

fn create_draft(kind: &str) -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    let dir = root.join(".product/requests");
    std::fs::create_dir_all(&dir)?;

    let ts = chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S");
    let filename = format!("{}-{}.yaml", ts, kind);
    let path = dir.join(&filename);

    let template = if kind == "create" {
        r#"type: create
schema-version: 1
reason: ""

artifacts:
  # - type: feature
  #   ref: ft-example
  #   title: Example Feature
  #   phase: 1
  #   domains: []
"#
    } else {
        r#"type: change
schema-version: 1
reason: ""

changes:
  # - target: FT-001
  #   mutations:
  #     - op: append
  #       field: domains
  #       value: api
"#
    };

    std::fs::write(&path, template)?;
    println!("Draft: {}", path.display());
    if let Ok(editor) = std::env::var("EDITOR") {
        if !editor.is_empty() && std::io::IsTerminal::is_terminal(&std::io::stdin()) {
            let _ = std::process::Command::new(editor).arg(&path).status();
        }
    }
    Ok(())
}

fn validate(file: Option<&Path>, fmt: &str) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let file = resolve_file_or_draft(file, &root)?;

    let features_dir = config.resolve_path(&root, &config.paths.features);
    let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
    let tests_dir = config.resolve_path(&root, &config.paths.tests);
    let deps_dir = config.resolve_path(&root, &config.paths.dependencies);
    let loaded = product_lib::parser::load_all_with_deps(
        &features_dir, &adrs_dir, &tests_dir, Some(&deps_dir),
    )?;
    let graph = product_lib::graph::KnowledgeGraph::build_with_deps(
        loaded.features, loaded.adrs, loaded.tests, loaded.dependencies,
    );

    let request = match request::parse_request(&file) {
        Ok(r) => r,
        Err(findings) => {
            print_findings(&findings, fmt);
            std::process::exit(1);
        }
    };

    let ctx = request::ValidationContext { config: &config, graph: &graph };
    let mut findings = request::validate_request(&request, &ctx);

    let apply_result = request::apply_request(
        &request,
        &config,
        &root,
        ApplyOptions { dry_run: true, skip_git_identity: true },
    );
    findings.extend(apply_result.findings);
    dedup_findings(&mut findings);

    print_findings(&findings, fmt);
    if findings.iter().any(|f| f.is_error()) {
        std::process::exit(1);
    }
    println!("  validate: clean ({} warning(s))", findings.len());
    Ok(())
}

fn apply(file: &Path, commit: bool, fmt: &str) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let _lock = fileops::RepoLock::acquire(&root)?;

    let request = match request::parse_request(file) {
        Ok(r) => r,
        Err(findings) => {
            print_findings(&findings, fmt);
            std::process::exit(1);
        }
    };

    let result = request::apply_request(&request, &config, &root, ApplyOptions::default());

    if fmt == "json" {
        print_json_result(&result);
        if !result.applied {
            std::process::exit(1);
        }
        return Ok(());
    }

    print_findings(&result.findings, fmt);
    if !result.applied {
        std::process::exit(1);
    }

    print_apply_summary(&result);

    if commit {
        run_git_commit(&root, &request.reason);
    }
    Ok(())
}

fn diff(file: Option<&Path>, fmt: &str) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let file = resolve_file_or_draft(file, &root)?;
    let request = match request::parse_request(&file) {
        Ok(r) => r,
        Err(findings) => {
            print_findings(&findings, fmt);
            std::process::exit(1);
        }
    };
    let result = request::apply_request(
        &request,
        &config,
        &root,
        ApplyOptions { dry_run: true, skip_git_identity: true },
    );
    if fmt == "json" {
        print_json_result(&result);
        return Ok(());
    }
    println!("# Request diff");
    println!("reason: {}", request.reason);
    println!("type:   {}", request.request_type);
    println!("artifacts to create: {}", request.artifacts.len());
    println!("changes:             {}", request.changes.len());
    print_findings(&result.findings, fmt);
    Ok(())
}

fn list_drafts() -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    let dir = root.join(".product/requests");
    if !dir.exists() {
        println!("No drafts found at {}", dir.display());
        return Ok(());
    }
    let mut entries: Vec<_> = std::fs::read_dir(&dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("yaml"))
        .collect();
    entries.sort_by_key(|e| {
        e.metadata()
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    entries.reverse();
    if entries.is_empty() {
        println!("No drafts at {}", dir.display());
        return Ok(());
    }
    for e in entries {
        let p = e.path();
        let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("?");
        let mut type_s = String::new();
        let mut reason_s = String::new();
        if let Ok(content) = std::fs::read_to_string(&p) {
            if let Ok(v) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                if let Some(t) = v.get("type").and_then(|x| x.as_str()) {
                    type_s = t.to_string();
                }
                if let Some(r) = v.get("reason").and_then(|x| x.as_str()) {
                    reason_s = r.to_string();
                }
            }
        }
        if type_s.is_empty() && reason_s.is_empty() {
            println!("  {}  (unparseable)", fname);
        } else {
            println!("  {}  [{}]  {}", fname, type_s, reason_s);
        }
    }
    Ok(())
}
