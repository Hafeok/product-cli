//! TC observability validation (FT-072, ADR-051).
//!
//! Pure module — no I/O, no println. Operationalises the
//! "TCs assert against named surfaces, not response envelopes" lesson by
//! exposing predicates for the graph-check layer:
//!
//! - `requires_observes(tc_type, phase, config)` — does this TC need
//!   `observes:` declared at all?
//! - `validate_surface(name, config)` — is this a recognised surface
//!   value (built-in or `[tc-observability].custom`)?
//! - `body_references_surface(body, surface)` — does the TC body mention
//!   the declared surface or a known synonym?
//!
//! The grammar is intentionally flat (a list of strings) per ADR-051's
//! "start cheap, promote later" decision.

use crate::config::TcObservabilityConfig;

/// Built-in surface vocabulary (ADR-051).
pub const BUILTIN_SURFACES: &[&str] = &[
    "file",
    "graph",
    "exit-code",
    "tag",
    "stdout",
    "stderr",
    "disk-state",
    "mcp-response",
];

/// True iff a TC of the given `tc_type` at the given `phase` must carry a
/// non-empty `observes:` list under `config`.
pub fn requires_observes(tc_type: &str, phase: u32, config: &TcObservabilityConfig) -> bool {
    if phase < config.required_from_phase {
        return false;
    }
    config.required_for_types.iter().any(|t| t == tc_type)
}

/// True iff `surface` is in the built-in vocabulary or
/// `[tc-observability].custom`.
pub fn is_known_surface(surface: &str, config: &TcObservabilityConfig) -> bool {
    BUILTIN_SURFACES.contains(&surface) || config.custom.iter().any(|s| s == surface)
}

/// Hint string listing every recognised surface — used in E026 / E032
/// diagnostics.
pub fn surface_hint(config: &TcObservabilityConfig) -> String {
    let mut all: Vec<String> = BUILTIN_SURFACES.iter().map(|s| (*s).to_string()).collect();
    for c in &config.custom {
        all.push(c.clone());
    }
    all.join(", ")
}

/// Synonyms recognised by the body-reference check. Intentionally short —
/// the goal is to nudge, not to police. Custom surfaces (those outside the
/// built-in vocabulary) fall back to literal name matching, handled by the
/// `body_references_surface` caller.
fn synonyms(surface: &str) -> &'static [&'static str] {
    match surface {
        "file" => &["file", "disk", "wrote", "on-disk", "write_file", "fs::"],
        "graph" => &["graph", "knowledge", "front-matter", "validates", "load_all"],
        "exit-code" => &["exit-code", "exit code", "exit(", "exit_code", "exits with"],
        "tag" => &["tag", "git tag", "product/ft-", "product/adr-"],
        "stdout" => &["stdout", "println", "println!", "print!", "output"],
        "stderr" => &["stderr", "eprintln", "eprintln!", "warning", "error"],
        "disk-state" => &["disk", "filesystem", "directory", "file tree"],
        "mcp-response" => &["mcp", "json-rpc", "envelope", "tool response"],
        _ => &[],
    }
}

/// True iff the TC body contains a case-insensitive match for the surface
/// name or one of its synonyms.
pub fn body_references_surface(body: &str, surface: &str) -> bool {
    let lower_body = body.to_lowercase();
    if lower_body.contains(&surface.to_lowercase()) {
        return true;
    }
    for syn in synonyms(surface) {
        if lower_body.contains(&syn.to_lowercase()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn required_for_scenario_at_phase_5_default_config() {
        let cfg = TcObservabilityConfig::default();
        assert!(requires_observes("scenario", 5, &cfg));
    }

    #[test]
    fn not_required_for_scenario_below_threshold() {
        let cfg = TcObservabilityConfig::default();
        assert!(!requires_observes("scenario", 4, &cfg));
    }

    #[test]
    fn not_required_for_invariant_type() {
        let cfg = TcObservabilityConfig::default();
        assert!(!requires_observes("invariant", 5, &cfg));
    }

    #[test]
    fn not_required_for_property_type() {
        let cfg = TcObservabilityConfig::default();
        assert!(!requires_observes("property", 5, &cfg));
    }

    #[test]
    fn builtin_surfaces_known() {
        let cfg = TcObservabilityConfig::default();
        for s in BUILTIN_SURFACES {
            assert!(is_known_surface(s, &cfg), "missing builtin: {}", s);
        }
    }

    #[test]
    fn custom_surface_known_when_in_config() {
        let mut cfg = TcObservabilityConfig::default();
        cfg.custom.push("my-custom".into());
        assert!(is_known_surface("my-custom", &cfg));
    }

    #[test]
    fn unknown_surface_rejected() {
        let cfg = TcObservabilityConfig::default();
        assert!(!is_known_surface("bogus_surface", &cfg));
    }

    #[test]
    fn body_reference_direct_match() {
        assert!(body_references_surface("The graph is queried", "graph"));
    }

    #[test]
    fn body_reference_synonym_match() {
        assert!(body_references_surface("The file is wrote to disk", "file"));
    }

    #[test]
    fn body_reference_case_insensitive() {
        assert!(body_references_surface("GRAPH is queried", "graph"));
    }

    #[test]
    fn body_reference_missing() {
        assert!(!body_references_surface(
            "completely unrelated text",
            "file"
        ));
    }
}
