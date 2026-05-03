//! Drift self-test: every registered Check must have a matching convention doc.

use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::checks::Registry;
use crate::conventions;
use crate::diagnostic::Diagnostic;

/// Verify the descriptor-vs-doc invariant.
///
/// Two-way scan:
///
/// 1. For every registered `Check`:
///    * `conventions/docs/{ID}.md` exists.
///    * Frontmatter `id` matches `Check::id()`.
///    * Frontmatter `title` matches `Check::title()`.
///    * `Check::help_url()` resolves to the GitHub blob URL of that file.
///    * Frontmatter `severity` / `tier` / `mechanism` / `applies_to` parse to
///      known values.
///    * Every `ADR-####` referenced in frontmatter resolves to a file under
///      `conventions/adr/`.
///
/// 2. For every doc under `conventions/docs/`:
///    * Frontmatter parses cleanly.
///    * If `mechanism: xtask`, a `Check` with matching id is registered. An
///      orphan xtask doc is a drift error — the rule isn't actually enforced.
pub fn run(registry: &Registry, root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let mut registered_ids: HashSet<String> = HashSet::new();

    for check in registry.iter() {
        let id = check.id();
        registered_ids.insert(id.as_str().to_string());
        diagnostics.extend(validate_against_doc(check, root));
    }

    diagnostics.extend(validate_orphans(root, &registered_ids));
    diagnostics
}

fn validate_against_doc(check: &dyn crate::Check, root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let id = check.id();
    let doc_path = conventions::doc_path(root, id.as_str());
    if !doc_path.exists() {
        diagnostics.push(
            Diagnostic::error(
                "DRIFT",
                format!("registered check {id} has no matching convention doc"),
                doc_path.clone(),
            )
            .with_help(format!(
                "create conventions/docs/{id}.md with frontmatter id/title/severity/tier/mechanism/adrs"
            )),
        );
        return diagnostics;
    }
    let frontmatter = match conventions::read(&doc_path) {
        Ok(f) => f,
        Err(e) => {
            diagnostics.push(Diagnostic::error("DRIFT", e, doc_path));
            return diagnostics;
        }
    };
    if frontmatter.id != id.as_str() {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!(
                "frontmatter id `{}` does not match Check::id() `{}`",
                frontmatter.id, id
            ),
            doc_path.clone(),
        ));
    }
    if frontmatter.title != check.title() {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!(
                "frontmatter title `{}` does not match Check::title() `{}`",
                frontmatter.title,
                check.title()
            ),
            doc_path.clone(),
        ));
    }
    let expected_url = check.help_url();
    let expected_path = format!("conventions/docs/{id}.md");
    if !expected_url.ends_with(&expected_path) {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!("Check::help_url() `{expected_url}` does not point at `{expected_path}`"),
            doc_path.clone(),
        ));
    }
    diagnostics.extend(validate_frontmatter_shape(&frontmatter, &doc_path));
    for adr in &frontmatter.adrs {
        if !conventions::adr_exists(root, adr) {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                format!("ADR `{adr}` referenced in frontmatter has no file under conventions/adr/"),
                doc_path.clone(),
            ));
        }
    }
    diagnostics
}

fn validate_orphans(root: &Path, registered_ids: &HashSet<String>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let docs_dir = root.join("conventions").join("docs");
    let Ok(entries) = fs::read_dir(&docs_dir) else {
        return diagnostics;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "md") {
            continue;
        }
        let frontmatter = match conventions::read(&path) {
            Ok(f) => f,
            Err(e) => {
                diagnostics.push(Diagnostic::error("DRIFT", e, path));
                continue;
            }
        };
        diagnostics.extend(validate_frontmatter_shape(&frontmatter, &path));
        for adr in &frontmatter.adrs {
            if !conventions::adr_exists(root, adr) {
                diagnostics.push(Diagnostic::error(
                    "DRIFT",
                    format!("ADR `{adr}` referenced in frontmatter has no file under conventions/adr/"),
                    path.clone(),
                ));
            }
        }
        if frontmatter.mechanism == "xtask" && !registered_ids.contains(&frontmatter.id) {
            diagnostics.push(
                Diagnostic::error(
                    "DRIFT",
                    format!(
                        "doc `{}` declares mechanism `xtask` but no Check with that id is registered",
                        frontmatter.id
                    ),
                    path.clone(),
                )
                .with_help("register the check in xtask/src/checks/mod.rs::Registry::default_set, or change `mechanism` to the correct enforcement tier".to_string()),
            );
        }
    }
    diagnostics
}

fn validate_frontmatter_shape(
    frontmatter: &conventions::Frontmatter,
    path: &Path,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if !matches!(frontmatter.severity.as_str(), "deny" | "warn") {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!(
                "frontmatter severity `{}` is not `deny` or `warn`",
                frontmatter.severity
            ),
            path.to_path_buf(),
        ));
    }
    if !(1..=3).contains(&frontmatter.tier) {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!("frontmatter tier `{}` is not 1, 2, or 3", frontmatter.tier),
            path.to_path_buf(),
        ));
    }
    let known_mechanisms = ["type", "macro", "clippy", "xtask", "cargo-deny", "dylint"];
    if !known_mechanisms.contains(&frontmatter.mechanism.as_str()) {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            format!(
                "frontmatter mechanism `{}` is not one of {known_mechanisms:?}",
                frontmatter.mechanism
            ),
            path.to_path_buf(),
        ));
    }
    if frontmatter.applies_to.is_empty() {
        diagnostics.push(Diagnostic::error(
            "DRIFT",
            "frontmatter `applies_to` is empty; rule has no scope".to_string(),
            path.to_path_buf(),
        ));
    }
    // `exclude` may legitimately be empty; we read it to ensure parse works.
    let _ = frontmatter.exclude.len();
    diagnostics
}
