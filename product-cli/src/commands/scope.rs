//! `product scope …` — authoring scopes as bounded co-authors of the What (§14).
//!
//! An authoring scope declares which What-element kinds a tool (Figma, a legacy
//! schema, an Event-Modeling board) MAY author. `add` validates + vendors a
//! scope under `.product/authoring-scopes/<tool>.yaml`; `validate` re-checks a
//! stored one; `enforce` runs the §14.3 enforcement oracle over a tool
//! submission; `join` runs the §14.4 completeness join across every stored scope.

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use clap::Subcommand;
use product_core::error::ProductError;
use product_core::pf::authoring_scope::{validate_scope, AuthoringScope};
use product_core::pf::authoring_scope_enforce::{enforce, Submission};
use product_core::pf::authoring_scope_join::completeness_join;
use serde_json::json;

use super::output::{CmdResult, Output};

#[derive(Subcommand)]
pub enum ScopeCommands {
    /// Validate an authoring-scope file (§14.2) and vendor it under
    /// `.product/authoring-scopes/<tool>.yaml`
    Add {
        /// Path to the authoring-scope file (YAML or JSON)
        file: PathBuf,
        #[arg(long)]
        product: Option<String>,
    },
    /// Run the §14.3 enforcement oracle over a tool submission: accept in-scope
    /// authorship, reject the rest, split the gap
    Enforce {
        /// The stored scope's tool id
        tool: String,
        /// Path to the submission JSON (`authored` + `unauthored-candidates`)
        submission: PathBuf,
        #[arg(long)]
        product: Option<String>,
    },
    /// Completeness join (§14.4) across every stored scope: is each required
    /// kind authored by some sanctioned tool
    Join {
        /// Required kinds, comma-separated (e.g. aio,journey,decider)
        #[arg(long)]
        required: Option<String>,
        /// A file of required kinds (comma- or newline-separated)
        #[arg(long = "required-file")]
        required_file: Option<PathBuf>,
        /// Mark a tool's authored kinds: `tool=kind,kind` (repeatable)
        #[arg(long = "authored")]
        authored: Vec<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// List the stored authoring scopes
    List {
        #[arg(long)]
        product: Option<String>,
    },
    /// Show a stored authoring scope
    Show {
        /// The stored scope's tool id
        tool: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Validate a stored authoring scope: wholeness + kind vocabulary + the
    /// derived-kind rule (§14.2)
    Validate {
        /// The stored scope's tool id
        tool: String,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_scope(cmd: ScopeCommands) -> CmdResult {
    match cmd {
        ScopeCommands::Add { file, product } => add(&file, product.as_deref()),
        ScopeCommands::Enforce { tool, submission, product } => {
            enforce_cmd(&tool, &submission, product.as_deref())
        }
        ScopeCommands::Join { required, required_file, authored, product } => {
            join_cmd(required, required_file, &authored, product.as_deref())
        }
        ScopeCommands::List { product } => list(product.as_deref()),
        ScopeCommands::Show { tool, product } => show(&tool, product.as_deref()),
        ScopeCommands::Validate { tool, product } => validate(&tool, product.as_deref()),
    }
}

fn scopes_dir(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "authoring-scopes")
}

fn load(tool: &str, product: Option<&str>) -> Result<AuthoringScope, ProductError> {
    let path = scopes_dir(product).join(format!("{tool}.yaml"));
    let text = std::fs::read_to_string(&path).map_err(|_| {
        ProductError::NotFound(format!(
            "no authoring scope '{tool}' at {} — add one with `product scope add`",
            path.display()
        ))
    })?;
    AuthoringScope::from_yaml(&text)
}

fn findings_error(label: &str, findings: &[product_core::pf::Violation]) -> ProductError {
    let joined = findings
        .iter()
        .map(|f| format!("  - [{}] {}: {}", f.focus, f.path, f.message))
        .collect::<Vec<_>>()
        .join("\n");
    ProductError::ConfigError(format!("{label}: {} finding(s):\n{joined}", findings.len()))
}

fn add(file: &Path, product: Option<&str>) -> CmdResult {
    let _lock = super::shared::acquire_write_lock_typed()?;
    let text = std::fs::read_to_string(file)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", file.display())))?;
    let scope = AuthoringScope::from_yaml(&text)?;
    let findings = validate_scope(&scope);
    if !findings.is_empty() {
        return Err(findings_error(
            &format!("authoring scope '{}' is not whole (§14.2) — nothing saved", scope.tool),
            &findings,
        ));
    }
    let dir = scopes_dir(product);
    let path = dir.join(format!("{}.yaml", scope.tool));
    product_core::fileops::write_file_atomic(&path, &scope.to_yaml()?)?;
    Ok(Output::text(format!(
        "added authoring scope '{}' (adapter '{}') → {} — {} author(s), {} excluded",
        scope.tool,
        scope.adapter,
        path.display(),
        scope.authors.len(),
        scope.excluded.len(),
    )))
}

fn validate(tool: &str, product: Option<&str>) -> CmdResult {
    let scope = load(tool, product)?;
    let findings = validate_scope(&scope);
    if !findings.is_empty() {
        return Err(findings_error(&format!("authoring scope '{tool}'"), &findings));
    }
    Ok(Output::text(format!(
        "authoring scope '{tool}' is valid — {} author(s), {} excluded (§14.2)",
        scope.authors.len(),
        scope.excluded.len(),
    )))
}

fn show(tool: &str, product: Option<&str>) -> CmdResult {
    let scope = load(tool, product)?;
    let json = serde_json::to_value(&scope)
        .map_err(|e| ProductError::Internal(format!("serialize scope: {e}")))?;
    let text = format!(
        "scope: {}\nadapter: {}\nauthors: {}\nexcluded: {}\nprocess-slice: {}",
        scope.tool,
        scope.adapter,
        scope.authored_kinds().join(", "),
        scope.excluded.join(", "),
        scope.process_slice.as_deref().unwrap_or("(none)"),
    );
    Ok(Output::both(text, json))
}

fn list(product: Option<&str>) -> CmdResult {
    let mut names = stems(&scopes_dir(product));
    names.sort();
    if names.is_empty() {
        return Ok(Output::text(
            "(no authoring scopes — add one with `product scope add <file>`)",
        ));
    }
    Ok(Output::both(names.join("\n"), json!({ "scopes": names })))
}

fn enforce_cmd(tool: &str, submission_path: &Path, product: Option<&str>) -> CmdResult {
    let scope = load(tool, product)?;
    let text = std::fs::read_to_string(submission_path)
        .map_err(|e| ProductError::IoError(format!("{}: {e}", submission_path.display())))?;
    let submission = Submission::from_json(&text).map_err(ProductError::ConfigError)?;
    let (valid, findings) = enforce(&scope, &submission);
    let json = json!({
        "tool": scope.tool,
        "valid": valid,
        "findings": findings,
    });
    let summary = format!(
        "scope '{}' enforcement: {} — {} accepted, {} rejected-out-of-scope, {} unauthored-within-scope, {} outside-scope",
        scope.tool,
        if valid { "valid" } else { "INVALID (out-of-scope authorship)" },
        findings.accepted.len(),
        findings.rejected_out_of_scope.len(),
        findings.unauthored_within_scope.len(),
        findings.outside_scope.len(),
    );
    Ok(Output::both(summary, json))
}

fn join_cmd(
    required: Option<String>,
    required_file: Option<PathBuf>,
    authored: &[String],
    product: Option<&str>,
) -> CmdResult {
    let required_kinds = required_kinds(required, required_file)?;
    let scopes = load_all(product)?;
    let authored_by_tool = parse_authored(authored)?;
    let (complete, report) = completeness_join(&required_kinds, &scopes, &authored_by_tool);

    let mut lines = vec![format!(
        "completeness join over {} scope(s): {}",
        scopes.len(),
        if complete { "COMPLETE" } else { "incomplete" }
    )];
    for (kind, coverage) in &report {
        lines.push(format!("  {kind}: {}", coverage.status()));
    }
    let json = json!({
        "complete": complete,
        "scopes": scopes.iter().map(|s| s.tool.clone()).collect::<Vec<_>>(),
        "report": report,
    });
    Ok(Output::both(lines.join("\n"), json))
}

fn required_kinds(
    required: Option<String>,
    required_file: Option<PathBuf>,
) -> Result<Vec<String>, ProductError> {
    let raw = match (required, required_file) {
        (Some(r), _) => r,
        (None, Some(f)) => std::fs::read_to_string(&f)
            .map_err(|e| ProductError::IoError(format!("{}: {e}", f.display())))?,
        (None, None) => {
            return Err(ProductError::ConfigError(
                "pass --required <kind,kind,…> or --required-file <path>".to_string(),
            ))
        }
    };
    let kinds: Vec<String> = raw
        .split([',', '\n', ' ', '\t', '\r'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();
    if kinds.is_empty() {
        return Err(ProductError::ConfigError("no required kinds supplied".to_string()));
    }
    Ok(kinds)
}

fn parse_authored(authored: &[String]) -> Result<BTreeMap<String, HashSet<String>>, ProductError> {
    let mut out: BTreeMap<String, HashSet<String>> = BTreeMap::new();
    for spec in authored {
        let (tool, kinds) = spec.split_once('=').ok_or_else(|| {
            ProductError::ConfigError(format!("--authored '{spec}' must be `tool=kind,kind`"))
        })?;
        let set = kinds
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
        out.insert(tool.trim().to_string(), set);
    }
    Ok(out)
}

fn load_all(product: Option<&str>) -> Result<Vec<AuthoringScope>, ProductError> {
    let dir = scopes_dir(product);
    let mut tools = stems(&dir);
    tools.sort();
    tools.iter().map(|t| load(t, product)).collect()
}

fn stems(dir: &Path) -> Vec<String> {
    match std::fs::read_dir(dir) {
        Ok(it) => it
            .flatten()
            .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("yaml"))
            .filter_map(|e| e.path().file_stem().and_then(|s| s.to_str()).map(String::from))
            .collect(),
        Err(_) => Vec::new(),
    }
}
