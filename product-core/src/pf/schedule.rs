//! Dependency-ordered scheduling of work units (§5 — `derived_from` is the edge).
//!
//! A work unit carries its inputs in `context.derived_from`; when one names
//! another unit (by work-unit id or originating cell id) that is a dependency.
//! `layers` topologically groups the units so a unit runs only after the units
//! it is derived from — within a layer units are independent and parallelisable,
//! across layers they sequence (so a `write-test` unit's artifact is frozen
//! before the `implement` unit that depends on it runs). Unknown references and
//! cycles are tolerated: the still-pending units release into one final layer so
//! a build never deadlocks on a stray pointer.

use super::work_unit::WorkUnit;

/// Does a `derived_from` entry `dep` reference the unit whose id is `id`? The
/// entry may be a full work-unit id or the originating cell id (a prefix of the
/// slugged unit id); a `kind:` prefix (domain/slot/behaviour) is stripped first.
pub fn references(dep: &str, id: &str) -> bool {
    let d = dep.rsplit(':').next().unwrap_or(dep);
    d == id || id.starts_with(&format!("{d}-")) || d.starts_with(&format!("{id}-"))
}

/// For each unit, the indices of the other units it derives from.
fn deps_of(units: &[WorkUnit]) -> Vec<Vec<usize>> {
    units
        .iter()
        .map(|u| {
            (0..units.len())
                .filter(|&j| units[j].id != u.id)
                .filter(|&j| u.context.derived_from.iter().any(|dep| references(dep, &units[j].id)))
                .collect()
        })
        .collect()
}

/// Topologically layer the units: every unit in layer *k* derives only from
/// units in layers `< k`. Indices are into `units`.
pub fn layers(units: &[WorkUnit]) -> Vec<Vec<usize>> {
    let deps = deps_of(units);
    let n = units.len();
    let mut placed = vec![false; n];
    let mut out: Vec<Vec<usize>> = Vec::new();
    let mut remaining = n;
    while remaining > 0 {
        let ready: Vec<usize> = (0..n)
            .filter(|&i| !placed[i] && deps[i].iter().all(|&d| placed[d]))
            .collect();
        // Empty ready set ⇒ a cycle or unresolved ref: release the rest at once.
        let layer = if ready.is_empty() { (0..n).filter(|&i| !placed[i]).collect() } else { ready };
        for &i in &layer {
            placed[i] = true;
        }
        remaining -= layer.len();
        out.push(layer);
    }
    out
}

#[cfg(test)]
#[path = "schedule_tests.rs"]
mod tests;
