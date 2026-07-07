//! Skip-if-clean dispatch (§5 idempotence over refined trees).
//!
//! A unit whose declared artifact is tracked and unmodified was realised by an
//! earlier build and committed — re-dispatching would overwrite refined state
//! (fix-round work, hand edits) with a fresh whole-file rewrite. `--redispatch`
//! forces every unit; untracked or dirty artifacts (and non-git trees) always
//! dispatch.

use std::path::Path;

use product_core::pf::work_unit::WorkUnit;

/// Partition the fleet into the units to dispatch, announcing what is skipped.
pub(super) fn to_dispatch(units: &[WorkUnit], redispatch: bool, root: &Path) -> Vec<WorkUnit> {
    if redispatch {
        return units.to_vec();
    }
    let (clean, live): (Vec<&WorkUnit>, Vec<&WorkUnit>) =
        units.iter().partition(|u| unit_is_clean(u, root));
    if !clean.is_empty() {
        println!(
            "Skipping {} clean unit(s) (artifact tracked + unmodified; --redispatch to force): {}",
            clean.len(),
            clean.iter().map(|u| u.id.as_str()).collect::<Vec<_>>().join(", ")
        );
    }
    live.into_iter().cloned().collect()
}

/// A unit is clean when its declared artifact exists, is tracked, and carries
/// no local modification.
fn unit_is_clean(wu: &WorkUnit, root: &Path) -> bool {
    let rel = wu.produces.path.trim();
    !rel.is_empty() && root.join(rel).is_file() && super::build::git_status(root, rel) == "unchanged"
}
