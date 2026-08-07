//! Server instructions — the framing a client loads at connection time.
//!
//! MCP hands `instructions` to the model once, before any tool call, so this
//! is the only place to say things an agent needs *before* it acts. It is
//! generated from repo state rather than hardcoded: a governance paragraph
//! that describes a gate the repo has not switched on would be a lie the
//! agent then plans around.

use std::path::Path;

const BASE: &str = "\
The Product framework graph for this repo lives under `.product/`: the What \
(domain model §3.1 + event model §3.2), the How (§4 architecture contract), \
and delivery (§7). Author it through these tools rather than editing the \
files by hand — every write is validated in-loop, and a shape violation \
rolls the whole node back, so supply all shape-required fields in one call.";

const PHASES: &str = "\
This is a phase-gated session: What -> How -> Build. `tools/list` shows only \
the current phase's family, and out-of-phase writes are rejected. Use \
`product_workflow_status` to see where you are, `product_workflow_advance` to \
move on, and `product_session_finalize` to validate and close.";

/// The instructions for a server rooted at `repo_root`.
pub fn instructions(repo_root: &Path, phase_gated: bool) -> String {
    let mut out = String::from(BASE);
    if phase_gated {
        out.push_str("\n\n");
        out.push_str(PHASES);
    }
    if let Some(gate) = governance(repo_root) {
        out.push_str("\n\n");
        out.push_str(&gate);
    }
    out
}

/// The governance paragraph, or `None` when this repo has no `.ddd/` store
/// or has switched interception off.
fn governance(repo_root: &Path) -> Option<String> {
    let store = ddd_core::store::load(&repo_root.join(ddd_core::store::STORE_DIR));
    let cfg = store.config.as_ref()?;
    let mode = cfg.intercept_for(crate::what_intercept::ARTIFACT_CLASS);
    match mode {
        "enforce" => Some(ENFORCE.to_string()),
        "warn" => Some(WARN.to_string()),
        _ => None,
    }
}

const ENFORCE: &str = "\
GOVERNANCE (intercept: enforce). This repo carries a `.ddd/` governance \
store, so What writes are classified against a boundary policy table before \
they land. A boundary is: a system (§3.2.5), a context mapping (§3.1), a \
journey crossing (§3.0.1), a quality demand (§3.6), or wiring a §3.2.0 \
Translation — which publishes the View it watches, the Command it issues, and \
the Events that View projects. Entities, value objects, views and UI steps \
are internal elaboration and are never gated; an event or command is internal \
until a Translation carries it.\n\n\
Forming a boundary with nothing declared for it returns `\"status\": \
\"rejected\"` and the edit is rolled back. When that happens: do NOT retry the \
edit unchanged, and do NOT work around it by renaming the node, splitting it, \
or switching to a different tool. Read `demands[].surface`, call \
`product_what_declare` with the supplied `template` plus a `verdict_knowledge` \
you author — one sentence on what this boundary lets the other side learn, \
which is deliberately never pre-filled because it is the judgment being asked \
for — then retry the identical edit. It will apply.";

const WARN: &str = "\
GOVERNANCE (intercept: warn). This repo carries a `.ddd/` governance store, \
so What writes are classified against a boundary policy table. Boundary-\
forming edits (a system §3.2.5, a context mapping §3.1, a journey crossing \
§3.0.1, a quality demand §3.6, or wiring a §3.2.0 Translation) still apply, \
but come back with a `warning` and a `surface` list. File the declaration \
with `product_what_declare`, supplying a `verdict_knowledge` you author: one \
sentence on what the boundary lets the other side learn.";

#[cfg(test)]
mod tests {
    use super::*;

    fn repo(config: Option<&str>) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("tempdir");
        if let Some(c) = config {
            std::fs::create_dir_all(tmp.path().join(".ddd")).expect("mkdir");
            std::fs::write(tmp.path().join(".ddd/config.yaml"), c).expect("write");
        }
        tmp
    }

    #[test]
    fn an_ungoverned_repo_gets_no_governance_paragraph() {
        let tmp = repo(None);
        let text = instructions(tmp.path(), false);
        assert!(text.contains("Product framework graph"));
        assert!(!text.contains("GOVERNANCE"), "{text}");
    }

    #[test]
    fn interception_off_is_not_advertised_as_a_gate() {
        let tmp = repo(Some("format: 3\nintercept: off\nignore: []\n"));
        assert!(!instructions(tmp.path(), false).contains("GOVERNANCE"));
    }

    #[test]
    fn enforce_names_the_recovery_path_and_forbids_working_around_it() {
        let tmp = repo(Some("format: 3\nintercept: enforce\nignore: []\n"));
        let text = instructions(tmp.path(), false);
        assert!(text.contains("GOVERNANCE (intercept: enforce)"), "{text}");
        assert!(text.contains("product_what_declare"));
        assert!(text.contains("do NOT work around it"));
        assert!(text.contains("retry the identical edit"));
    }

    #[test]
    fn warn_says_the_write_still_lands() {
        let tmp = repo(Some("format: 3\nintercept: warn\nignore: []\n"));
        let text = instructions(tmp.path(), false);
        assert!(text.contains("GOVERNANCE (intercept: warn)"), "{text}");
        assert!(text.contains("still apply"));
    }

    #[test]
    fn the_per_class_override_decides_the_mode() {
        let tmp = repo(Some(
            "format: 3\nintercept: off\nignore: []\nintercept_by_class:\n  specification: enforce\n",
        ));
        assert!(instructions(tmp.path(), false).contains("intercept: enforce"));
    }

    #[test]
    fn the_phase_model_appears_only_for_a_gated_session() {
        let tmp = repo(None);
        assert!(!instructions(tmp.path(), false).contains("phase-gated"));
        assert!(instructions(tmp.path(), true).contains("phase-gated"));
    }
}
