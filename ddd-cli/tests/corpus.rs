//! Hand-labelled classifier corpus harness (ddd research protocol §3).
//!
//! Walks `tests/corpus/<language>/<case>/`, replays each case's
//! before→after transition as a commit pair in one temp git repo per
//! language, classifies the pair through `diff_contracts` — the same M8
//! repository-diff path CI runs — then measures the classifier against the
//! case's hand-written labels:
//!
//! - **recall** — every labelled `surface` row must appear as a surface
//!   event (spec success criterion 1);
//! - **false-demand rate** — no labelled `non_surface` row may appear as a
//!   surface event (spec success criterion 2, the friction number).
//!
//! The corpus doubles as the acceptance harness for policy-table
//! amendments: a row change that trades recall for friction fails here by
//! case name. Reading: `docs/audits/classifier-corpus-2026-08.md`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use ddd_core::surface::ChangeKind;
use ddd_lsp::revdiff::diff_contracts;
use ddd_lsp::HostManager;
use serde::Deserialize;

// ------------------------------------------------------------- the labels

/// One case's hand-written ground truth (`labels.yaml`).
#[derive(Deserialize)]
struct Labels {
    case: String,
    #[serde(default)]
    surface: Vec<Label>,
    #[serde(default)]
    non_surface: Vec<Label>,
}

/// One labelled change: symbol name plus the change kind a careful human
/// reading of the diff assigns it.
#[derive(Deserialize)]
struct Label {
    symbol: String,
    /// A `ChangeKind` in kebab-case, or `any` — used where the correct
    /// classifier behaviour is *no event at all* (e.g. a body-only edit),
    /// so any surface event on the symbol is a false demand.
    change: String,
    /// Optional disambiguation when two changed symbols share a name.
    #[serde(default)]
    container: Option<String>,
}

impl Label {
    fn matches(&self, ev: &Observed) -> bool {
        ev.symbol == self.symbol
            && (self.change == "any" || self.change == ev.change)
            && self.container.as_deref().map(|c| c == ev.container).unwrap_or(true)
    }
}

/// One classified event, flattened for label matching.
struct Observed {
    symbol: String,
    container: String,
    change: &'static str,
    surface: bool,
    rule: String,
}

fn change_str(c: ChangeKind) -> &'static str {
    match c {
        ChangeKind::Added => "added",
        ChangeKind::Removed => "removed",
        ChangeKind::SignatureChanged => "signature-changed",
        ChangeKind::DecoratorChanged => "decorator-changed",
        ChangeKind::VisibilityChanged => "visibility-changed",
    }
}

// ------------------------------------------------------------ git plumbing

fn git(dir: &Path, args: &[&str]) {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?}: {e}"));
    assert!(out.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&out.stderr));
}

fn git_head(dir: &Path) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap_or_else(|e| panic!("git rev-parse: {e}"));
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

fn commit_all(dir: &Path, msg: &str) {
    git(dir, &["add", "-A"]);
    git(dir, &["commit", "-q", "--allow-empty", "-m", msg]);
}

fn init_repo(dir: &Path) {
    git(dir, &["init", "-q"]);
    git(dir, &["config", "user.email", "corpus@example.com"]);
    git(dir, &["config", "user.name", "Corpus"]);
}

// -------------------------------------------------------------- the corpus

fn corpus_dir(language: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/corpus").join(language)
}

fn case_dirs(language: &str) -> Vec<PathBuf> {
    let root = corpus_dir(language);
    let rd = std::fs::read_dir(&root).unwrap_or_else(|e| panic!("{}: {e}", root.display()));
    let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    dirs.sort();
    assert!(!dirs.is_empty(), "no cases under {}", root.display());
    dirs
}

/// The `before.*`/`after.*` pair of one case, with the repo-relative file
/// the transition targets (fixed per extension, so the classifier sees a
/// stable path across cases).
fn case_files(case: &Path) -> (String, String, &'static str) {
    for ext in ["rs", "css", "html"] {
        let before = case.join(format!("before.{ext}"));
        if before.exists() {
            let after = case.join(format!("after.{ext}"));
            let target = match ext {
                "rs" => "src/lib.rs",
                "css" => "page.css",
                _ => "page.html",
            };
            return (read(&before), read(&after), target);
        }
    }
    panic!("{}: no before.rs/.css/.html", case.display());
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn load_labels(case: &Path) -> Labels {
    let path = case.join("labels.yaml");
    let labels: Labels =
        serde_yaml::from_str(&read(&path)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let dir_name = case.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    assert_eq!(labels.case, dir_name, "labels.yaml `case` must match its directory");
    assert!(
        !labels.surface.is_empty() || !labels.non_surface.is_empty(),
        "{dir_name}: a case must label at least one change"
    );
    labels
}

// ------------------------------------------------------------- the harness

struct CaseOutcome {
    name: String,
    surface_total: usize,
    surface_matched: usize,
    non_surface_total: usize,
    false_demands: usize,
    problems: Vec<String>,
}

impl CaseOutcome {
    fn ok(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Replay every case of one language through the diff path, one repo, one
/// `HostManager` (so a hosted language starts its server once).
fn run_language(language: &str, scaffold: impl Fn(&Path)) -> Vec<CaseOutcome> {
    let tmp = tempfile::tempdir().expect("tempdir");
    let root = tmp.path();
    scaffold(root);
    init_repo(root);
    commit_all(root, "scaffold");
    let mut manager = HostManager::new(root.to_path_buf());
    let mut outcomes = Vec::new();
    for case in case_dirs(language) {
        let labels = load_labels(&case);
        let (before, after, target) = case_files(&case);
        assert_ne!(before, after, "{}: before equals after", labels.case);
        if let Some(parent) = root.join(target).parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(root.join(target), &before).expect("write before");
        commit_all(root, &format!("{} before", labels.case));
        let base = git_head(root);
        std::fs::write(root.join(target), &after).expect("write after");
        commit_all(root, &format!("{} after", labels.case));
        let head = git_head(root);
        let report = diff_contracts(&mut manager, &base, Some(&head), Duration::from_secs(120))
            .unwrap_or_else(|e| panic!("{}: diff_contracts: {e}", labels.case));
        let problems: Vec<String> =
            report.skipped.iter().map(|s| format!("skipped {}: {}", s.file, s.reason)).collect();
        let events: Vec<Observed> = report
            .files
            .iter()
            .flat_map(|f| f.events.iter())
            .map(|e| Observed {
                symbol: e.event.facts.name.clone(),
                container: e.event.facts.container.clone(),
                change: change_str(e.event.change),
                surface: e.event.surface,
                rule: e.event.rule.unwrap_or("-").to_string(),
            })
            .collect();
        if std::env::var_os("CORPUS_VERBOSE").is_some() {
            println!("{}: [{}]", labels.case, describe_events(&events));
        }
        outcomes.push(judge(&labels, &events, problems));
    }
    outcomes
}

/// Compare one case's observed events against its labels.
fn judge(labels: &Labels, events: &[Observed], mut problems: Vec<String>) -> CaseOutcome {
    let mut surface_matched = 0;
    for label in &labels.surface {
        if events.iter().any(|e| e.surface && label.matches(e)) {
            surface_matched += 1;
        } else {
            problems.push(format!(
                "missed surface change `{}`/{} — observed: [{}]",
                label.symbol,
                label.change,
                describe_events(events)
            ));
        }
    }
    let mut false_demands = 0;
    for label in &labels.non_surface {
        if let Some(e) = events.iter().find(|e| e.surface && label.matches(e)) {
            false_demands += 1;
            problems.push(format!(
                "false demand on `{}`/{} (rule {})",
                label.symbol, e.change, e.rule
            ));
        }
    }
    CaseOutcome {
        name: labels.case.clone(),
        surface_total: labels.surface.len(),
        surface_matched,
        non_surface_total: labels.non_surface.len(),
        false_demands,
        problems,
    }
}

fn describe_events(events: &[Observed]) -> String {
    events
        .iter()
        .map(|e| {
            let s = if e.surface { "surface" } else { "non-surface" };
            format!("{}/{} {s} ({})", e.symbol, e.change, e.rule)
        })
        .collect::<Vec<_>>()
        .join(", ")
}

/// Print the per-case table plus the two aggregate numbers, then fail on
/// any label mismatch.
fn report(language: &str, outcomes: &[CaseOutcome]) {
    println!("\ncorpus: {language} ({} cases)", outcomes.len());
    println!("{:<32} {:>9} {:>9} {:>12} {:>14}  verdict", "case", "surface", "matched", "non-surface", "false-demands");
    let (mut st, mut sm, mut nt, mut fd) = (0, 0, 0, 0);
    for o in outcomes {
        st += o.surface_total;
        sm += o.surface_matched;
        nt += o.non_surface_total;
        fd += o.false_demands;
        let verdict = if o.ok() { "ok" } else { "FAIL" };
        println!(
            "{:<32} {:>9} {:>9} {:>12} {:>14}  {verdict}",
            o.name, o.surface_total, o.surface_matched, o.non_surface_total, o.false_demands
        );
        for p in &o.problems {
            println!("    {p}");
        }
    }
    let recall = if st == 0 { 100.0 } else { 100.0 * sm as f64 / st as f64 };
    let friction = if nt == 0 { 0.0 } else { 100.0 * fd as f64 / nt as f64 };
    println!(
        "{language} totals: recall {sm}/{st} = {recall:.1}%, false-demand {fd}/{nt} = {friction:.1}%"
    );
    let failing: Vec<&str> =
        outcomes.iter().filter(|o| !o.ok()).map(|o| o.name.as_str()).collect();
    assert!(failing.is_empty(), "{language}: cases disagree with their labels: {failing:?}");
}

// ------------------------------------------------------------ per language

/// The adapter's resolved rust-analyzer command as a YAML flow sequence,
/// failing loudly when the toolchain component is missing (it is pinned in
/// rust-toolchain.toml, so CI carries it).
fn rust_host_command_yaml() -> String {
    let command = ddd_lsp::adapter::rust::host_command();
    let program = command.first().cloned().unwrap_or_default();
    assert!(
        Command::new(&program).arg("--version").output().is_ok(),
        "cannot run `{program}` — install it with `rustup component add rust-analyzer`"
    );
    serde_json::to_string(&command).unwrap_or_else(|_| "[]".to_string())
}

#[test]
fn rust_corpus_matches_its_labels() {
    let outcomes = run_language("rust", |root| {
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/rust/Cargo.toml.txt");
        std::fs::write(root.join("Cargo.toml"), read(&manifest)).expect("manifest");
        std::fs::create_dir_all(root.join("src")).expect("mkdir src");
        std::fs::write(root.join("src/lib.rs"), "\n").expect("lib");
        std::fs::create_dir_all(root.join(".ddd")).expect("mkdir .ddd");
        std::fs::write(
            root.join(".ddd/config.yaml"),
            format!(
                "format: 3\nintercept: warn\nignore: []\nadapter:\n  rust:\n    command: {}\n",
                rust_host_command_yaml()
            ),
        )
        .expect("config");
    });
    report("rust", &outcomes);
}

#[test]
fn htmlcss_corpus_matches_its_labels() {
    let outcomes = run_language("htmlcss", |_root| {});
    report("htmlcss", &outcomes);
}
