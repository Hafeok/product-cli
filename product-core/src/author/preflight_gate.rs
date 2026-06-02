//! Post-session preflight gate: refuse to auto-commit when an authored feature
//! still has cross-cutting, domain, or runner gaps. Mirrors the Step 0 gate in
//! `product implement` so the same gaps cannot slip past the author flow.

use crate::config::ProductConfig;
use crate::domains;
use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::parser;
use std::path::Path;
use std::process::Command;

/// Parse the output of `git status --porcelain <features-path>` and return the
/// FT-XXX identifiers extracted from each touched filename. Pure function —
/// kept separate from the I/O wrapper so it can be unit-tested.
pub(super) fn extract_touched_feature_ids(porcelain: &str) -> Vec<String> {
    let mut ids = Vec::new();
    for line in porcelain.lines() {
        // git status --porcelain format: two status chars, a space, then the
        // path (or "old -> new" for renames). We take whichever side ends in a
        // recognisable feature filename.
        let rest = match line.get(3..) {
            Some(r) => r.trim(),
            None => continue,
        };
        let path_part = rest.rsplit(" -> ").next().unwrap_or(rest);
        let stem = match Path::new(path_part).file_stem().and_then(|s| s.to_str()) {
            Some(s) => s,
            None => continue,
        };
        let parts: Vec<&str> = stem.splitn(3, '-').collect();
        if parts.len() >= 2 && parts[0] == "FT" && parts[1].chars().all(|c| c.is_ascii_digit()) {
            ids.push(format!("FT-{}", parts[1]));
        }
    }
    ids.sort();
    ids.dedup();
    ids
}

/// Run preflight against every feature file touched in this session. Returns
/// `Ok(())` when all touched features are clean (or none were touched), and
/// `Err(ProductError::ConfigError)` after printing a per-feature preflight
/// report when any are not clean. Caller must skip the auto-commit and
/// propagate the error so the process exits non-zero.
pub(super) fn run_post_session_gate(config: &ProductConfig, root: &Path) -> Result<()> {
    let touched_ids = git_touched_feature_ids(root, &config.paths.features);
    if touched_ids.is_empty() {
        return Ok(());
    }

    let graph = load_graph_from_disk(config, root)?;
    let failures = collect_preflight_failures(&graph, &touched_ids, config)?;
    if failures.is_empty() {
        return Ok(());
    }

    print_failure_report(&failures);
    Err(ProductError::ConfigError(
        "preflight not clean — author session blocked from committing".to_string(),
    ))
}

fn load_graph_from_disk(config: &ProductConfig, root: &Path) -> Result<KnowledgeGraph> {
    let features_dir = config.resolve_path(root, &config.paths.features);
    let adrs_dir = config.resolve_path(root, &config.paths.adrs);
    let tests_dir = config.resolve_path(root, &config.paths.tests);
    let deps_dir = config.resolve_path(root, &config.paths.dependencies);
    let loaded =
        parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;
    Ok(KnowledgeGraph::build_with_deps(
        loaded.features,
        loaded.adrs,
        loaded.tests,
        loaded.dependencies,
    ))
}

fn collect_preflight_failures(
    graph: &KnowledgeGraph,
    touched_ids: &[String],
    config: &ProductConfig,
) -> Result<Vec<(String, domains::PreflightResult)>> {
    let mut failures = Vec::new();
    for fid in touched_ids {
        // Deletions land in `git status` too — skip ids whose front-matter is
        // no longer in the graph.
        if !graph.features.contains_key(fid.as_str()) {
            continue;
        }
        let result = domains::preflight(graph, fid, &config.domains, &config.features.default_acknowledged_cross_cutting)?;
        if !result.is_clean {
            failures.push((fid.clone(), result));
        }
    }
    Ok(failures)
}

fn print_failure_report(failures: &[(String, domains::PreflightResult)]) {
    eprintln!();
    eprintln!(
        "error: preflight not clean for {} authored feature(s) — refusing to auto-commit",
        failures.len()
    );
    eprintln!();
    for (fid, result) in failures {
        eprintln!("─── {} ───", fid);
        eprintln!("{}", domains::render_preflight(result));
    }
    eprintln!("Resolve the gaps (link missing ADRs/TCs, or set `domains-acknowledged`");
    eprintln!("with a written reason), then re-run `product preflight FT-XXX` until clean.");
    eprintln!("Your changes remain on disk — they have NOT been committed.");
}

fn git_touched_feature_ids(root: &Path, features_path: &str) -> Vec<String> {
    let output = Command::new("git")
        .args(["status", "--porcelain", features_path])
        .current_dir(root)
        .output();
    match output {
        Ok(out) => extract_touched_feature_ids(&String::from_utf8_lossy(&out.stdout)),
        Err(_) => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_ft_ids_from_added_and_modified_lines() {
        let porcelain = "\
?? docs/features/FT-070-new-thing.md
 M docs/features/FT-012-existing.md
A  docs/features/FT-099-another.md
";
        let ids = extract_touched_feature_ids(porcelain);
        assert_eq!(ids, vec!["FT-012", "FT-070", "FT-099"]);
    }

    #[test]
    fn dedups_and_sorts_ids() {
        let porcelain = "\
 M docs/features/FT-005-a.md
?? docs/features/FT-005-a.md
 M docs/features/FT-001-b.md
";
        let ids = extract_touched_feature_ids(porcelain);
        assert_eq!(ids, vec!["FT-001", "FT-005"]);
    }

    #[test]
    fn ignores_non_feature_files_and_malformed_lines() {
        let porcelain = "\
 M docs/adrs/ADR-001-foo.md
?? docs/tests/TC-050-bar.md

 M docs/features/not-an-id.md
?? docs/features/FT-AB-non-numeric.md
";
        assert!(extract_touched_feature_ids(porcelain).is_empty());
    }

    #[test]
    fn handles_rename_arrow_form() {
        let porcelain = "R  docs/features/FT-010-old.md -> docs/features/FT-010-new.md\n";
        let ids = extract_touched_feature_ids(porcelain);
        assert_eq!(ids, vec!["FT-010"]);
    }

    #[test]
    fn empty_input_yields_no_ids() {
        assert!(extract_touched_feature_ids("").is_empty());
    }
}
