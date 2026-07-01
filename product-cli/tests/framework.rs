//! Framework What/How-graph integration tests.

#![allow(clippy::unwrap_used)]

use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Test harness: manages a temp dir with product.toml and artifact directories
pub struct Harness {
    pub dir: tempfile::TempDir,
    pub bin: PathBuf,
}

pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl Harness {
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = Self::find_binary();

        // Create product.toml
        // FT-055: by default we disable W030 in test fixtures (most tests
        // don't carry full functional specs). Tests for W030 itself
        // override `[features]` in their own product.toml.
        let config = r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
dependencies = "docs/dependencies"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
dependency = "DEP"
[features]
required-sections = []
functional-spec-subsections = []
"#;
        std::fs::write(dir.path().join("product.toml"), config).expect("write config");
        std::fs::create_dir_all(dir.path().join("docs/features")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("docs/adrs")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("docs/tests")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("docs/graph")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("docs/dependencies")).expect("mkdir");

        Self { dir, bin }
    }

    fn find_binary() -> PathBuf {
        // The binary is built by cargo test
        let mut path = std::env::current_exe().expect("current_exe");
        path.pop(); // remove test binary name
        path.pop(); // remove deps/
        path.push("product");
        if !path.exists() {
            // Try debug directory
            path = PathBuf::from("target/debug/product");
        }
        path
    }

    pub fn write(&self, path: &str, content: &str) -> &Self {
        let full_path = self.dir.path().join(path);
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&full_path, content).expect("write");
        self
    }

    pub fn run(&self, args: &[&str]) -> Output {
        let output = Command::new(&self.bin)
            .args(args)
            .current_dir(self.dir.path())
            .output()
            .expect("run binary");
        Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    pub fn run_with_env(&self, args: &[&str], env: &[(&str, &str)]) -> Output {
        let mut cmd = Command::new(&self.bin);
        cmd.args(args).current_dir(self.dir.path());
        for (k, v) in env {
            cmd.env(k, v);
        }
        let output = cmd.output().expect("run binary");
        Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    pub fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(path)).unwrap_or_default()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.dir.path().join(path).exists()
    }

    /// Create a bare harness — temp dir with no product.toml or directories.
    /// Useful for testing `product init`.
    pub fn new_bare() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = Self::find_binary();
        Self { dir, bin }
    }

    pub fn run_with_stdin(&self, args: &[&str], stdin_data: &str) -> Output {
        use std::io::Write;
        let mut child = Command::new(&self.bin)
            .args(args)
            .current_dir(self.dir.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("spawn binary");

        if let Some(ref mut stdin) = child.stdin {
            let _ = stdin.write_all(stdin_data.as_bytes());
        }
        drop(child.stdin.take());

        let output = child.wait_with_output().expect("wait");
        Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

impl Output {
    pub fn assert_exit(&self, code: i32) -> &Self {
        assert_eq!(
            self.exit_code, code,
            "Expected exit code {}, got {}.\nstdout: {}\nstderr: {}",
            code, self.exit_code, self.stdout, self.stderr
        );
        self
    }

    pub fn assert_stderr_contains(&self, s: &str) -> &Self {
        assert!(
            self.stderr.contains(s),
            "Expected stderr to contain '{}', got:\n{}",
            s, self.stderr
        );
        self
    }

    pub fn assert_stdout_contains(&self, s: &str) -> &Self {
        assert!(
            self.stdout.contains(s),
            "Expected stdout to contain '{}', got:\n{}",
            s, self.stdout
        );
        self
    }
}

// --- Framework What/How-graph integration tests (extracted in the graph-only pivot) ---

#[test]
fn tc_1020_init_demo_seeds_a_conformant_bookstore() {
    let h = Harness::new_bare();
    let out = h.run(&["init", "--yes", "--name", "bookstore", "--demo"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Seeded the bookstore demo"), "stdout:\n{}", out.stdout);

    // The seeded What graph is real and conformant.
    let v = h.run(&["domain", "validate"]);
    v.assert_exit(0);
    assert!(v.stdout.contains("conformant"), "domain validate stdout:\n{}", v.stdout);

    // guide reflects the captured, conformant What.
    let g = h.run(&["guide"]);
    assert!(g.stdout.contains("[x] Captured a What model"), "guide stdout:\n{}", g.stdout);
    assert!(g.stdout.contains("[x] What is conformant"), "guide stdout:\n{}", g.stdout);
}

#[test]
fn init_writes_the_bundled_workflow_skills() {
    let h = Harness::new_bare();
    let out = h.run(&["init", "--yes", "--name", "bookstore"]);
    out.assert_exit(0);
    // The four phase-guide skills land under .claude/skills/ and are listed.
    for name in ["product-session", "product-what", "product-how", "product-build"] {
        let path = format!(".claude/skills/{name}/SKILL.md");
        assert!(h.exists(&path), "{path} should exist after init\nstdout:\n{}", out.stdout);
        assert!(h.read(&path).contains(&format!("name: {name}")), "{name} carries its frontmatter");
    }
}

#[test]
fn init_no_skills_opts_out() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--no-skills"]).assert_exit(0);
    assert!(!h.exists(".claude/skills/product-session/SKILL.md"), "--no-skills should skip the skills");
}

#[test]
fn skills_install_writes_into_an_existing_repo() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--no-skills"]).assert_exit(0);
    let out = h.run(&["skills", "install"]);
    out.assert_exit(0);
    out.assert_stdout_contains("product-how");
    assert!(h.exists(".claude/skills/product-build/SKILL.md"));
}

#[test]
fn skills_install_global_honors_home() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--no-skills"]).assert_exit(0);
    // --global writes under $HOME/.claude/skills; point HOME at the temp dir.
    let home = h.dir.path().to_string_lossy().to_string();
    let out = h.run_with_env(&["skills", "install", "--global"], &[("HOME", &home)]);
    out.assert_exit(0);
    assert!(h.exists(".claude/skills/product-what/SKILL.md"), "global install lands under $HOME");
}

#[test]
fn install_hooks_gates_on_a_live_command() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["install-hooks"]).assert_exit(0);

    let hook = h.read(".git/hooks/pre-commit");
    assert!(hook.contains("product domain validate"), "pre-commit gates on the What graph:\n{hook}");
    // Regression: the pivot removed `adr`; the hook must not invoke a dead command.
    assert!(!hook.contains("adr review"), "hook must not reference the removed adr command");
}

#[test]
fn tc_994_seed_and_list_the_core_aio_set() {
    let h = Harness::new();
    // The closed-core AIO vocabulary (§3.2.2) is always recognised.
    let out = h.run(&["domain", "list", "aio"]);
    out.assert_exit(0);
    for aio in [
        "trigger-action", "single-select", "multi-select", "text-entry", "numeric-entry",
        "date-entry", "display-value", "display-collection", "navigate", "edit",
    ] {
        assert!(out.stdout.contains(aio), "core AIO {aio} should be listed, stdout:\n{}", out.stdout);
    }

    // A context of use is declarable and surfaced.
    let mk = h.run(&[
        "domain", "new", "context-of-use", "phone",
        "--label", "Phone", "--dimension", "form-factor", "--value", "phone",
    ]);
    mk.assert_exit(0);
    let cou = h.run(&["domain", "list", "context-of-use"]);
    cou.assert_exit(0);
    assert!(cou.stdout.contains("phone"), "context of use should be listed, stdout:\n{}", cou.stdout);
}

#[test]
fn tc_995_uistep_typed_against_aios_passes_structural_check() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // A UI step whose interactions are typed against core AIOs is conformant.
    let mk = h.run(&[
        "domain", "new", "ui-step", "ReviewOrder", "--label", "Review order",
        "--surfaces", "OrderSummary:display-collection",
        "--offers", "PlaceOrder:trigger-action",
    ]);
    mk.assert_exit(0);
    let v = h.run(&["domain", "validate"]);
    v.assert_exit(0);
    assert!(v.stdout.contains("conformant"), "stdout:\n{}", v.stdout);
}

#[test]
fn tc_996_uistep_referencing_a_cio_fails_the_aio_only_rule() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // A UI step naming a concrete control (a CIO) is a structural violation.
    let out = h.run(&[
        "domain", "new", "ui-step", "BadStep", "--label", "Bad",
        "--offers", "PlaceOrder:primary-button",
    ]);
    out.assert_exit(1);
    assert!(
        out.stderr.contains("AIO") || out.stderr.contains("typedAs"),
        "should reject the CIO reference, stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_997_mark_flow_entry_page_and_navigate_edges() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Browse", "--label", "Browse", "--transitions-to", "Review"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Review", "--label", "Review"]).assert_exit(0);
    h.run(&["domain", "new", "flow", "checkout", "--label", "Checkout", "--steps", "Browse,Review", "--entry-page", "Browse"]).assert_exit(0);
    // The flow records its entry page; the navigate edge is in the export.
    let fl = h.run(&["domain", "list", "flow"]);
    assert!(fl.stdout.contains("entry: Browse"), "flow should show entry page, stdout:\n{}", fl.stdout);
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("pf:transitionsTo d:Review"), "navigate edge missing, stdout:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:entryPage d:Browse"), "entry page missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_998_top_level_is_derived_from_the_application_root() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Browse", "--label", "Browse"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Review", "--label", "Review"]).assert_exit(0);
    h.run(&["domain", "new", "application-root", "root", "--label", "App", "--navigates-from-root", "Browse"]).assert_exit(0);
    // Browse has an inbound edge from the root → top-level; Review is nested.
    let out = h.run(&["domain", "list", "ui-step"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("Browse [top-level]"), "Browse should be top-level, stdout:\n{}", out.stdout);
    assert!(!out.stdout.contains("Review [top-level]"), "Review should be nested, stdout:\n{}", out.stdout);
}

#[test]
fn tc_1030_system_is_a_first_class_what_node() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // §3.2.5 — a system without a purpose is rejected.
    h.run(&["domain", "new", "system", "sys-bad", "--label", "Bad", "--system-kind", "application"])
        .assert_exit(1)
        .assert_stderr_contains("§3.2.5");
    // A complete system is captured and conformant.
    h.run(&[
        "domain", "new", "system", "sys-shop", "--label", "Acme Shop", "--system-kind", "application",
        "--purpose", "consumer e-commerce", "--target-platforms", "ios,web", "--target-classes", "gui",
    ])
    .assert_exit(0);
    h.run(&["domain", "validate"]).assert_exit(0);
    // Its identity and reach are in the Turtle export under pf:System.
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("a pf:System"), "system class missing, stdout:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:systemKind"), "system kind missing, stdout:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:targetsClass"), "interaction class missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1031_a_flow_belongs_to_a_declared_system() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&[
        "domain", "new", "system", "sys-shop", "--label", "Acme Shop",
        "--system-kind", "application", "--purpose", "shop",
    ])
    .assert_exit(0);
    // A flow owned by a declared system is accepted.
    h.run(&["domain", "new", "flow", "checkout", "--label", "Checkout", "--system", "sys-shop"]).assert_exit(0);
    // A flow naming an undeclared system is a §3.2.5 finding.
    h.run(&["domain", "new", "flow", "ghost", "--label", "Ghost", "--system", "no-such-system"])
        .assert_exit(1)
        .assert_stderr_contains("§3.2.5");
    // The ownership edge is in the export.
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("pf:systemOf d:sys-shop"), "ownership edge missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1032_trigger_block_issues_a_command() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // §3.2.0 — a trigger's source must be user/external/automated.
    h.run(&["domain", "new", "trigger", "t-bad", "--label", "Bad", "--trigger-source", "robot", "--issues", "PlaceOrder"])
        .assert_exit(1)
        .assert_stderr_contains("§3.2.0");
    // A user trigger issuing a declared command is the Command pattern's start.
    h.run(&["domain", "new", "trigger", "t-user", "--label", "User places order", "--trigger-source", "user", "--issues", "PlaceOrder"])
        .assert_exit(0);
    h.run(&["domain", "validate"]).assert_exit(0);
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("a pf:Trigger"), "trigger class missing, stdout:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:issues d:PlaceOrder"), "issues edge missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1033_automation_trigger_must_watch_a_view() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // §3.2.0 Automation — an automated trigger that watches no View is a finding.
    h.run(&["domain", "new", "trigger", "t-auto-bad", "--label", "Auto", "--trigger-source", "automated", "--issues", "PlaceOrder"])
        .assert_exit(1)
        .assert_stderr_contains("watch a View");
    // Watching a declared read model satisfies the Automation pattern shape.
    h.run(&["domain", "new", "trigger", "t-auto", "--label", "Auto restock", "--trigger-source", "automated", "--issues", "PlaceOrder", "--watches", "OrderSummary"])
        .assert_exit(0);
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("pf:watches d:OrderSummary"), "watches edge missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1034_interaction_class_is_the_gating_context_dimension() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // §3.2.2 — a system may target the recognised gui/tui classes.
    h.run(&[
        "domain", "new", "system", "sys-cli", "--label", "Dev CLI",
        "--system-kind", "cli", "--purpose", "developer tool", "--target-classes", "tui",
    ])
    .assert_exit(0);
    // An unrecognised interaction class is a §3.2.2 finding.
    h.run(&[
        "domain", "new", "system", "sys-bad", "--label", "Bad",
        "--system-kind", "application", "--purpose", "x", "--target-classes", "holographic",
    ])
    .assert_exit(1)
    .assert_stderr_contains("§3.2.2");
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("pf:targetsClass \"tui\""), "class edge missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1035_state_and_decider_justification_are_advisory_findings() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // A guard-less Decider over the demo's Order aggregate: it evolves `placed`
    // but reads nothing and never rejects — both are §3.3 model-gap findings.
    h.write(
        ".product/deciders/Order-decider.yaml",
        "id: Order-decider\ndecides_for: Order\nhandles:\n- PlaceOrder\nemits:\n- OrderPlaced\nevolves_from:\n- OrderPlaced\nlogic:\n  initial:\n    placed: false\n  evolve:\n  - on: OrderPlaced\n    set:\n      placed: true\n  decide:\n  - on: PlaceOrder\n    emit:\n    - OrderPlaced\n",
    );
    let out = h.run(&["decider", "validate", "Order-decider"]);
    // §3.3/§3.4 — the findings are advisory warnings, not blocking gates.
    out.assert_exit(0);
    assert!(
        out.stderr.contains("State justification") && out.stderr.contains("placed"),
        "state justification warning missing, stderr:\n{}",
        out.stderr
    );
    assert!(
        out.stderr.contains("Decider justification"),
        "decider justification warning missing, stderr:\n{}",
        out.stderr
    );
}

#[test]
fn tc_1036_unreifiable_aio_is_a_recorded_coverage_gap() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // §4.5 — a recorded gap needs a rationale; without one it is a silent omission.
    h.run(&["domain", "new", "unreifiable-rule", "u-bad", "--aio", "display-collection", "--class", "tui"])
        .assert_exit(1)
        .assert_stderr_contains("§4.5");
    // A complete recorded gap (real AIO, recognised class, a rationale) is captured.
    h.run(&[
        "domain", "new", "unreifiable-rule", "u-gallery", "--aio", "display-collection",
        "--class", "tui", "--rationale", "an image gallery has no faithful character-grid form",
    ])
    .assert_exit(0);
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("a pf:UnreifiableRule"), "class missing, stdout:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:unreifiableIn \"tui\""), "unreifiableIn edge missing, stdout:\n{}", ttl.stdout);
}

#[test]
fn tc_1037_strict_what_conformance_checks_graph_completeness() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // Ordinary validate is per-node and passes; strict adds graph-level
    // completeness (a command needs a trigger, a view needs a consumer).
    h.run(&["domain", "validate"]).assert_exit(0);
    let strict = h.run(&["domain", "validate", "--strict"]);
    strict.assert_exit(1);
    assert!(
        strict.stderr.contains("§3.2.0") && strict.stderr.contains("Command pattern"),
        "command-pattern finding missing, stderr:\n{}",
        strict.stderr
    );
    assert!(
        strict.stderr.contains("§3.4") && strict.stderr.contains("View consumed"),
        "view-consumed finding missing, stderr:\n{}",
        strict.stderr
    );
    // Closing the gaps — a trigger for the command, a UI step surfacing the view —
    // makes the strict check pass.
    h.run(&["domain", "new", "trigger", "t-place", "--label", "Place", "--trigger-source", "user", "--issues", "PlaceOrder"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Review", "--label", "Review", "--surfaces", "OrderSummary:display-collection"]).assert_exit(0);
    h.run(&["domain", "validate", "--strict"]).assert_exit(0);
}

#[test]
fn tc_999_primary_navigation_recomputes_when_a_flow_joins_the_root() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Browse", "--label", "Browse"]).assert_exit(0);
    // Before joining the root, nothing is top-level.
    let before = h.run(&["domain", "list", "ui-step"]);
    assert!(!before.stdout.contains("[top-level]"), "nothing top-level yet, stdout:\n{}", before.stdout);
    // Adding a root edge recomputes the primary navigation set.
    h.run(&["domain", "new", "application-root", "root", "--navigates-from-root", "Browse"]).assert_exit(0);
    let after = h.run(&["domain", "list", "ui-step"]);
    assert!(after.stdout.contains("Browse [top-level]"), "Browse should now be top-level, stdout:\n{}", after.stdout);
}

// FT-136 — read-model state space + UI state coverage helpers.
fn setup_state_space(h: &Harness) {
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // OrderSummary's state space: present + empty + failed.
    h.run(&["domain", "edit", "OrderSummary", "--states", "empty,failed"]).assert_exit(0);
}

#[test]
fn tc_1000_ui_step_covers_every_projection_state() {
    let h = Harness::new_bare();
    setup_state_space(&h);
    let mk = h.run(&[
        "domain", "new", "ui-step", "Review", "--label", "Review",
        "--surfaces", "OrderSummary:display-collection",
        "--state-meaning", "OrderSummary:present:The order total",
        "--state-meaning", "OrderSummary:empty:Your cart is empty",
        "--state-meaning", "OrderSummary:failed:Could not load",
    ]);
    mk.assert_exit(0);
    h.run(&["domain", "validate"]).assert_exit(0);
}

#[test]
fn tc_1001_forgotten_failed_state_fails_coverage() {
    let h = Harness::new_bare();
    setup_state_space(&h);
    // Omitting the `failed` meaning (and not waiving it) is a coverage violation.
    let out = h.run(&[
        "domain", "new", "ui-step", "Forgetful", "--label", "F",
        "--surfaces", "OrderSummary:display-collection",
        "--state-meaning", "OrderSummary:present:total",
        "--state-meaning", "OrderSummary:empty:empty",
    ]);
    out.assert_exit(1);
    assert!(out.stderr.contains("failed") && out.stderr.contains("coverage"), "stderr:\n{}", out.stderr);
}

#[test]
fn tc_1002_waiving_an_ignorable_state_passes_with_reason() {
    let h = Harness::new_bare();
    setup_state_space(&h);
    // Waiving `failed` with a reason satisfies coverage.
    let out = h.run(&[
        "domain", "new", "ui-step", "Waived", "--label", "W",
        "--surfaces", "OrderSummary:display-collection",
        "--state-meaning", "OrderSummary:present:total",
        "--state-meaning", "OrderSummary:empty:empty",
        "--waive-state", "OrderSummary:failed:logged elsewhere",
    ]);
    out.assert_exit(0);
}

#[test]
fn tc_1003_step_inherits_accessibility_obligations_from_its_aios() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // A step using a text-entry AIO inherits its labelling criteria (the union),
    // each annotated with the AIO it came from — no hand-maintained list.
    h.run(&["domain", "new", "ui-step", "EditProfile", "--label", "Edit", "--offers", "PlaceOrder:text-entry"]).assert_exit(0);
    let out = h.run(&["domain", "accessibility", "EditProfile"]);
    assert!(out.stdout.contains("3.3.2") && out.stdout.contains("from text-entry"),
        "union should inherit labelling from text-entry, stdout:\n{}", out.stdout);
    assert!(!out.stdout.contains("1.1.1"), "no display-value yet, stdout:\n{}", out.stdout);
    // Adding a display-value AIO adds 1.1.1 Non-text Content to the union.
    h.run(&["domain", "new", "ui-step", "WithImage", "--label", "Img",
        "--offers", "PlaceOrder:text-entry", "--surfaces", "OrderSummary:display-value"]).assert_exit(0);
    let out2 = h.run(&["domain", "accessibility", "WithImage"]);
    assert!(out2.stdout.contains("1.1.1"), "display-value should add 1.1.1, stdout:\n{}", out2.stdout);
}

#[test]
fn tc_1004_machine_criterion_is_a_deterministic_gate() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "wcag-criterion", "contrast", "--label", "Contrast", "--level", "AA", "--verification", "machine"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Page1", "--label", "P1", "--must-satisfy", "contrast"]).assert_exit(0);
    // Unsatisfied machine criterion → the gate fails (mechanical).
    let fail = h.run(&["domain", "accessibility", "Page1"]);
    fail.assert_exit(1);
    assert!(fail.stdout.contains("AA") && fail.stdout.contains("machine"), "verdict reports level + basis, stdout:\n{}", fail.stdout);
    // Satisfied → the gate passes.
    h.run(&["domain", "edit", "contrast", "--satisfied", "true"]).assert_exit(0);
    h.run(&["domain", "accessibility", "Page1"]).assert_exit(0);
}

#[test]
fn tc_1005_assisted_criterion_discharged_by_attestation() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "wcag-criterion", "focusvis", "--label", "Focus", "--level", "AA", "--verification", "assisted"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Page2", "--label", "P2", "--must-satisfy", "focusvis"]).assert_exit(0);
    // No attestation → undischarged.
    h.run(&["domain", "accessibility", "Page2"]).assert_exit(1);
    // A dated, attributed attestation discharges it.
    h.run(&["domain", "new", "attestation", "att1", "--step", "Page2", "--criterion", "focusvis", "--date", "2026-06-19", "--by", "QA"]).assert_exit(0);
    h.run(&["domain", "accessibility", "Page2"]).assert_exit(0);
    // An attestation missing its date/attribution is rejected at the boundary.
    let bad = h.run(&["domain", "new", "attestation", "att2", "--step", "Page2", "--criterion", "focusvis"]);
    bad.assert_exit(1);
}

#[test]
fn tc_1006_ui_step_references_content_by_key_and_role() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // Content is carried by key + role, recorded as references_content edges.
    h.run(&["domain", "new", "ui-step", "ReviewOrder", "--label", "Review",
        "--content", "checkout.review.heading:heading", "--content", "cart.empty.message:empty-message"]).assert_exit(0);
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("referencesContent") && ttl.stdout.contains("checkout.review.heading"),
        "content refs should be in the export, stdout:\n{}", ttl.stdout);
    // A literal sentence baked in as a "key" is rejected (no literals on the What).
    let bad = h.run(&["domain", "new", "ui-step", "BadStep", "--label", "Bad", "--content", "Review your order:heading"]);
    bad.assert_exit(1);
    assert!(bad.stderr.contains("keyed reference"), "stderr:\n{}", bad.stderr);
}

#[test]
fn tc_1007_content_coverage_over_key_and_locale() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "ReviewOrder", "--label", "Review",
        "--content", "checkout.review.heading:heading", "--content", "cart.empty.message:body"]).assert_exit(0);
    // A store covering both keys in en+es → coverage passes.
    h.run(&["domain", "new", "content-store", "store", "--locales", "en,es",
        "--resolves", "checkout.review.heading:en:Review your order",
        "--resolves", "checkout.review.heading:es:Revisa tu pedido",
        "--resolves", "cart.empty.message:en:Nothing here",
        "--resolves", "cart.empty.message:es:Nada aqui"]).assert_exit(0);
    h.run(&["domain", "validate"]).assert_exit(0);
    // Dropping the es value for one key → coverage fails naming the pair.
    h.run(&["domain", "edit", "store", "--locales", "en,es",
        "--resolves", "checkout.review.heading:en:Review",
        "--resolves", "checkout.review.heading:es:Revisa",
        "--resolves", "cart.empty.message:en:Nothing"]).assert_exit(0);
    let fail = h.run(&["domain", "validate"]);
    fail.assert_exit(1);
    assert!(fail.stderr.contains("cart.empty.message") && fail.stderr.contains("es"),
        "should name the missing (key, locale), stderr:\n{}", fail.stderr);
}

#[test]
fn tc_1008_role_conformance_catches_empty_error_message() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Cart", "--label", "Cart", "--content", "cart.failed.message:error-message"]).assert_exit(0);
    // An error-message role resolving to empty is caught at check time.
    h.run(&["domain", "new", "content-store", "store", "--locales", "en", "--resolves", "cart.failed.message:en:"]).assert_exit(0);
    let fail = h.run(&["domain", "validate"]);
    fail.assert_exit(1);
    assert!(fail.stderr.contains("empty") && fail.stderr.contains("error-message"), "stderr:\n{}", fail.stderr);
    // A non-empty resolution passes.
    h.run(&["domain", "edit", "store", "--locales", "en", "--resolves", "cart.failed.message:en:Could not load. Retry."]).assert_exit(0);
    h.run(&["domain", "validate"]).assert_exit(0);
}

// FT-139 — design system + reification helpers.
fn setup_design_system(h: &Harness) {
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "context-of-use", "phone", "--label", "Phone"]).assert_exit(0);
    h.run(&["domain", "new", "context-of-use", "tablet", "--label", "Tablet"]).assert_exit(0);
    h.run(&["domain", "new", "design-system", "ds",
        "--cios", "segmented-control,searchable-list,primary-button", "--tokens", "color.accent"]).assert_exit(0);
}

#[test]
fn tc_1009_aio_reifies_to_different_cios_by_context() {
    let h = Harness::new_bare();
    setup_design_system(&h);
    h.run(&["domain", "new", "reification-rule", "r1", "--aio", "single-select",
        "--context", "tablet", "--cio", "segmented-control", "--rationale", "few options, ample width"]).assert_exit(0);
    h.run(&["domain", "new", "reification-rule", "r2", "--aio", "single-select",
        "--context", "phone", "--cio", "searchable-list", "--rationale", "no room for many"]).assert_exit(0);
    let out = h.run(&["domain", "reification", "single-select"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("segmented-control") && out.stdout.contains("searchable-list"),
        "one AIO reifies to two CIOs by context, stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("few options"), "rationale should show, stdout:\n{}", out.stdout);
}

#[test]
fn tc_1010_reification_coverage_over_aio_and_context() {
    let h = Harness::new_bare();
    setup_design_system(&h);
    h.run(&["domain", "new", "ui-step", "Pick", "--label", "Pick", "--offers", "PlaceOrder:single-select"]).assert_exit(0);
    h.run(&["domain", "new", "reification-rule", "r1", "--aio", "single-select", "--context", "tablet", "--cio", "segmented-control"]).assert_exit(0);
    h.run(&["domain", "new", "reification-rule", "r2", "--aio", "single-select", "--context", "phone", "--cio", "searchable-list"]).assert_exit(0);
    h.run(&["domain", "reification", "--check"]).assert_exit(0);
    // Removing the (single-select, phone) rule makes coverage incomplete.
    h.run(&["domain", "rm", "r2"]).assert_exit(0);
    let fail = h.run(&["domain", "reification", "--check"]);
    fail.assert_exit(1);
    assert!(fail.stderr.contains("single-select") && fail.stderr.contains("phone"),
        "should name the uncovered (AIO, context), stderr:\n{}", fail.stderr);
}

#[test]
fn tc_1011_off_system_component_and_literal_style_are_rejected() {
    let h = Harness::new_bare();
    setup_design_system(&h);
    // A reification rule targeting a non-catalog CIO fails the closed-vocab check.
    h.run(&["domain", "new", "reification-rule", "bad", "--aio", "trigger-action", "--context", "phone", "--cio", "fancy-carousel"]).assert_exit(0);
    let fail = h.run(&["domain", "reification", "--check"]);
    fail.assert_exit(1);
    assert!(fail.stderr.contains("fancy-carousel"), "off-system CIO named, stderr:\n{}", fail.stderr);
    // A literal style value (not a token) is non-conformant.
    h.run(&["domain", "rm", "bad"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Styled", "--label", "S", "--styles", "#3366ff"]).assert_exit(0);
    let fail2 = h.run(&["domain", "reification", "--check"]);
    fail2.assert_exit(1);
    assert!(fail2.stderr.contains("literal"), "literal style rejected, stderr:\n{}", fail2.stderr);
}

/// Author the §3.1 structure/data split, then `domain data` finds clean
/// production data conformant with zero divergence.
fn author_order_data_split(h: &Harness) {
    h.run(&["domain", "new", "context", "Sales", "--label", "Sales"]).assert_exit(0);
    h.run(&["domain", "new", "entity", "Order", "--label", "Order", "--definition", "a customer order", "--context", "Sales"]).assert_exit(0);
    h.run(&["domain", "new", "reference-set", "ShippingMethods", "--concept", "Order", "--values", "standard,express"]).assert_exit(0);
    h.run(&["domain", "new", "data-shape", "OrderShape", "--target", "Order", "--required", "id,total", "--enum", "shipping=ShippingMethods"]).assert_exit(0);
    h.run(&["domain", "new", "production-dataset", "OrdersLive", "--shape", "OrderShape", "--source", "orders.json"]).assert_exit(0);
}

#[test]
fn tc_1021_author_the_structure_data_split() {
    let h = Harness::new();
    author_order_data_split(&h);
    // Reference data, the shape, and the dataset are all in the graph.
    let v = h.run(&["domain", "validate"]);
    v.assert_exit(0);
    assert!(v.stdout.contains("conformant"), "stdout:\n{}", v.stdout);
    let list = h.run(&["domain", "list", "reference-set"]);
    assert!(list.stdout.contains("ShippingMethods"), "stdout:\n{}", list.stdout);
    // The data side exports as RDF on the structure/data split predicates.
    let ttl = h.run(&["domain", "export"]);
    assert!(ttl.stdout.contains("pf:referenceDataFor d:Order"), "ttl:\n{}", ttl.stdout);
    assert!(ttl.stdout.contains("pf:conformsToShape d:OrderShape"), "ttl:\n{}", ttl.stdout);
}

#[test]
fn tc_1022_clean_production_data_has_zero_divergence() {
    let h = Harness::new();
    author_order_data_split(&h);
    h.write("orders.json", r#"[{"id":"o1","total":10,"shipping":"standard"},{"id":"o2","total":20,"shipping":"express"}]"#);
    let out = h.run(&["domain", "data", "OrdersLive"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("divergence rate 0.0%"), "stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("all records conform"), "stdout:\n{}", out.stdout);
}

#[test]
fn tc_1023_data_conformance_catches_drift_and_reports_the_rate() {
    let h = Harness::new();
    author_order_data_split(&h);
    // One row drops a required field and carries an enum value the set never declared.
    h.write("orders.json", r#"[{"id":"o1","total":10,"shipping":"standard"},{"id":"o2","shipping":"drone"},{"id":"o3","total":5,"shipping":"express"}]"#);
    let out = h.run(&["domain", "data", "OrdersLive"]);
    out.assert_exit(1);
    assert!(out.stdout.contains("divergence rate 33.3%"), "stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("missing-required"), "stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("not-in-reference-set"), "stdout:\n{}", out.stdout);
    // The verdict reads both ways (data wrong or spec stale).
    assert!(out.stderr.contains("fix the data") && out.stderr.contains("fix the shape"), "stderr:\n{}", out.stderr);
}

#[test]
fn tc_1024_validate_catches_dangling_data_cross_references() {
    let h = Harness::new();
    h.run(&["domain", "new", "context", "Sales", "--label", "Sales"]).assert_exit(0);
    h.run(&["domain", "new", "entity", "Order", "--label", "Order", "--definition", "d", "--context", "Sales"]).assert_exit(0);
    // A shape targeting a non-existent entity is authorable but caught by validate.
    h.run(&["domain", "new", "data-shape", "GhostShape", "--target", "Nonexistent"]).assert_exit(0);
    let v = h.run(&["domain", "validate"]);
    v.assert_exit(1);
    assert!(v.stderr.contains("GhostShape"), "stderr:\n{}", v.stderr);
}

#[test]
fn tc_1025_data_shape_datatype_constraint_catches_type_drift() {
    let h = Harness::new();
    h.run(&["domain", "new", "context", "Sales", "--label", "Sales"]).assert_exit(0);
    h.run(&["domain", "new", "entity", "Order", "--label", "Order", "--definition", "d", "--context", "Sales"]).assert_exit(0);
    h.run(&["domain", "new", "data-shape", "OrderShape", "--target", "Order", "--required", "id", "--type", "total=integer"]).assert_exit(0);
    h.run(&["domain", "new", "production-dataset", "OrdersLive", "--shape", "OrderShape", "--source", "orders.json"]).assert_exit(0);
    h.write("orders.json", r#"[{"id":"o1","total":10},{"id":"o2","total":"twelve"}]"#);
    let out = h.run(&["domain", "data", "OrdersLive"]);
    out.assert_exit(1);
    assert!(out.stdout.contains("not-of-type"), "stdout:\n{}", out.stdout);
    assert!(out.stdout.contains("divergence rate 50.0%"), "stdout:\n{}", out.stdout);
}

#[test]
fn tc_1026_divergence_rate_trend_is_surfaced_across_runs() {
    let h = Harness::new();
    author_order_data_split(&h);
    // First run: clean data, zero divergence, recorded as the baseline.
    h.write("orders.json", r#"[{"id":"o1","total":10,"shipping":"standard"}]"#);
    let first = h.run(&["domain", "data", "OrdersLive"]);
    first.assert_exit(0);
    assert!(first.stdout.contains("first run"), "stdout:\n{}", first.stdout);
    // Second run: data has drifted — the trend reports the rate rising.
    h.write("orders.json", r#"[{"id":"o1","total":10,"shipping":"drone"}]"#);
    let second = h.run(&["domain", "data", "OrdersLive"]);
    second.assert_exit(1);
    assert!(second.stdout.contains("rising"), "trend should rise, stdout:\n{}", second.stdout);
    // --no-record leaves the history untouched (no standing signal written).
    let n = h.run(&["domain", "data", "OrdersLive", "--no-record"]);
    n.assert_exit(1);
    assert!(!h.exists(".product/author-domain/test/data-history.jsonl")
        || h.read(".product/author-domain/test/data-history.jsonl").lines().count() == 2,
        "history should hold exactly the two recorded runs");
}

#[test]
fn domain_rm_deletes_every_node_kind() {
    // Regression: `remove` must cover all node kinds, not just the original 11.
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    for (kind, id, extra) in [
        ("aio", "range-select", vec!["--label", "R"]),
        ("context-of-use", "phone", vec!["--label", "P"]),
        ("design-system", "ds", vec!["--cios", "primary-button"]),
        ("cio", "primary-button", vec!["--label", "B"]),
    ] {
        let mut args = vec!["domain", "new", kind, id];
        args.extend(extra);
        h.run(&args).assert_exit(0);
        // The node deletes, and a second delete fails (it is truly gone).
        h.run(&["domain", "rm", id]).assert_exit(0);
        h.run(&["domain", "rm", id]).assert_exit(1);
    }
}

#[test]
fn tc_1012_seam_passes_when_screen_and_step_agree() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // ReviewOrder surfaces a real projection, offers a real command, AIO-typed.
    h.run(&["domain", "new", "ui-step", "ReviewOrder", "--label", "Review",
        "--surfaces", "OrderSummary:display-collection", "--offers", "PlaceOrder:trigger-action"]).assert_exit(0);
    let out = h.run(&["seam", "ReviewOrder"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("conformant") && out.stdout.contains("✓ datum-projected"),
        "seam verdict reports each passing sub-check, stdout:\n{}", out.stdout);
}

#[test]
fn tc_1013_seam_fails_on_unprojected_datum_or_foreign_command() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // A page displaying data no read model projects fails datum-projected.
    h.run(&["domain", "new", "ui-step", "BadDatum", "--label", "B", "--surfaces", "Nonexistent:display-collection"]).assert_exit(0);
    let d = h.run(&["seam", "BadDatum"]);
    d.assert_exit(1);
    assert!(d.stdout.contains("✗ datum-projected") && d.stderr.contains("Nonexistent"), "stdout:\n{}\nstderr:\n{}", d.stdout, d.stderr);
    // A control issuing a command the step cannot accept fails control-maps-to-command.
    h.run(&["domain", "new", "ui-step", "BadCmd", "--label", "B", "--offers", "GhostCmd:trigger-action"]).assert_exit(0);
    let c = h.run(&["seam", "BadCmd"]);
    c.assert_exit(1);
    assert!(c.stdout.contains("✗ control-maps-to-command") && c.stderr.contains("GhostCmd"), "stdout:\n{}\nstderr:\n{}", c.stdout, c.stderr);
}

#[test]
fn tc_1014_seam_composes_coverage_failures() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    // Create a conformant step, then let the model drift around it so the seam
    // catches a state gap, a content gap, and a reification gap independently.
    h.run(&["domain", "new", "ui-step", "Multi", "--label", "M",
        "--surfaces", "OrderSummary:single-select", "--content", "x.y:heading"]).assert_exit(0);
    h.run(&["domain", "edit", "OrderSummary", "--states", "empty"]).assert_exit(0);     // state gap
    h.run(&["domain", "new", "content-store", "cs", "--locales", "en"]).assert_exit(0); // content gap (no x.y)
    h.run(&["domain", "new", "context-of-use", "phone", "--label", "P"]).assert_exit(0);// reify gap (no rule)
    let out = h.run(&["seam", "Multi"]);
    out.assert_exit(1);
    for sub in ["✗ state-coverage", "✗ content-coverage", "✗ reification-coverage"] {
        assert!(out.stdout.contains(sub), "composite should list {sub}, stdout:\n{}", out.stdout);
    }
}

/// A §11.3 design-system manifest (canonical YAML) with the given reification
/// rules (each `(aio, when, cio)`). `whole_ds_manifest` reifies single-select on
/// phone and trigger-action by emphasis.
fn ds_manifest(rules: &[(&str, &str, &str)]) -> String {
    let reify: String = rules.iter().map(|(aio, when, cio)| format!(
        "    - {{ aio: {aio}, when: {{ {when} }}, cio: {cio}, rationale: x }}\n")).collect();
    format!(
        "design_system:\n  id: acme\n  version: \"1.0\"\n  wcag_target: \"2.2-AA\"\n\
         \x20 contexts_supported: {{ form_factor: [phone, tablet], modality: [touch] }}\n\
         \x20 components:\n\
         \x20   - {{ id: searchable-list, tokens: [color.accent], satisfies: [{{ criterion: \"1.3.1\", level: A, via: machine }}] }}\n\
         \x20   - {{ id: primary-button, tokens: [color.accent], satisfies: [{{ criterion: \"2.5.8\", level: AA, via: machine }}] }}\n\
         \x20 reification:\n{reify}\
         \x20 tokens: [{{ id: color.accent, type: color }}]\n")
}

fn whole_ds_manifest() -> String {
    ds_manifest(&[
        ("single-select", "form_factor: phone", "searchable-list"),
        ("trigger-action", "emphasis: primary", "primary-button"),
    ])
}

#[test]
fn tc_1015_design_system_manifest_validates_internally() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    h.write("ds.yaml", &whole_ds_manifest());
    h.run(&["preview", "design-system", "ds.yaml"]).assert_exit(0);
    // A reification naming a cio absent from components fails wholeness.
    h.write("bad.yaml", &whole_ds_manifest().replacen("cio: searchable-list", "cio: ghost-cio", 1));
    let out = h.run(&["preview", "design-system", "bad.yaml"]);
    out.assert_exit(1);
    assert!(out.stderr.contains("ghost-cio") && out.stderr.contains("absent"), "stderr:\n{}", out.stderr);
}

#[test]
fn tc_1016_design_system_coupling_covers_every_aio_context() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "context-of-use", "phone", "--label", "P",
        "--dimension", "form_factor", "--value", "phone"]).assert_exit(0);
    // A UI step referencing single-select (surfaced) + trigger-action (offered),
    // against the demo's existing read model + command.
    h.run(&["domain", "new", "ui-step", "Pick", "--label", "Pick",
        "--surfaces", "OrderSummary:single-select", "--offers", "PlaceOrder:trigger-action"]).assert_exit(0);
    h.write("ds.yaml", &whole_ds_manifest());
    // Both referenced AIOs reify on phone → coupling complete.
    h.run(&["preview", "design-system", "ds.yaml", "--couple"]).assert_exit(0);
    // Drop single-select's rule → non-conforming for phone, naming the gap.
    h.write("gap.yaml", &ds_manifest(&[("trigger-action", "emphasis: primary", "primary-button")]));
    let out = h.run(&["preview", "design-system", "gap.yaml", "--couple"]);
    out.assert_exit(1);
    assert!(out.stderr.contains("single-select") && out.stderr.contains("phone"), "stderr:\n{}", out.stderr);
}

/// A whole §12.2 content-store manifest (canonical YAML) with two entries over en/de.
fn whole_content_manifest() -> String {
    "content_store:\n  id: copy\n  version: \"1.0\"\n  locales_supported: [en, de]\n  entries:\n\
     \x20   - key: cart.empty.message\n      role: empty-message\n\
     \x20     values: { en: \"Your cart is empty\", de: \"Ihr Warenkorb ist leer\" }\n\
     \x20   - key: checkout.title\n      role: heading\n\
     \x20     values: { en: \"Checkout\", de: \"Kasse\" }\n".to_string()
}

#[test]
fn tc_1017_content_store_manifest_validates_internally() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    h.write("cs.yaml", &whole_content_manifest());
    h.run(&["preview", "content-store", "cs.yaml"]).assert_exit(0);
    // A key missing a value for a claimed locale fails wholeness.
    h.write("nolocale.yaml", &whole_content_manifest().replacen(", de: \"Kasse\"", "", 1));
    let a = h.run(&["preview", "content-store", "nolocale.yaml"]);
    a.assert_exit(1);
    assert!(a.stderr.contains("checkout.title") && a.stderr.contains("de"), "stderr:\n{}", a.stderr);
    // An error/empty-message role resolving to empty text fails.
    h.write("empty.yaml", &whole_content_manifest().replacen("Your cart is empty", "", 1));
    let b = h.run(&["preview", "content-store", "empty.yaml"]);
    b.assert_exit(1);
    assert!(b.stderr.contains("cart.empty.message") && b.stderr.contains("empty"), "stderr:\n{}", b.stderr);
}

#[test]
fn tc_1018_content_store_coupling_resolves_every_referenced_key() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Cart", "--label", "Cart",
        "--content", "cart.empty.message:empty-message"]).assert_exit(0);
    h.write("cs.yaml", &whole_content_manifest());
    // The store resolves the referenced key in every locale → coupling complete.
    h.run(&["preview", "content-store", "cs.yaml", "--couple"]).assert_exit(0);
    // A UI step referencing an unresolved key → non-conforming for that locale.
    h.run(&["domain", "new", "ui-step", "P2", "--label", "P2", "--content", "missing.key:body"]).assert_exit(0);
    let out = h.run(&["preview", "content-store", "cs.yaml", "--couple"]);
    out.assert_exit(1);
    assert!(out.stderr.contains("missing.key") && (out.stderr.contains("de") || out.stderr.contains("en")), "stderr:\n{}", out.stderr);
}

#[test]
fn tc_1029_data_conformance_is_adoptable_standalone() {
    // A graph with ONLY a domain structure + a production dataset — no event
    // model, Decider, Projector, UI, or work units — is §13's minimal adoption.
    let h = Harness::new();
    author_order_data_split(&h);
    // Structurally valid with nothing but its data side.
    h.run(&["domain", "validate"]).assert_exit(0);
    // Data conformance runs end to end and reports the divergence rate.
    h.write("orders.json", r#"[{"id":"o1","total":10,"shipping":"standard"}]"#);
    let out = h.run(&["domain", "data", "OrdersLive"]);
    out.assert_exit(0);
    assert!(out.stdout.contains("divergence rate"), "should report the rate, stdout:\n{}", out.stdout);
}

/// Build a small page graph: root → flow `checkout` with one UI step `Review`
/// that surfaces a display-value (WCAG-bearing) and offers a trigger-action.
fn seed_page_graph(h: &Harness) {
    // The demo already seeds OrderSummary + PlaceOrder. Create the step while the
    // read model has no states, then add states (the render contract projects the
    // state space regardless of step-level coverage).
    h.run(&["domain", "new", "application-root", "root", "--label", "Root",
        "--navigates-from-root", "Review"]).assert_exit(0);
    h.run(&["domain", "new", "ui-step", "Review", "--label", "Review", "--intent", "Confirm",
        "--surfaces", "OrderSummary:display-value", "--offers", "PlaceOrder:trigger-action"]).assert_exit(0);
    h.run(&["domain", "edit", "OrderSummary", "--states", "empty,present"]).assert_exit(0);
    h.run(&["domain", "new", "flow", "checkout", "--label", "Checkout",
        "--steps", "Review", "--entry-page", "Review"]).assert_exit(0);
}

#[test]
fn tc_1027_render_contract_projects_page_graph_and_aui() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    seed_page_graph(&h);
    let out = h.run(&["preview", "render-contract", "checkout"]);
    out.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(v["contract_version"], "preview-0");
    assert_eq!(v["flow"]["entry"], "Review");
    assert_eq!(v["root"]["destinations"][0]["to"], "Review");
    let screen = &v["screens"][0];
    assert_eq!(screen["projection"], "OrderSummary");
    assert_eq!(screen["state_space"], serde_json::json!(["empty", "present"]));
    // The display-value element inherits its AIO's WCAG obligation (1.1.1).
    let disp = screen["elements"].as_array().unwrap().iter()
        .find(|e| e["role"] == "display").unwrap();
    assert_eq!(disp["aio"], "display-value");
    assert!(disp["wcag"].as_array().unwrap().iter().any(|c| c == "1.1.1"), "wcag: {}", disp["wcag"]);
    // The control issues a command and transitions are projected.
    let ctrl = screen["elements"].as_array().unwrap().iter()
        .find(|e| e["role"] == "control").unwrap();
    assert_eq!(ctrl["issues"], "PlaceOrder");
}

#[test]
fn tc_1028_render_contract_resolves_content_and_rejects_unknown_flow() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "shop", "--demo"]).assert_exit(0);
    // The demo seeds OrderSummary; reference it.
    h.run(&["domain", "new", "ui-step", "Review", "--label", "Review",
        "--surfaces", "OrderSummary:display-value", "--content", "cart.empty:empty-message"]).assert_exit(0);
    h.run(&["domain", "new", "flow", "checkout", "--label", "Checkout",
        "--steps", "Review", "--entry-page", "Review"]).assert_exit(0);
    h.run(&["domain", "new", "content-store", "cs", "--locales", "en",
        "--resolves", "cart.empty:en:Your cart is empty"]).assert_exit(0);
    let out = h.run(&["preview", "render-contract", "checkout", "--locale", "en"]);
    out.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(&out.stdout).expect("valid JSON");
    assert_eq!(v["content_store"]["cart.empty"]["value"], "Your cart is empty");
    // An unknown flow exits non-zero and names the missing flow.
    let bad = h.run(&["preview", "render-contract", "ghost"]);
    bad.assert_exit(1);
    assert!(bad.stderr.contains("ghost"), "stderr:\n{}", bad.stderr);
}


// --- What→How→Build session (phase-gated MCP workflow) ---

#[test]
fn tc_workflow_session_phases_gate_the_tool_surface() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);

    // Scaffold a session without launching an agent.
    let start = h.run(&["session", "start", "--no-launch", "bookstore"]);
    start.assert_exit(0);
    start.assert_stdout_contains("Scaffolded session");

    // The workspace is an isolated copy of `.product` and carries a journal.
    let list = h.run(&["session", "list"]);
    list.assert_exit(0);
    let id = list.stdout.split_whitespace().next().expect("session id").to_string();
    assert!(id.starts_with("bookstore-"), "list:\n{}", list.stdout);
    assert!(h.exists(&format!(".product/sessions/{id}/workflow.json")), "journal missing");
    assert!(
        h.exists(&format!(".product/sessions/{id}/ws/.product/author-domain/bookstore/bookstore.ttl")),
        "draft graph not seeded",
    );

    // Drive the phase-gated server over stdio.
    let reqs = [
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_build_run","arguments":{"deliverable":"x"}}}"#,
        r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"product_workflow_advance","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"product_workflow_advance","arguments":{}}}"#,
        r#"{"jsonrpc":"2.0","id":6,"method":"tools/list","params":{}}"#,
    ].join("\n") + "\n";
    let out = h.run_with_stdin(&["mcp", "--workflow", "--session", &id, "--repo", "."], &reqs);
    out.assert_exit(0);

    // initialize advertises dynamic tool lists.
    out.assert_stdout_contains("\"listChanged\":true");
    // What phase exposes the domain tools and the controls.
    out.assert_stdout_contains("product_domain_new");
    out.assert_stdout_contains("product_workflow_advance");
    // The Build tool is rejected in the What phase.
    out.assert_stdout_contains("belongs to the build phase");
    // Advancing fires a list_changed notification.
    out.assert_stdout_contains("notifications/tools/list_changed");
    // Once in Build, the build tool is advertised.
    out.assert_stdout_contains("product_build_run");
}

#[test]
fn tc_workflow_show_reports_phase_and_journey() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["session", "start", "--no-launch", "--until", "how", "bookstore"]).assert_exit(0);
    let id = h.run(&["session", "list"]).stdout.split_whitespace().next().unwrap().to_string();

    let show = h.run(&["session", "show", &id]);
    show.assert_exit(0);
    let v: serde_json::Value = serde_json::from_str(&show.stdout).expect("valid JSON");
    assert_eq!(v["phase"], "what");
    assert_eq!(v["until"], "how");
    assert_eq!(v["finalized"], false);
}

#[test]
fn tc_workflow_finalize_promotes_isolated_draft_to_canonical() {
    let h = Harness::new_bare();
    h.run(&["init", "--yes", "--name", "bookstore", "--demo"]).assert_exit(0);
    h.run(&["session", "start", "--no-launch", "bookstore"]).assert_exit(0);
    let id = h.run(&["session", "list"]).stdout.split_whitespace().next().unwrap().to_string();

    // Author a new context into the draft, then finalize to promote it.
    let add = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_domain_new","arguments":{"kind":"context","id":"Shipping","label":"Shipping","definition":"Delivery"}}}"#;
    h.run_with_stdin(&["mcp", "--workflow", "--session", &id, "--repo", ".", "--write"], &format!("{add}\n"))
        .assert_exit(0);

    // Isolation: the write is in the draft, not yet in the canonical graph.
    let canonical = ".product/author-domain/bookstore/bookstore.ttl";
    let draft = format!(".product/sessions/{id}/ws/.product/author-domain/bookstore/bookstore.ttl");
    assert!(h.read(&draft).contains("Shipping"), "draft should hold the new context");
    assert!(!h.read(canonical).contains("Shipping"), "canonical must stay untouched before finalize");

    // Finalize validates the draft and promotes it.
    let fin = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_session_finalize","arguments":{}}}"#;
    let out = h.run_with_stdin(&["mcp", "--workflow", "--session", &id, "--repo", ".", "--write"], &format!("{fin}\n"));
    out.assert_exit(0);
    out.assert_stdout_contains("\\\"ok\\\": true");
    assert!(h.read(canonical).contains("Shipping"), "canonical must hold the promoted context after finalize");
}

// --- §3.0/§3.0.1/§3.6 the product-boundary node family (1.6.0) -------------

#[test]
fn tc_1030_product_journey_quality_demand_and_journey_conformance() {
    let h = Harness::new(); // seeds the bookstore demo (product `bookstore`)

    // Build a minimal multi-flow slice with a Translation trigger.
    h.run(&["domain", "new", "context", "checkout", "--label", "Checkout"]).assert_exit(0);
    h.run(&["domain", "new", "entity", "order", "--context", "checkout",
        "--label", "Order", "--definition", "a customer order"]).assert_exit(0);
    h.run(&["domain", "new", "system", "sys-shop", "--label", "Shop",
        "--system-kind", "application", "--purpose", "store",
        "--references-domain", "checkout"]).assert_exit(0);
    h.run(&["domain", "new", "event", "ev-fulfilled", "--context", "checkout",
        "--label", "Fulfilled", "--changes", "order"]).assert_exit(0);
    h.run(&["domain", "new", "command", "cmd-fulfil", "--label", "Fulfil",
        "--context", "checkout", "--targets", "order", "--emits", "ev-fulfilled"]).assert_exit(0);
    h.run(&["domain", "new", "read-model", "rm-orders", "--label", "Orders",
        "--projects", "ev-fulfilled"]).assert_exit(0);
    h.run(&["domain", "new", "trigger", "tr-trans", "--label", "Translate",
        "--trigger-source", "automated", "--issues", "cmd-fulfil",
        "--watches", "rm-orders", "--translates-from", "sys-shop"]).assert_exit(0);
    h.run(&["domain", "new", "flow", "flow-a", "--label", "Shop flow", "--system", "sys-shop"]).assert_exit(0);
    h.run(&["domain", "new", "flow", "flow-b", "--label", "Admin flow", "--system", "sys-shop"]).assert_exit(0);

    // §3.0 — a product root owning the domain + system, carrying a What-version.
    h.run(&["domain", "new", "product", "acme", "--label", "Acme",
        "--purpose", "the acme product", "--owns-domain", "checkout",
        "--owns-system", "sys-shop", "--what-version", "1.0.0"]).assert_exit(0);
    let pl = h.run(&["domain", "list", "product"]);
    pl.assert_exit(0);
    assert!(pl.stdout.contains("acme"), "product should be listed:\n{}", pl.stdout);
    let show = h.run(&["domain", "show", "acme"]);
    show.assert_exit(0);
    assert!(show.stdout.contains("1.0.0") && show.stdout.contains("sys-shop"),
        "product show carries version + owned system:\n{}", show.stdout);

    // §3.6 — a runtime-bound quality demand requires a telemetry source.
    h.run(&["domain", "new", "quality-demand", "qd-bad", "--label", "X",
        "--demand-kind", "runtime-bound", "--bound", "p99<=200ms", "--scopes", "sys-shop"])
        .assert_exit(1);
    h.run(&["domain", "new", "quality-demand", "qd-lat", "--label", "Latency",
        "--demand-kind", "runtime-bound", "--bound", "p99<=200ms",
        "--scopes", "sys-shop", "--measured-by", "prod-telemetry"]).assert_exit(0);
    // An architectural constraint binds the How, not telemetry.
    h.run(&["domain", "new", "quality-demand", "qd-res", "--label", "Residency",
        "--demand-kind", "architectural", "--bound", "EU only",
        "--scopes", "sys-shop", "--constrains", "DEC-residency"]).assert_exit(0);

    // §3.0.1 — a journey crossing via a Translation passes journey conformance.
    h.run(&["domain", "new", "journey", "jrny", "--label", "Order to fulfil",
        "--owner", "acme", "--composes-flow", "flow-a,flow-b", "--crosses-via", "tr-trans"])
        .assert_exit(0);
    let strict = h.run(&["domain", "validate", "--strict"]);
    strict.assert_exit(0);
    assert!(strict.stdout.contains("conformant"), "strict validate clean:\n{}", strict.stdout);

    // A crossing that is NOT a Translation is a journey-conformance finding.
    h.run(&["domain", "new", "trigger", "tr-plain", "--label", "Plain",
        "--trigger-source", "user", "--issues", "cmd-fulfil"]).assert_exit(0);
    h.run(&["domain", "edit", "jrny", "--crosses-via", "tr-plain"]).assert_exit(0);
    let bad = h.run(&["domain", "validate", "--strict"]);
    bad.assert_exit(1);
    assert!(bad.stderr.contains("Journey conformance"),
        "non-Translation crossing fails journey conformance:\n{}", bad.stderr);
}

// --- §7.3 versions and direction (1.6.0) -----------------------------------

#[test]
fn tc_1031_how_versions_and_target_direction() {
    let h = Harness::new();
    // A node to anchor a feature on.
    h.run(&["domain", "new", "context", "Catalog", "--label", "Catalog"]).assert_exit(0);

    // §7.3 — the How carries its own version and the What-version it realises.
    h.run(&["how", "init"]).assert_exit(0);
    h.run(&["how", "set", "version", "--id", "1.3.0"]).assert_exit(0);
    h.run(&["how", "set", "realises-version", "--id", "2.0"]).assert_exit(0);
    let hs = h.run(&["how", "show"]);
    hs.assert_exit(0);
    assert!(hs.stdout.contains("1.3.0") && hs.stdout.contains("What 2.0"),
        "how show carries both versions:\n{}", hs.stdout);

    // A target version is a declared partition of features (deliverables).
    h.run(&["feature", "new", "sl-1", "--anchor", "Catalog"]).assert_exit(0);
    h.run(&["deliverable", "new", "del-1", "--feature", "sl-1", "--accept", "ac1:the catalog ships"]).assert_exit(0);
    h.run(&["target", "new", "v2", "--version", "2.0", "--feature", "del-1"]).assert_exit(0);
    let ts = h.run(&["target", "show", "v2"]);
    ts.assert_exit(0);
    assert!(ts.stdout.contains("What 2.0") && ts.stdout.contains("del-1"),
        "target show carries version + members:\n{}", ts.stdout);

    // Direction is the computed gap: an unrealised member is in the distance.
    let dir = h.run(&["target", "direction", "v2"]);
    dir.assert_exit(1); // del-1 is not done yet
    assert!(dir.stdout.contains("unrealised") && dir.stdout.contains("del-1"),
        "direction reports the unrealised feature:\n{}", dir.stdout);

    // A target naming a non-deliverable member is rejected.
    h.run(&["target", "new", "bad", "--feature", "nope"]).assert_exit(1);
}

// --- §5.1 the Build seam — verdict validation (1.6.0) ----------------------

#[test]
fn tc_1032_build_seam_verdict_validation() {
    let h = Harness::new();
    // A well-formed verdict event validates (the worked-example shape).
    h.write("verdict-good.json", r#"{
        "event_id": "ev-9f81", "emitted_at": "2026-06-26T02:14:08Z",
        "unit_ref": "wu-0007", "bundle_hash": "b3d1f2a9",
        "verdict": "accepted", "next_consequence": "advance",
        "executor_extension": { "kind": "spark", "tier_ran": "coder-medium" }
    }"#);
    let good = h.run(&["verdict", "verdict-good.json"]);
    good.assert_exit(0);
    assert!(good.stdout.contains("valid verdict") && good.stdout.contains("accepted"),
        "a conforming verdict validates:\n{}", good.stdout);

    // An off-vocabulary verdict is rejected (the §6.2 vocabulary is pinned).
    h.write("verdict-bad.json", r#"{ "event_id": "e", "emitted_at": "t", "unit_ref": "u", "bundle_hash": "h", "verdict": "maybe" }"#);
    let bad = h.run(&["verdict", "verdict-bad.json"]);
    bad.assert_exit(1);
    assert!(bad.stderr.contains("accepted") && bad.stderr.contains("escalate"),
        "the verdict vocabulary is reported:\n{}", bad.stderr);
}
