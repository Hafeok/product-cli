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

/// The committed representative projection page — the HTML half of the
/// governed pair (`pair.units` in this repo's .ddd/config.yaml). The store
/// below exercises every class the render stylesheet defines, so the pair
/// contract check in `ddd report escapes` proves the vocabulary live in
/// both directions. Refresh after a deliberate renderer change with:
/// `UPDATE_FIXTURES=1 cargo test -p ddd-cli --test render`.
#[test]
fn render_sample_stays_in_step_with_the_renderer() {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd(tmp.path()).arg("init").assert().success();
    // One claim per status rung; the retired one also carries an expired
    // cadence so the violations table renders.
    for (id, status, revalidate) in [
        ("DDD-ex-1", "projected", "2027-01-01"),
        ("DDD-ex-2", "reported", "2027-01-01"),
        ("DDD-ex-3", "established", "2027-01-01"),
        ("DDD-ex-4", "retired", "2026-01-01"),
    ] {
        write(tmp.path(), &format!(".ddd/claims/{id}.yaml"), &format!(
            "format: 2\nid: {id}\nstatement: a worked-example claim at the {status} rung\nstatus: {status}\nevidence: exercised in the sample graph\nfalsifier: a counterexample\nowner: none\nchanged: 2026-08-01\nrevalidate_by: {revalidate}\n"));
    }
    // A decision with one holding pin and one that moved (ok + bad spans).
    write(tmp.path(), ".ddd/decisions/adopt.yaml",
        "format: 2\nid: dec/example/adopt\ntitle: Adopt the worked example\nrationale: the sample graph shows every render state once\nprincipal: Emil\nbased_on:\n  - claim: DDD-ex-3\n    status: established\n    changed: 2026-08-01\n  - claim: DDD-ex-2\n    status: projected\n    changed: 2026-07-01\n");
    // One governed rule plus one awaiting a decision (warn span).
    write(tmp.path(), ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/example/adopt\n  - rule_id: CA1848\n    severity: warning\n    governance: UNGOVERNED\n");
    // A seam declaration plus one correspondence row (card + table).
    write(tmp.path(), ".ddd/seams/seam-example-api.yaml",
        "format: 1\nid: seam/example/api\nboundary: method Ping in Api.cs\nverdict_knowledge: callers learn whether the service answers\ncontract_location: Api.cs#Ping\n");
    write(tmp.path(), ".ddd/seams/events/0001-ping.yaml",
        "format: 1\nid: seam-event/1\nat: 2026-08-10T12:00:00Z\nlanguage: csharp\nartifact_class: code\nmode: enforce\nfile: Api.cs\nsymbol: Ping\ncontainer: Api\nkind: method\nchange: added\nvisibility: public\nsignature: void Ping()\nreference_count: 2\nrule: cs-add-exposed\noutcome: applied-linked\nlinked_declaration: seam/example/api\n");
    ddd(tmp.path()).args(["render", "--today", "2026-08-10"]).assert().success();
    let rendered = std::fs::read_to_string(tmp.path().join(".ddd/render.html")).expect("page");

    let sample = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../docs/examples/ddd-render-sample.html");
    if std::env::var("UPDATE_FIXTURES").is_ok() {
        std::fs::write(&sample, &rendered).expect("update sample");
        return;
    }
    let committed = std::fs::read_to_string(&sample).expect(
        "docs/examples/ddd-render-sample.html missing — run UPDATE_FIXTURES=1 cargo test -p ddd-cli --test render",
    );
    assert_eq!(
        committed, rendered,
        "the committed render sample drifted from the renderer — regenerate with UPDATE_FIXTURES=1 and re-run ddd report escapes (the pair check reads this page)"
    );
}
