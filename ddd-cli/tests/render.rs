//! `ddd render` acceptance (M2.5): four sections from the current graph,
//! no external assets, regeneration byte-identical.

use std::path::Path;

use assert_cmd::Command;

fn ddd(dir: &Path) -> Command {
    let mut cmd = Command::cargo_bin("ddd").expect("binary");
    cmd.current_dir(dir);
    cmd
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

#[test]
fn renders_the_graph_to_one_offline_page_reproducibly() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    write(tmp.path(), ".ddd/claims/a.yaml",
        "format: 2\nid: DDD-a-1\nstatement: holds\nstatus: established\nevidence: e\nfalsifier: f\nowner: none\nchanged: 2026-08-01\nrevalidate_by: 2026-01-01\n");
    write(tmp.path(), ".ddd/decisions/d.yaml",
        "format: 2\nid: dec/x/y\ntitle: T\nrationale: R\nprincipal: Emil\nbased_on:\n  - claim: DDD-a-1\n    status: projected\n    changed: 2026-07-01\n");
    write(tmp.path(), ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/x/y\n");
    write(tmp.path(), ".ddd/seams/s.yaml",
        "format: 1\nid: seam/x/y\nboundary: b\nverdict_knowledge: what callers learn\ncontract_location: f.cs\n");

    ddd(tmp.path()).args(["render", "--today", "2026-08-06"]).assert().success();
    let out = tmp.path().join(".ddd/render.html");
    let first = std::fs::read_to_string(&out).expect("page");

    // All four sections, from the current graph, plus the seam map.
    for marker in [
        "<h2>Claims (1)</h2>",
        "chip established",
        "<h2>Decisions (1)</h2>",
        "basis moved",
        "<h2>Manifest coverage (1 rules)</h2>",
        "<h2>Escapes dashboard</h2>",
        "Cadence violations (1)",
        "Seam map (1 declarations, 0 interception rows)",
    ] {
        assert!(first.contains(marker), "missing `{marker}`");
    }
    // Offline: no external fetches, no scripts, no editable state.
    for banned in ["http://", "https://", "<script", "src=", "contenteditable"] {
        assert!(!first.contains(banned), "`{banned}` found in the page");
    }

    // Regenerating from the unchanged graph is byte-identical.
    ddd(tmp.path()).args(["render", "--today", "2026-08-06"]).assert().success();
    let second = std::fs::read_to_string(&out).expect("page");
    assert_eq!(first, second);
}
