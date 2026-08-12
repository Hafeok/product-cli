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

/// Run `git` in `root`, asserting success.
fn git(root: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

/// Commit the whole tree, so every governed file's parent state is
/// committed — the precondition for binding (M8 ruling 2).
fn commit_all(root: &Path) {
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "init"]);
}

/// A governed temp repo: `.ddd/` scaffold, format-3 config binding both
/// adapters to the mock host, the fixture sources on disk — all committed,
/// because enforce mode only binds against committed parent state (M8).
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
    git(tmp.path(), &["init", "-q"]);
    git(tmp.path(), &["config", "user.email", "t@example.com"]);
    git(tmp.path(), &["config", "user.name", "T"]);
    commit_all(tmp.path());
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
    // The template carries the pre-computed binding for this transition.
    let binding = demand["template"]["binding"].clone();
    assert_eq!(binding["symbol"], "Pong", "{demand}");
    assert_eq!(binding["file"], "Library.cs", "{demand}");
    assert!(binding["before"].is_string() && binding["after"].is_string(), "{demand}");
    assert!(binding["base_revision"].is_string(), "{demand}");
    assert!(std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Ping"));
    assert!(!std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));

    // 2. Declare the seam the demand names, signing the demanded transition.
    let declared = call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong",
        "boundary": "method Pong in Library.cs",
        "contract_location": "Library.cs#Pong",
        "symbol": "Pong",
        "verdict_knowledge": "callers learn whether the service answers",
        "binding": binding,
    }));
    assert_eq!(declared["status"], "filed", "{declared}");
    assert!(declared["warning"].is_null(), "{declared}");
    assert!(declared["binding"]["hash"].is_string(), "{declared}");

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
        "binding": demand["template"]["binding"],
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

/// Amending a filed declaration (dec/ddd/amend-explicit-evidence-frozen):
/// judgement fields revise, identity plus LSP-derived evidence do not.
#[test]
fn m4_amend_fills_in_the_warned_verdict_knowledge() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let cs = fixture("csharp/Library.cs");
    let edited = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n",
    );
    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");

    // File it warned: the boundary declares cost with no demand absorbed.
    // The binding still signs the demanded transition, so the re-apply
    // below discharges.
    let warned = call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "boundary": "method Pong in Library.cs",
        "contract_location": "Library.cs#Pong", "symbol": "Pong", "verdict_knowledge": "",
        "binding": rejected["demands"][0]["template"]["binding"],
    }));
    assert_eq!(warned["status"], "filed");
    assert!(warned["warning"].is_string(), "{warned}");

    // Re-declaring without the flag still refuses — no silent overwrite.
    let clash = registry.call_tool("ddd_declare_seam", &json!({
        "id": "seam/csharp/pong", "boundary": "b", "contract_location": "c",
    }));
    assert!(clash.expect_err("must refuse").contains("amend: true"));

    // Re-apply so the interceptor writes its LSP-derived evidence onto the
    // declaration; the amendment below must leave that evidence alone.
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");

    // Amending supplies only the field being revised.
    let amended = call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "amend": true,
        "verdict_knowledge": "callers learn whether the service answers",
    }));
    assert_eq!(amended["status"], "amended", "{amended}");
    assert!(amended["warning"].is_null(), "{amended}");

    let seam = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-csharp-pong.yaml"))
        .expect("seam");
    assert!(seam.contains("callers learn whether the service answers"), "{seam}");
    // Omitted judgement fields are preserved, not blanked.
    assert!(seam.contains("boundary: method Pong in Library.cs"), "{seam}");
    // The interceptor's evidence survives the amendment.
    assert!(seam.contains("kind: method"), "{seam}");
    assert!(seam.contains("reference_count:"), "{seam}");

    let why = call(&registry, "ddd_why", json!({"id": "seam/csharp/pong"}));
    assert!(why["chain"].as_str().expect("chain").contains("callers learn"), "{why}");
}

#[test]
fn m4_amend_cannot_create_and_cannot_rewrite_frozen_evidence() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    // Amending something never filed is an error, not a create.
    let missing = registry.call_tool("ddd_declare_seam", &json!({
        "id": "seam/csharp/ghost", "amend": true, "verdict_knowledge": "x",
    }));
    assert!(missing.expect_err("must refuse").contains("nothing is filed"));
    assert!(!tmp.path().join(".ddd/seams/seam-csharp-ghost.yaml").exists());

    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "boundary": "method Pong", "symbol": "Pong",
        "contract_location": "Library.cs#Pong", "verdict_knowledge": "v",
    }));
    // contract_location is evidence: a supplied value is ignored, not applied.
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "amend": true, "contract_location": "Elsewhere.cs#Fake",
    }));
    let seam = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-csharp-pong.yaml"))
        .expect("seam");
    assert!(seam.contains("Library.cs#Pong"), "{seam}");
    assert!(!seam.contains("Elsewhere.cs#Fake"), "{seam}");
}

#[test]
fn m4_amend_refines_one_obligation_without_restating_the_rest() {
    let (tmp, registry) = repo("intercept: enforce\n");
    write(tmp.path(), ".ddd/predicates/composition-decorator.yaml",
        &fixture("../../../.ddd/predicates/composition-decorator.yaml"));
    let args = json!({
        "id": "pat/decorator/logging", "pattern": "pred/composition/decorator",
        "instance": "Scrutor .Decorate in Program.cs",
        "obligation_answers": {
            "ordering": "logging outermost",
            "identity-preservation": "decorated interface only",
            "forwarding-completeness": "all members forwarded",
        },
    });
    call(&registry, "ddd_declare_pattern", args.clone());

    // One obligation revised; the other two carry forward and completeness
    // still holds.
    let amended = call(&registry, "ddd_declare_pattern", json!({
        "id": "pat/decorator/logging", "amend": true,
        "obligation_answers": {"ordering": "logging outermost, retry innermost"},
    }));
    assert_eq!(amended["status"], "amended", "{amended}");
    let q = call(&registry, "ddd_graph_query", json!({"select": "patterns"}));
    let obligations = &q["entries"][0]["obligations"];
    assert_eq!(obligations["ordering"], "logging outermost, retry innermost");
    assert_eq!(obligations["identity-preservation"], "decorated interface only");
    assert_eq!(obligations["forwarding-completeness"], "all members forwarded");

    // The predicate is identity: a filed instance cannot be re-pointed.
    let repointed = registry.call_tool("ddd_declare_pattern", &json!({
        "id": "pat/decorator/logging", "amend": true, "pattern": "pred/other/thing",
    }));
    assert!(repointed.expect_err("must refuse").contains("cannot change its predicate"));
}

#[test]
fn m4_accept_risk_amends_its_rationale() {
    let (tmp, registry) = repo("intercept: enforce\n");
    call(&registry, "ddd_accept_risk", json!({
        "diagnostic_id": "CA1848", "rationale": "first pass", "principal": "Emil",
    }));
    let amended = call(&registry, "ddd_accept_risk", json!({
        "diagnostic_id": "CA1848", "amend": true,
        "rationale": "the service logs under 200 lines/min; the codegen is not worth it",
    }));
    assert_eq!(amended["status"], "amended", "{amended}");
    let record = std::fs::read_to_string(tmp.path().join(".ddd/decisions/risk-rule-ca1848.yaml"))
        .expect("record");
    assert!(record.contains("200 lines/min"), "{record}");
    // principal was not re-supplied, so it is preserved.
    assert!(record.contains("principal: Emil"), "{record}");
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
    git(tmp.path(), &["init", "-q"]);
    git(tmp.path(), &["config", "user.email", "t@example.com"]);
    git(tmp.path(), &["config", "user.name", "T"]);
    commit_all(tmp.path());
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
        "binding": rejected["demands"][0]["template"]["binding"],
    }));
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"][0], "seam/csharp/pong");
    assert_eq!(event_rows(tmp.path()).len(), 2);
}

/// `dec/ddd/enforce-matching-tightens-to-symbol`, the regression M6 observed
/// as seam-event/4: one edit adds two public symbols, two declarations are
/// authored one per symbol, and each surface event must link to the
/// declaration naming its own symbol — never to whichever declaration
/// happened to share the file. The metadata write must stay symbol-exact
/// too, or the correspondence record silently carries another symbol's
/// facts (DDD-arch-05).
#[test]
fn m7_enforce_links_each_symbol_to_its_own_declaration() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let cs = fixture("csharp/Library.cs");
    let edited = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Alpha() { }\n    public void Beta(int n) { }\n",
    );
    // One rejection raises one demand per symbol, each carrying its own
    // pre-computed binding.
    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    let demands = rejected["demands"].as_array().expect("demands").clone();
    let binding_for = |symbol: &str| {
        demands
            .iter()
            .find(|d| d["surface"]["symbol"] == symbol)
            .unwrap_or_else(|| panic!("no demand for {symbol}: {demands:?}"))["template"]["binding"]
            .clone()
    };
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/alpha", "boundary": "method Alpha in Library.cs",
        "contract_location": "Library.cs#Alpha", "symbol": "Alpha",
        "verdict_knowledge": "callers learn the alpha outcome",
        "binding": binding_for("Alpha"),
    }));
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/beta", "boundary": "method Beta in Library.cs",
        "contract_location": "Library.cs#Beta", "symbol": "Beta",
        "verdict_knowledge": "callers learn the beta count",
        "binding": binding_for("Beta"),
    }));
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(applied["status"], "applied", "{applied}");

    // Every applied-linked row links the declaration naming its own symbol.
    let mut linked_symbols = Vec::new();
    for row in event_rows(tmp.path()) {
        if row["outcome"] != "applied-linked" {
            continue;
        }
        let (symbol, linked) = (row["symbol"].as_str().expect("symbol"),
                                row["linked_declaration"].as_str().expect("linked"));
        match symbol {
            "Alpha" => assert_eq!(linked, "seam/csharp/alpha", "{row}"),
            "Beta" => assert_eq!(linked, "seam/csharp/beta", "{row}"),
            other => panic!("unexpected row symbol {other}"),
        }
        linked_symbols.push(symbol.to_string());
    }
    assert!(linked_symbols.contains(&"Alpha".to_string()), "{linked_symbols:?}");
    assert!(linked_symbols.contains(&"Beta".to_string()), "{linked_symbols:?}");

    // Each declaration carries its own symbol's facts — the mis-attribution
    // seam-event/4 produced can no longer occur.
    let alpha = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-csharp-alpha.yaml")).expect("alpha");
    assert!(alpha.contains("symbol: Alpha"), "{alpha}");
    let beta = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-csharp-beta.yaml")).expect("beta");
    assert!(beta.contains("symbol: Beta"), "{beta}");
    assert!(beta.contains("signature: void Beta(int n)"), "{beta}");
}

/// The file arm is gone from enforce mode: a stored declaration whose
/// contract_location names the file but whose symbol is a different one
/// admits nothing — and under M8 a declaration with no signed binding
/// discharges nothing at all, however its location matches.
#[test]
fn m7_enforce_rejects_a_file_arm_only_declaration() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/ping", "boundary": "method Ping in Library.cs",
        "contract_location": "Library.cs",
        "symbol": "Ping",
        "verdict_knowledge": "an unrelated symbol in the same file",
    }));
    let cs = fixture("csharp/Library.cs");
    let edited = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n",
    );
    let out = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(out["status"], "rejected", "{out}");
    assert_eq!(out["demands"][0]["surface"]["symbol"], "Pong", "{out}");
    let rows = event_rows(tmp.path());
    assert_eq!(rows.last().expect("row")["outcome"], "rejected");
}

/// Warn mode keeps the broad arm: the row gets the generous link, but the
/// declaration's metadata is never overwritten by a file-arm match.
#[test]
fn m7_warn_keeps_the_file_arm_and_never_writes_metadata() {
    let (tmp, registry) = repo("intercept: warn\n");
    warm(&registry);
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/bicep/storage", "boundary": "the storage module contract",
        "contract_location": "storage-module.bicep",
        "verdict_knowledge": "deployers learn the module's parameter shape",
    }));
    let bi = fixture("bicep/storage-module.bicep");
    let edited = bi.replace(
        "param accountName string\n",
        "param accountName string\nparam replicas int = 2\n",
    );
    let out = call(&registry, "ddd_apply_edit",
        json!({"file": "storage-module.bicep", "new_text": edited}));
    assert_eq!(out["status"], "applied", "{out}");
    assert_eq!(out["mode"], "warn");
    let rows = event_rows(tmp.path());
    let row = rows.last().expect("row");
    assert_eq!(row["outcome"], "applied-warned");
    assert_eq!(row["linked_declaration"], "seam/bicep/storage", "{row}");
    let seam = std::fs::read_to_string(tmp.path().join(".ddd/seams/seam-bicep-storage.yaml")).expect("seam");
    assert!(!seam.contains("symbol: replicas"), "file-arm match wrote metadata: {seam}");
}

/// M8 ruling 2: enforce mode never binds uncommitted parent state. A
/// governed file that is dirty against HEAD rejects the surface edit
/// outright, and a binding attempt against the dirty file refuses by name.
#[test]
fn m8_dirty_parent_state_refuses_to_bind() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    // Dirty the governed file after the initial commit: disk != HEAD.
    let cs = fixture("csharp/Library.cs");
    let dirty = format!("// uncommitted local drift\n{cs}");
    std::fs::write(tmp.path().join("Library.cs"), &dirty).expect("dirty write");

    let edited = dirty.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n",
    );
    let out = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": edited}));
    assert_eq!(out["status"], "rejected", "{out}");
    let reason = out["reason"].as_str().expect("reason");
    assert!(
        reason.contains("never binds uncommitted parent state") || reason.contains("M8 ruling 2"),
        "{reason}"
    );
    assert!(!std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));

    // Declaring a binding whose file is dirty refuses, naming the constraint.
    let err = registry
        .call_tool("ddd_declare_seam", &json!({
            "id": "seam/csharp/pong", "boundary": "method Pong in Library.cs",
            "contract_location": "Library.cs#Pong", "symbol": "Pong",
            "verdict_knowledge": "callers learn whether the service answers",
            "binding": {
                "symbol": "Pong", "file": "Library.cs",
                "before": "sha256:0000000000000000000000000000000000000000000000000000000000000000",
                "after": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
            },
        }))
        .expect_err("a dirty file must refuse to bind");
    assert!(err.contains("dirty against HEAD"), "{err}");
}

/// A signed binding discharges exactly the transition it names (M8, spec
/// invariant 3): the same symbol arriving via a different edit — different
/// proposed content — stays rejected; only the signed edit applies.
#[test]
fn m8_a_binding_does_not_discharge_a_different_transition() {
    let (tmp, registry) = repo("intercept: enforce\n");
    warm(&registry);
    let cs = fixture("csharp/Library.cs");
    let e1 = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { }\n",
    );
    let e2 = cs.replace(
        "    public void Ping() { }\n",
        "    public void Ping() { }\n    public void Pong() { System.Console.WriteLine(\"pong\"); }\n",
    );

    let rejected = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": e1}));
    assert_eq!(rejected["status"], "rejected", "{rejected}");
    call(&registry, "ddd_declare_seam", json!({
        "id": "seam/csharp/pong", "boundary": "method Pong in Library.cs",
        "contract_location": "Library.cs#Pong", "symbol": "Pong",
        "verdict_knowledge": "callers learn whether the service answers",
        "binding": rejected["demands"][0]["template"]["binding"],
    }));

    // E2 adds the same symbol but lands different content — not the
    // transition the binding signs.
    let still = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": e2}));
    assert_eq!(still["status"], "rejected", "{still}");
    assert!(!std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));

    // E1 exactly is what was signed.
    let applied = call(&registry, "ddd_apply_edit", json!({"file": "Library.cs", "new_text": e1}));
    assert_eq!(applied["status"], "applied", "{applied}");
    assert_eq!(applied["linked"][0], "seam/csharp/pong");
    assert!(std::fs::read_to_string(tmp.path().join("Library.cs")).expect("read").contains("Pong"));
}
