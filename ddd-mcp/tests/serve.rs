//! M3+M4 acceptance over the ddd_* registry with mock LSP hosts: language
//! tools on the fixtures, the manifest join, and the full interceptor
//! loop (reject → declare → re-apply → correspondence rows).

use std::path::{Path, PathBuf};

use serde_json::{json, Value};

fn fixture(rel: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../ddd-cli/tests/fixtures")
        .join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn write(root: &Path, rel: &str, content: &str) {
    let path = root.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("mkdir");
    }
    std::fs::write(path, content).expect("write");
}

/// A governed temp repo: `.ddd/` scaffold, format-3 config binding both
/// adapters to the mock host, the fixture sources on disk.
fn repo(config_tail: &str) -> (tempfile::TempDir, product_mcp::ToolRegistry) {
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd_core::init::apply_init(&ddd_core::init::plan_init(tmp.path())).expect("init");
    let mock = env!("CARGO_BIN_EXE_ddd-mcp-mock-lsp");
    write(
        tmp.path(),
        ".ddd/config.yaml",
        &format!(
            "format: 3\nignore: []\nadapter:\n  csharp:\n    command: [\"{mock}\", \"--handshake\", \"roslyn\"]\n  bicep:\n    command: [\"{mock}\"]\n{config_tail}"
        ),
    );
    write(tmp.path(), "Library.cs", &fixture("csharp/Library.cs"));
    write(tmp.path(), "Api.csproj", "<Project Sdk=\"Microsoft.NET.Sdk\" />");
    write(tmp.path(), "storage-module.bicep", &fixture("bicep/storage-module.bicep"));
    let (_state, registry) = ddd_mcp::serve::build_registry(tmp.path().to_path_buf());
    (tmp, registry)
}

fn call(registry: &product_mcp::ToolRegistry, name: &str, args: Value) -> Value {
    registry.call_tool(name, &args).unwrap_or_else(|e| panic!("{name}: {e}"))
}

fn warm(registry: &product_mcp::ToolRegistry) {
    let out = call(registry, "ddd_warmup", json!({"wait_ms": 10_000}));
    let hosts = out["hosts"].as_array().expect("hosts");
    for h in hosts {
        assert_eq!(h["readiness"]["state"], "ready", "{h}");
    }
}

/// Zero-based (line, character) of the first occurrence of `needle`.
fn pos_of(text: &str, needle: &str) -> (usize, usize) {
    for (i, line) in text.lines().enumerate() {
        if let Some(col) = line.find(needle) {
            return (i, col);
        }
    }
    panic!("`{needle}` not in text");
}

fn event_rows(root: &Path) -> Vec<Value> {
    let dir = root.join(".ddd/seams/events");
    let mut rows = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        let mut paths: Vec<_> = rd.flatten().map(|e| e.path()).collect();
        paths.sort();
        for p in paths {
            let text = std::fs::read_to_string(&p).expect("row");
            rows.push(serde_yaml::from_str::<Value>(&text).expect("yaml"));
        }
    }
    rows
}

const CLAIM: &str = "format: 1\nid: DDD-t-1\nstatement: the rule pays rent\nstatus: reported\nevidence: exercised\nfalsifier: a counterexample\nowner: none\nchanged: 2026-08-01\n";
const DECISION: &str = "format: 1\nid: dec/cs/async-config\ntitle: Adopt CA2007\nrationale: sync contexts bite\nprincipal: Emil\nbased_on: [DDD-t-1]\n";

#[test]
fn m3_language_tools_answer_on_both_fixtures() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let cs = fixture("csharp/Library.cs");

    let found = call(&registry, "ddd_find_symbol", json!({"file": "Library.cs"}));
    let names: Vec<&str> =
        found["symbols"].as_array().expect("symbols").iter().filter_map(|s| s["name"].as_str()).collect();
    for expect in ["PublicApi", "Ping", "IContract", "Inner", "Fetch"] {
        assert!(names.contains(&expect), "{expect} missing from {names:?}");
    }

    let (line, ch) = pos_of(&cs, "Ping");
    let refs = call(&registry, "ddd_references", json!({"file": "Library.cs", "line": line, "character": ch}));
    assert!(refs["count"].as_u64().expect("count") >= 1, "{refs}");

    let hover = call(&registry, "ddd_hover", json!({"file": "Library.cs", "line": line, "character": ch}));
    assert!(hover["contents"].as_str().expect("contents").contains("Ping"), "{hover}");

    let sig = call(&registry, "ddd_signature", json!({"file": "Library.cs", "line": line, "character": ch}));
    assert!(!sig["signatures"].as_array().expect("sigs").is_empty(), "{sig}");

    let ren = call(&registry, "ddd_rename",
        json!({"file": "Library.cs", "line": line, "character": ch, "new_name": "Pulse"}));
    assert_eq!(ren["status"], "computed");
    assert_eq!(ren["applied"], false);
    assert!(!ren["workspace_edit"].as_array().expect("we").is_empty());

    let bi = fixture("bicep/storage-module.bicep");
    let bi_found = call(&registry, "ddd_find_symbol", json!({"file": "storage-module.bicep"}));
    let bi_names: Vec<&str> =
        bi_found["symbols"].as_array().expect("symbols").iter().filter_map(|s| s["name"].as_str()).collect();
    for expect in ["location", "accountName", "stg", "storageId"] {
        assert!(bi_names.contains(&expect), "{expect} missing from {bi_names:?}");
    }
    let (bl, bc) = pos_of(&bi, "accountName");
    let bi_refs = call(&registry, "ddd_references",
        json!({"file": "storage-module.bicep", "line": bl, "character": bc}));
    assert!(bi_refs["count"].as_u64().expect("count") >= 2, "{bi_refs}");
    drop(tmp);
}

#[test]
fn m3_diagnostics_join_the_manifest_by_rule_id() {
    let (tmp, registry) = repo("intercept: enforce\n");
    write(tmp.path(), ".ddd/manifest/analyzers.yaml",
        "format: 1\nrules:\n  - rule_id: CA2007\n    severity: warning\n    decision: dec/cs/async-config\n");
    write(tmp.path(), ".ddd/claims/ddd-t-1.yaml", CLAIM);
    write(tmp.path(), ".ddd/decisions/dec-cs-async-config.yaml", DECISION);
    write(tmp.path(), "Library.cs.diag.json",
        r#"[{"range": {"start": {"line": 4, "character": 0}, "end": {"line": 4, "character": 10}},
             "severity": 2, "code": "CA2007", "source": "csharp", "message": "consider ConfigureAwait"},
            {"range": {"start": {"line": 5, "character": 0}, "end": {"line": 5, "character": 10}},
             "severity": 3, "code": "CA1848", "source": "csharp", "message": "use LoggerMessage"}]"#);
    warm(&registry);
    let out = call(&registry, "ddd_diagnostics", json!({"file": "Library.cs"}));
    let diags = out["diagnostics"].as_array().expect("diagnostics");
    assert_eq!(diags.len(), 2, "{out}");
    let ca2007 = diags.iter().find(|d| d["rule_id"] == "CA2007").expect("CA2007");
    assert_eq!(ca2007["namespace"], "analyzers");
    assert_eq!(ca2007["governance"]["decision"], "dec/cs/async-config");
    let ca1848 = diags.iter().find(|d| d["rule_id"] == "CA1848").expect("CA1848");
    assert!(ca1848["governance"]["marker"].as_str().expect("marker").contains("no manifest entry"));
}

#[test]
fn m4_full_loop_csharp_reject_declare_reapply() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let cs = fixture("csharp/Library.cs");
    let edited = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n",
    );

    // 1. The edit touches contract surface → rejected with the demand.
    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let demand = &rejected["demands"][0];
    assert_eq!(demand["surface"]["symbol"], "Pong");
    assert_eq!(demand["surface"]["kind"], "method");
    assert_eq!(demand["surface"]["visibility"], "public");
    assert_eq!(demand["surface"]["rule"], "cs-add-exposed");
    assert!(demand["surface"]["reference_count"].is_u64(), "{demand}");
    assert_eq!(demand["template"]["verdict_knowledge"], "");
    assert!(std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Ping"));
    assert!(!std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));

    // 2. Declare the seam the demand names.
    let declared = call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong",
        "boundary": "method Pong in Library.cs",
        "contract_location": "Library.cs#Pong",
        "symbol": "Pong",
        "verdict_knowledge": "callers learn whether the service answers",
    }));
    assert_eq!(declared["status"], "filed", "{declared}");
    assert!(declared["warning"].is_null(), "{declared}");

    // 3. Re-apply → applied, linked to the declaration.
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"][0], "seam/csharp/pong");
    assert!(std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));

    // 4. The correspondence rows: rejected then applied-linked, with the
    // structural metadata.
    let rows = event_rows(tmp.path());
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0]["outcome"], "rejected");
    assert_eq!(rows[1]["outcome"], "applied-linked");
    assert_eq!(rows[1]["symbol"], "Pong");
    assert_eq!(rows[1]["kind"], "method");
    assert_eq!(rows[1]["linked_declaration"], "seam/csharp/pong");
    assert!(rows[1]["reference_count"].is_u64(), "{:?}", rows[1]);

    // 5. The seam declaration carries the LSP-derived metadata now.
    let seam = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-csharp-pong.yaml")).expect("seam");
    assert!(seam.contains("kind: method"), "{seam}");
    assert!(seam.contains("reference_count:"), "{seam}");

    // 6. why resolves the seam id end-to-end.
    let why = call(&registry, "ddd_why", json!({"id": "seam/csharp/pong"}));
    assert_eq!(why["status"], "ok");
    assert!(why["chain"].as_str().expect("chain").contains("callers learn"), "{why}");
}

#[test]
fn m4_full_loop_bicep_param_change() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let bi = fixture("bicep/storage-module.bicep");
    let edited = bi.replace(
        "param accountName string\n",
        "param accountName string\nparam replicas int = 2\n",
    );
    let rejected = call(&registry, "ddd_apply_edit",
        json!({"file": "storage-module.bicep", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let demand = &rejected["demands"][0];
    assert_eq!(demand["surface"]["symbol"], "replicas");
    assert_eq!(demand["surface"]["kind"], "param");
    assert_eq!(demand["surface"]["rule"], "bi-param-membership");
    assert_eq!(demand["surface"]["extra"]["ground_provenance"], "controlled");

    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/bicep/storage-replicas",
        "boundary": "storage module param replicas",
        "contract_location": "storage-module.bicep",
        "symbol": "replicas",
        "verdict_knowledge": "deployers choose redundancy; the module encodes the allowed range",
    }));
    let applied = call(&registry, "ddd_apply_edit",
        json!({"file": "storage-module.bicep", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    let rows = event_rows(tmp.path());
    assert_eq!(rows.last().expect("row")["extra"]["ground_provenance"], "controlled");
}

#[test]
fn m4_warn_and_off_modes_per_artifact_class() {
    // code (C#) stays enforce; configuration (Bicep) warns.
    let (tmp, registry) =
        repo("intercept: enforce\nintercept_by_class:\n  configuration: warn\n");
    warm(&registry);
    let bi = fixture("bicep/storage-module.bicep");
    let edited = bi.replace("param accountName string\n", "param accountName string\nparam extra int = 1\n");
    let out = call(&registry, "ddd_apply_edit",
        json!({"file": "storage-module.bicep", "new_text": edited}));
    assert_eq!(out["status"], "applied", "{out}");
    assert_eq!(out["mode"], "warn");
    assert!(out["warning"].as_str().expect("warning").contains("declare the seam"));
    let rows = event_rows(tmp.path());
    assert_eq!(rows.last().expect("row")["outcome"], "applied-warned");

    // C# under the same config still enforces.
    let cs = fixture("csharp/Library.cs");
    let cs_edit = cs.replace("    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n");
    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": cs_edit}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");

    // off skips classification entirely: no host consulted, no rows.
    let (tmp2, registry2) = repo("intercept: off\n");
    let out2 = call(&registry2, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": cs_edit}));
    assert_eq!(out2["status"], "applied");
    assert_eq!(out2["intercepted"], false);
    assert!(event_rows(tmp2.path()).is_empty());
}

#[test]
fn m4_declare_pattern_round_trips_the_decorator_obligations() {
    let (tmp, registry) = repo("intercept: enforce\n");
    write(tmp.path(), ".ddd/predicates/composition-decorator.yaml",
        &fixture("../../../.ddd/predicates/composition-decorator.yaml"));
    let missing = registry.call_tool("ddd_declare_pattern", &json!({
        "id": "pat/decorator/logging",
        "pattern": "pred/composition/decorator",
        "instance": "Scrutor .Decorate in Program.cs",
        "obligation_answers": {"ordering": "logging outermost"},
    }));
    let err = missing.expect_err("must reject unanswered obligations");
    assert!(err.contains("identity-preservation"), "{err}");
    assert!(err.contains("forwarding-completeness"), "{err}");

    let filed = call(&registry, "ddd_declare_pattern", json!({
        "id": "pat/decorator/logging",
        "pattern": "pred/composition/decorator",
        "instance": "Scrutor .Decorate in Program.cs",
        "obligation_answers": {
            "ordering": "logging outermost, retry innermost",
            "identity-preservation": "decorators re-expose the decorated interface only",
            "forwarding-completeness": "all members forwarded; verified by the interface test",
        },
    }));
    assert_eq!(filed["status"], "filed", "{filed}");
    let q = call(&registry, "ddd_graph_query", json!({"select": "patterns"}));
    assert_eq!(q["count"], 1, "{q}");
    let why = call(&registry, "ddd_why", json!({"id": "pat/decorator/logging"}));
    assert!(why["chain"].as_str().expect("chain").contains("forwarding"), "{why}");
}

#[test]
fn m4_accept_risk_satisfies_the_uncited_suppression_check() {
    let (tmp, registry) = repo("intercept: enforce\n");
    write(tmp.path(), "bicepconfig.json", &fixture("bicep/bicepconfig.json"));

    let filed = call(&registry, "ddd_accept_risk", json!({
        "diagnostic_id": "no-hardcoded-env-urls",
        "rationale": "the only URLs are Azure public-cloud endpoints pinned on purpose",
        "principal": "Emil (Context&)",
    }));
    assert_eq!(filed["status"], "filed", "{filed}");
    let risk_id = filed["id"].as_str().expect("id");

    write(tmp.path(), ".ddd/manifest/linter.yaml", &format!(
        "format: 1\nrules:\n  - rule_id: no-unused-params\n    severity: warning\n    governance: UNGOVERNED\n  - rule_id: no-hardcoded-env-urls\n    severity: \"off\"\n    governance: UNGOVERNED\n    suppression:\n      risk_acceptance: {risk_id}\n"
    ));
    let store = ddd_core::store::load(&tmp.path().join(".ddd"));
    let detected = ddd_core::detect::detect(tmp.path(), store.config.as_ref(), &[]);
    let report = ddd_core::diff::diff(&store, &detected);
    let uncited: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.kind == ddd_core::diff::FindingKind::UncitedSuppression)
        .collect();
    assert!(uncited.is_empty(), "{uncited:?}");
}

/// The full interceptor loop against the *real* Roslyn host — gated like
/// the other real-host tests (`DDD_LSP_E2E=1`, tools on PATH).
#[test]
fn m4_real_roslyn_full_loop() {
    if std::env::var("DDD_LSP_E2E").ok().as_deref() != Some("1") {
        return;
    }
    let tmp = tempfile::tempdir().expect("tempdir");
    ddd_core::init::apply_init(&ddd_core::init::plan_init(tmp.path())).expect("init");
    write(tmp.path(), ".ddd/config.yaml", "format: 3\nintercept: enforce\nignore: []\n");
    write(tmp.path(), "Library.cs", &fixture("csharp/Library.cs"));
    write(tmp.path(), "Api.csproj",
        "<Project Sdk=\"Microsoft.NET.Sdk\"><PropertyGroup><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>");
    let (_state, registry) = ddd_mcp::serve::build_registry(tmp.path().to_path_buf());
    let out = call(&registry, "ddd_warmup", json!({"wait_ms": 240_000}));
    assert_eq!(out["hosts"][0]["readiness"]["state"], "ready", "{out}");
    let cs = fixture("csharp/Library.cs");
    let edited = cs.replace("    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n");
    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    assert_eq!(rejected["demands"][0]["surface"]["symbol"], "Pong", "{rejected}");
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "boundary": "method Pong in Library.cs",
        "contract_location": "Library.cs#Pong", "symbol": "Pong",
        "verdict_knowledge": "callers learn whether the service answers",
    }));
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"][0], "seam/csharp/pong");
    assert_eq!(event_rows(tmp.path()).len(), 2);
}
