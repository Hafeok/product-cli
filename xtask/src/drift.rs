//! Drift self-test: every registered Check must have a matching convention doc.

use std::path::Path;

use crate::checks::Registry;
use crate::conventions;
use crate::diagnostic::Diagnostic;

/// Verify the descriptor-vs-doc invariant.
///
/// For every registered check:
///   * `conventions/docs/{ID}.md` exists.
///   * Frontmatter `id` matches `Check::id()`.
///   * Frontmatter `title` matches `Check::title()`.
///   * `Check::help_url()` resolves to the GitHub blob URL of that file.
///   * Every `ADR-####` referenced in frontmatter resolves to a file under
///     `conventions/adr/`.
pub fn run(registry: &Registry, root: &Path) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for check in registry.iter() {
        let doc_path = conventions::doc_path(root, check.id());
        if !doc_path.exists() {
            diagnostics.push(
                Diagnostic::error(
                    "DRIFT",
                    format!(
                        "registered check {} has no matching convention doc",
                        check.id()
                    ),
                    doc_path.clone(),
                )
                .with_help(format!(
                    "create conventions/docs/{}.md with frontmatter id/title/severity/tier/mechanism/adrs",
                    check.id()
                )),
            );
            continue;
        }
        let frontmatter = match conventions::read(&doc_path) {
            Ok(f) => f,
            Err(e) => {
                diagnostics.push(Diagnostic::error("DRIFT", e, doc_path.clone()));
                continue;
            }
        };
        if frontmatter.id != check.id() {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                format!(
                    "frontmatter id `{}` does not match Check::id() `{}`",
                    frontmatter.id,
                    check.id()
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
        let expected_path = format!("conventions/docs/{}.md", check.id());
        if !expected_url.ends_with(&expected_path) {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                format!(
                    "Check::help_url() `{}` does not point at `{}`",
                    expected_url, expected_path
                ),
                doc_path.clone(),
            ));
        }
        if !matches!(frontmatter.severity.as_str(), "deny" | "warn") {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                format!(
                    "frontmatter severity `{}` is not `deny` or `warn`",
                    frontmatter.severity
                ),
                doc_path.clone(),
            ));
        }
        if !(1..=3).contains(&frontmatter.tier) {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                format!("frontmatter tier `{}` is not 1, 2, or 3", frontmatter.tier),
                doc_path.clone(),
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
                doc_path.clone(),
            ));
        }
        if frontmatter.applies_to.is_empty() {
            diagnostics.push(Diagnostic::error(
                "DRIFT",
                "frontmatter `applies_to` is empty; rule has no scope".to_string(),
                doc_path.clone(),
            ));
        }
        // `exclude` may legitimately be empty; we read it to ensure parse works.
        let _ = frontmatter.exclude.len();
        for adr in &frontmatter.adrs {
            if !conventions::adr_exists(root, adr) {
                diagnostics.push(Diagnostic::error(
                    "DRIFT",
                    format!("ADR `{adr}` referenced in frontmatter has no file under conventions/adr/"),
                    doc_path.clone(),
                ));
            }
        }
    }
    diagnostics
}
