//! Integration tests — author.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn tc_149_author_session_preflight_first() {
    let h = harness_with_domains();

    // Cross-cutting ADR
    h.write("docs/adrs/ADR-013-error-model.md",
        "---\nid: ADR-013\ntitle: Error Model\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [error-handling]\nscope: cross-cutting\n---\n\nError model.\n");

    // Feature with gaps
    h.write("docs/features/FT-009-rate-limiting.md",
        "---\nid: FT-009\ntitle: Rate Limiting\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\ndomains: []\ndomains-acknowledged: {}\n---\n\nRate limiting.\n");

    // Run author feature with --feature flag — should be blocked by preflight
    let out = h.run(&["author", "feature", "--feature", "FT-009"]);
    assert!(
        out.exit_code != 0,
        "author session should be blocked by preflight gaps, got exit {}",
        out.exit_code
    );
    assert!(
        out.stderr.contains("preflight") || out.stderr.contains("Pre-flight") || out.stderr.contains("ADR-013"),
        "Should show preflight report before session starts, got stderr:\n{}",
        out.stderr
    );
}

