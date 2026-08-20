//! Committing a planned extraction to a registry instance's working tree.
//!
//! Refuses a non-conformant plan: a proposal that violates the instance's
//! own shapes must not reach a branch, because a reviewer reading per-triple
//! diffs would be reviewing content the registry's CI will reject anyway.
//! Writes are atomic, one file per assertion, and nothing outside the
//! target graph directory is touched.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{ProductError, Result};
use crate::fileops;

use super::plan::ExtractionPlan;

/// What the run wrote.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ExtractionReport {
    pub instance: PathBuf,
    pub written: Vec<String>,
    /// Files the plan produced that were already on disk, byte-identical —
    /// a re-run at the same ref, correctly writing nothing.
    pub unchanged: Vec<String>,
}

/// Write the plan's files into `instance`.
pub fn apply_extraction(plan: &ExtractionPlan, instance: &Path) -> Result<ExtractionReport> {
    if !plan.conformant() {
        return Err(ProductError::ConfigError(format!(
            "the proposal violates the instance's own shapes ({} finding(s)); nothing written",
            plan.shacl.len()
        )));
    }
    let mut written = Vec::new();
    let mut unchanged = Vec::new();
    let mut batch: Vec<(PathBuf, String)> = Vec::new();
    for file in &plan.files {
        let path = instance.join(&file.path);
        if std::fs::read_to_string(&path).is_ok_and(|on_disk| on_disk == file.contents) {
            unchanged.push(file.path.clone());
            continue;
        }
        batch.push((path, file.contents.clone()));
        written.push(file.path.clone());
    }
    for (path, contents) in &batch {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ProductError::IoError(format!("cannot create {}: {e}", parent.display()))
            })?;
        }
        fileops::write_file_atomic(path, contents)?;
    }
    Ok(ExtractionReport { instance: instance.to_path_buf(), written, unchanged })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ground::plan::PlannedFile;
    use crate::registry::{Finding, FindingKind};

    fn plan_with(files: Vec<PlannedFile>, shacl: Vec<Finding>) -> ExtractionPlan {
        ExtractionPlan {
            corpus: "corpus-backend".into(),
            git_ref: "deadbeef".into(),
            assertions: Vec::new(),
            rows: Vec::new(),
            fixpoint: crate::ground::entail::FixpointReport {
                rounds: 0,
                parses: 0,
                executions: 0,
                elapsed_ms: 0,
                seed_triples: 0,
                rules: Vec::new(),
            },
            files,
            shacl,
            constraints_evaluated: 0,
            already_ratified: 0,
            sidecar_entries: 0,
            batches: Vec::new(),
            notes: Default::default(),
        }
    }

    #[test]
    fn writes_one_file_per_assertion() {
        let tmp = tempfile::tempdir().expect("tmp");
        let plan = plan_with(
            vec![PlannedFile {
                path: "graphs/canonical/assertion-abc.ttl".into(),
                contents: "# one\n".into(),
            }],
            Vec::new(),
        );
        let report = apply_extraction(&plan, tmp.path()).expect("apply");
        assert_eq!(report.written, ["graphs/canonical/assertion-abc.ttl"]);
        assert!(tmp.path().join("graphs/canonical/assertion-abc.ttl").exists());
    }

    /// A re-run at the same ref writes nothing: identity is content-derived,
    /// so the file already on disk is the same file.
    #[test]
    fn a_second_run_over_the_same_facts_is_a_no_op() {
        let tmp = tempfile::tempdir().expect("tmp");
        let plan = plan_with(
            vec![PlannedFile {
                path: "graphs/canonical/assertion-abc.ttl".into(),
                contents: "# one\n".into(),
            }],
            Vec::new(),
        );
        apply_extraction(&plan, tmp.path()).expect("first");
        let second = apply_extraction(&plan, tmp.path()).expect("second");
        assert!(second.written.is_empty(), "{second:?}");
        assert_eq!(second.unchanged, ["graphs/canonical/assertion-abc.ttl"]);
    }

    #[test]
    fn a_non_conformant_plan_writes_nothing_at_all() {
        let tmp = tempfile::tempdir().expect("tmp");
        let plan = plan_with(
            vec![PlannedFile {
                path: "graphs/canonical/assertion-abc.ttl".into(),
                contents: "# one\n".into(),
            }],
            vec![Finding {
                kind: FindingKind::ShapeViolation,
                file: None,
                focus: "reg:assertion-abc".into(),
                message: "a Reading carries an assurance".into(),
            }],
        );
        assert!(apply_extraction(&plan, tmp.path()).is_err());
        assert!(!tmp.path().join("graphs").exists(), "no directory was created");
    }
}
