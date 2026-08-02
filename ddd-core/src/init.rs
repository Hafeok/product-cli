//! `.ddd/` scaffolding — plan the PRD §6 layout, then apply it atomically.
//!
//! `plan_init` inspects what already exists so `ddd init` stays idempotent:
//! present files are skipped, never clobbered. `apply_init` commits the plan
//! through `fileops::write_batch_atomic`. Manifest skeletons start empty;
//! detection-driven generation (each rule marked `UNGOVERNED`) is M2.

use std::path::{Path, PathBuf};

use product_core::error::Result;
use product_core::fileops::write_batch_atomic;

use crate::store::STORE_DIR;

/// The §6 store layout, one subdirectory per entry family.
pub const LAYOUT_DIRS: &[&str] =
    &["predicates", "claims", "decisions", "manifest", "patterns", "seams", "shared"];

const CONFIG_TEMPLATE: &str = "\
# .ddd/config.yaml — repo-level DDD governance configuration (format v1).
format: 1

# Interception mode for contract-surface edits: enforce | warn | off (PRD §8).
intercept: warn

# Globs the interceptor plus detection skip.
ignore: []
";

const MANIFEST_TEMPLATE: &str = "\
# One entry per enabled rule, each mapping to a decision (PRD §6 rule 4).
# Entries marked `governance: UNGOVERNED` warn until a decision is filed.
format: 1
rules: []
";

/// The intended scaffold: files to create plus files left untouched.
pub struct InitPlan {
    /// The `.ddd/` directory the plan targets.
    pub ddd_dir: PathBuf,
    /// Files the plan will write (path, content).
    pub creates: Vec<(PathBuf, String)>,
    /// Files that already exist; `apply_init` never touches them.
    pub skipped: Vec<PathBuf>,
}

/// Plan the scaffold for `repo_root`, skipping whatever already exists.
pub fn plan_init(repo_root: &Path) -> InitPlan {
    let ddd = repo_root.join(STORE_DIR);
    let mut plan = InitPlan { ddd_dir: ddd.clone(), creates: Vec::new(), skipped: Vec::new() };
    let file = |rel: PathBuf, content: &str, plan: &mut InitPlan| {
        let path = ddd.join(rel);
        if path.exists() {
            plan.skipped.push(path);
        } else {
            plan.creates.push((path, content.to_string()));
        }
    };
    file(PathBuf::from("config.yaml"), CONFIG_TEMPLATE, &mut plan);
    file(PathBuf::from("manifest/analyzers.yaml"), MANIFEST_TEMPLATE, &mut plan);
    file(PathBuf::from("manifest/linter.yaml"), MANIFEST_TEMPLATE, &mut plan);
    for dir in LAYOUT_DIRS {
        if *dir == "manifest" {
            continue;
        }
        file(PathBuf::from(dir).join(".gitkeep"), "", &mut plan);
    }
    plan
}

/// Write every planned file in one atomic batch; returns how many landed.
pub fn apply_init(plan: &InitPlan) -> Result<usize> {
    let writes: Vec<(&Path, &str)> =
        plan.creates.iter().map(|(p, c)| (p.as_path(), c.as_str())).collect();
    write_batch_atomic(&writes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolds_the_layout_once_then_skips() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let plan = plan_init(tmp.path());
        let n = apply_init(&plan).expect("apply");
        assert_eq!(n, plan.creates.len());
        for dir in LAYOUT_DIRS {
            assert!(tmp.path().join(".ddd").join(dir).is_dir(), "missing {dir}");
        }
        assert!(tmp.path().join(".ddd/config.yaml").is_file());
        assert!(tmp.path().join(".ddd/manifest/analyzers.yaml").is_file());

        let again = plan_init(tmp.path());
        assert!(again.creates.is_empty(), "second init must create nothing");
        assert_eq!(again.skipped.len(), plan.creates.len());
    }

    #[test]
    fn the_scaffolded_store_validates_clean() {
        let tmp = tempfile::tempdir().expect("tempdir");
        apply_init(&plan_init(tmp.path())).expect("apply");
        let store = crate::store::load(&tmp.path().join(".ddd"));
        let vs = crate::validate::validate_store(&store);
        assert!(vs.is_empty(), "{vs:?}");
    }
}
