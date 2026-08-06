//! Render projection: sections present, deterministic, self-contained.

use super::*;
use crate::detect::DetectedState;
use crate::report::report_escapes;
use crate::store::DddStore;

fn store_with_everything() -> DddStore {
    let mut s = DddStore::default();
    s.claims.push(
        serde_yaml::from_str(
            "format: 2\nid: DDD-a-1\nstatement: it <holds>\nstatus: reported\nevidence: e\nfalsifier: f\nowner: none\nchanged: 2026-08-01\nrevalidate_by: 2026-01-01\n",
        )
        .expect("claim"),
    );
    s.decisions.push(
        serde_yaml::from_str(
            "format: 2\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - claim: DDD-a-1\n    status: projected\n    changed: 2026-07-01\n",
        )
        .expect("decision"),
    );
    s.manifests.push(crate::manifest::ManifestSet {
        name: "analyzers".into(),
        file: serde_yaml::from_str(
            "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/x/y\n",
        )
        .expect("manifest"),
    });
    s.seams.push(
        serde_yaml::from_str(
            "format: 1\nid: seam/x/y\nboundary: b\nverdict_knowledge: ''\ncontract_location: f.cs\n",
        )
        .expect("seam"),
    );
    s
}

#[test]
fn renders_all_sections_and_flags_the_escapes() {
    let store = store_with_everything();
    let escapes = report_escapes(&store, &DetectedState::default(), "2026-08-06");
    let html = render_html(&store, &escapes, "2026-08-06");
    for marker in [
        "<h2>Claims (1)</h2>",
        "chip reported",
        "it &lt;holds&gt;",
        "<h2>Decisions (1)</h2>",
        "basis moved: pinned projected@2026-07-01",
        "<h2>Manifest coverage (1 rules)</h2>",
        "CA2007",
        "<h2>Escapes dashboard</h2>",
        "Cadence violations (1)",
        "Basis loss (1)",
        "Seam map (1 declarations, 0 interception rows)",
        "no verdict knowledge",
    ] {
        assert!(html.contains(marker), "missing `{marker}`");
    }
}

#[test]
fn regeneration_is_byte_identical_and_self_contained() {
    let store = store_with_everything();
    let escapes = report_escapes(&store, &DetectedState::default(), "2026-08-06");
    let a = render_html(&store, &escapes, "2026-08-06");
    let b = render_html(&store, &escapes, "2026-08-06");
    assert_eq!(a, b);
    for banned in ["http://", "https://", "<script", "src=", "@import"] {
        assert!(!a.contains(banned), "external reference `{banned}` in the page");
    }
}
