//! Session-based integration-test harness (FT-043, ADR-018 amended).
//!
//! A `Session` drives a fresh temporary repository through one or more
//! `product request apply` calls and asserts on the resulting graph state,
//! file content, and command output. Sessions build their fixtures through
//! the same interface real users and agents use — the request model.
//!
//! The canonical session library lives as one `#[test]` per scenario under
//! `tests/sessions/`. The harness is documented in
//! `docs/product-testing-spec.md` § Session Runner.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

// ---------------------------------------------------------------------------
// Output — raw command output
// ---------------------------------------------------------------------------

pub struct Output {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl Output {
    pub fn assert_exit(&self, code: i32) -> &Self {
        assert_eq!(
            self.exit_code, code,
            "expected exit code {}, got {}\nstdout: {}\nstderr: {}",
            code, self.exit_code, self.stdout, self.stderr
        );
        self
    }

    pub fn assert_stderr_contains(&self, s: &str) -> &Self {
        assert!(
            self.stderr.contains(s),
            "expected stderr to contain '{}', got:\n{}",
            s,
            self.stderr
        );
        self
    }

    pub fn assert_stdout_contains(&self, s: &str) -> &Self {
        assert!(
            self.stdout.contains(s),
            "expected stdout to contain '{}', got:\n{}",
            s,
            self.stdout
        );
        self
    }
}

// ---------------------------------------------------------------------------
// ApplyResult + associated structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct AssignedArtifact {
    pub ref_name: Option<String>,
    pub id: String,
    pub file: String,
}

#[derive(Debug, Clone)]
pub struct ChangedArtifact {
    pub id: String,
    pub mutations: usize,
    pub file: String,
}

#[derive(Debug, Clone)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub location: String,
}

#[derive(Debug, Clone)]
pub struct ApplyResult {
    pub applied: bool,
    pub created: Vec<AssignedArtifact>,
    pub changed: Vec<ChangedArtifact>,
    pub findings: Vec<Finding>,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
}

impl ApplyResult {
    pub fn assert_applied(&self) -> &Self {
        assert!(
            self.applied,
            "expected apply to succeed, but it failed.\nfindings: {:?}\nstderr: {}",
            self.findings, self.stderr
        );
        self
    }

    pub fn assert_failed(&self) -> &Self {
        assert!(
            !self.applied,
            "expected apply to fail, but it succeeded.\ncreated: {:?}\nchanged: {:?}",
            self.created, self.changed
        );
        self
    }

    pub fn assert_finding(&self, code: &str) -> &Self {
        assert!(
            self.findings.iter().any(|f| f.code == code),
            "expected finding with code '{}' — got {:?}",
            code,
            self.findings
                .iter()
                .map(|f| f.code.as_str())
                .collect::<Vec<_>>()
        );
        self
    }

    pub fn assert_no_finding(&self, code: &str) -> &Self {
        assert!(
            !self.findings.iter().any(|f| f.code == code),
            "expected no finding with code '{}' — got {:?}",
            code,
            self.findings
                .iter()
                .map(|f| f.code.as_str())
                .collect::<Vec<_>>()
        );
        self
    }

    pub fn assert_clean(&self) -> &Self {
        self.assert_applied();
        assert!(
            self.findings.is_empty(),
            "expected no findings, got: {:?}",
            self.findings
        );
        self
    }

    /// Return the ID assigned to a declared `ref:` name. Panics if the ref is
    /// not in the created array — tests should never call this with an
    /// unknown ref.
    pub fn id_for(&self, ref_name: &str) -> String {
        for c in &self.created {
            if c.ref_name.as_deref() == Some(ref_name) {
                return c.id.clone();
            }
        }
        panic!(
            "ref name '{}' not found in created artifacts: {:?}",
            ref_name,
            self.created
                .iter()
                .map(|c| (c.ref_name.clone(), c.id.clone()))
                .collect::<Vec<_>>()
        );
    }
}

// ---------------------------------------------------------------------------
// Session — the main harness type
// ---------------------------------------------------------------------------

pub struct Session {
    pub dir: tempfile::TempDir,
    pub bin: PathBuf,
    step: usize,
}

impl Default for Session {
    fn default() -> Self {
        Self::new()
    }
}

impl Session {
    /// Create a fresh session with a pre-initialised temp repository.
    /// The repository has a valid `product.toml`, a minimal domain vocabulary,
    /// and empty `docs/features/`, `docs/adrs/`, `docs/tests/`,
    /// `docs/dependencies/` directories.
    pub fn new() -> Self {
        let dir = tempfile::tempdir().expect("tempdir");
        let bin = find_binary();

        let config = r#"name = "session-test"
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
[mcp]
write = true
[domains]
api = "CLI surface, MCP tools"
security = "Authentication, authorisation, secrets"
networking = "mDNS, mTLS, DNS"
error-handling = "Error model, diagnostics"
storage = "Persistence, durability"
consensus = "Raft, leader election"
"#;
        std::fs::write(dir.path().join("product.toml"), config).expect("write product.toml");
        for sub in [
            "docs/features",
            "docs/adrs",
            "docs/tests",
            "docs/dependencies",
            "docs/graph",
        ] {
            std::fs::create_dir_all(dir.path().join(sub)).expect("mkdir");
        }

        Self { dir, bin, step: 0 }
    }

    /// Write an arbitrary file (relative to the session root).
    pub fn write(&self, path: &str, content: &str) -> &Self {
        let full = self.dir.path().join(path);
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent).expect("mkdir");
        }
        std::fs::write(&full, content).expect("write");
        self
    }

    /// Read a file (relative to the session root). Returns empty string if
    /// the file does not exist.
    pub fn read(&self, path: &str) -> String {
        std::fs::read_to_string(self.dir.path().join(path)).unwrap_or_default()
    }

    /// Run the compiled `product` binary in the session directory.
    pub fn run(&self, args: &[&str]) -> Output {
        let output = Command::new(&self.bin)
            .args(args)
            .current_dir(self.dir.path())
            .stdin(Stdio::null())
            .output()
            .expect("spawn product");
        Output {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }

    /// Apply an inline YAML request.
    pub fn apply(&mut self, request_yaml: &str) -> ApplyResult {
        self.step += 1;
        let filename = format!(".session-step-{:03}.yaml", self.step);
        self.write(&filename, request_yaml);
        self.apply_file(&filename)
    }

    /// Apply a request from a file path (relative to the session root).
    pub fn apply_file(&mut self, path: &str) -> ApplyResult {
        let out = self.run(&["--format", "json", "request", "apply", path]);
        parse_apply_json(&out)
    }

    /// Validate an inline YAML request (writes no files, returns findings).
    pub fn validate(&mut self, request_yaml: &str) -> ApplyResult {
        self.step += 1;
        let filename = format!(".session-step-{:03}.yaml", self.step);
        self.write(&filename, request_yaml);
        let out = self.run(&["--format", "json", "request", "validate", &filename]);
        parse_apply_json(&out)
    }

    // -----------------------------------------------------------------------
    // Assertions
    // -----------------------------------------------------------------------

    pub fn assert_file_exists(&self, path: &str) -> &Self {
        assert!(
            self.dir.path().join(path).exists(),
            "expected file to exist: {}",
            path
        );
        self
    }

    pub fn assert_file_missing(&self, path: &str) -> &Self {
        assert!(
            !self.dir.path().join(path).exists(),
            "expected file to be absent: {}",
            path
        );
        self
    }

    /// Assert a scalar front-matter field has exactly the given value.
    pub fn assert_frontmatter(&self, path: &str, field: &str, value: &str) -> &Self {
        let body = self.read(path);
        let fm = extract_frontmatter(&body)
            .unwrap_or_else(|| panic!("no front-matter block in {}", path));
        let doc: serde_yaml::Value = serde_yaml::from_str(fm)
            .unwrap_or_else(|e| panic!("front-matter parse error in {}: {}", path, e));
        let actual = doc
            .get(field)
            .unwrap_or_else(|| panic!("front-matter of {} has no field '{}'", path, field));
        let actual_str = match actual {
            serde_yaml::Value::String(s) => s.clone(),
            serde_yaml::Value::Number(n) => n.to_string(),
            serde_yaml::Value::Bool(b) => b.to_string(),
            other => {
                panic!(
                    "expected scalar for {}.{}; got: {:?}",
                    path, field, other
                )
            }
        };
        assert_eq!(
            actual_str.as_str(),
            value,
            "front-matter {}.{} expected '{}', got '{}'",
            path,
            field,
            value,
            actual_str
        );
        self
    }

    /// Assert that a sequence field in front-matter contains the given value.
    pub fn assert_array_contains(&self, path: &str, field: &str, value: &str) -> &Self {
        let body = self.read(path);
        let fm = extract_frontmatter(&body)
            .unwrap_or_else(|| panic!("no front-matter block in {}", path));
        let doc: serde_yaml::Value = serde_yaml::from_str(fm)
            .unwrap_or_else(|e| panic!("front-matter parse error in {}: {}", path, e));
        let arr = doc
            .get(field)
            .unwrap_or_else(|| panic!("front-matter of {} has no field '{}'", path, field));
        let seq = arr.as_sequence().unwrap_or_else(|| {
            panic!("field {}.{} is not a sequence: {:?}", path, field, arr)
        });
        let found = seq.iter().any(|v| v.as_str() == Some(value));
        assert!(
            found,
            "expected {}.{} to contain '{}'; got: {:?}",
            path, field, value, seq
        );
        self
    }

    /// Assert that a sequence field in front-matter does NOT contain the given value.
    pub fn assert_array_missing(&self, path: &str, field: &str, value: &str) -> &Self {
        let body = self.read(path);
        let fm = extract_frontmatter(&body)
            .unwrap_or_else(|| panic!("no front-matter block in {}", path));
        let doc: serde_yaml::Value = serde_yaml::from_str(fm)
            .unwrap_or_else(|e| panic!("front-matter parse error in {}: {}", path, e));
        if let Some(arr) = doc.get(field) {
            if let Some(seq) = arr.as_sequence() {
                assert!(
                    !seq.iter().any(|v| v.as_str() == Some(value)),
                    "expected {}.{} to not contain '{}'; got: {:?}",
                    path, field, value, seq
                );
            }
        }
        self
    }

    /// Run `product graph check` and assert the exit code is 0 or 2 (no errors).
    pub fn assert_graph_clean(&self) -> &Self {
        let out = self.run(&["graph", "check"]);
        assert!(
            out.exit_code == 0 || out.exit_code == 2,
            "expected graph check to be clean (exit 0 or 2), got {}\nstderr: {}",
            out.exit_code,
            out.stderr
        );
        self
    }

    /// Run `product graph check` and assert the specified E-code appears.
    pub fn assert_graph_error(&self, code: &str) -> &Self {
        let out = self.run(&["graph", "check"]);
        assert!(
            out.exit_code == 1,
            "expected graph check exit 1, got {}\nstderr: {}",
            out.exit_code,
            out.stderr
        );
        assert!(
            out.stderr.contains(code),
            "expected graph check stderr to contain '{}', got:\n{}",
            code,
            out.stderr
        );
        self
    }

    /// Run `product graph check` and assert the specified W-code appears.
    pub fn assert_graph_warning(&self, code: &str) -> &Self {
        let out = self.run(&["graph", "check"]);
        assert!(
            out.stderr.contains(code),
            "expected graph check stderr to contain '{}', got:\n{}",
            code,
            out.stderr
        );
        self
    }

    /// Compute SHA-256 digests of every file under `docs/`, keyed by relative
    /// path. Used to verify zero-files-changed invariants.
    pub fn docs_digest(&self) -> HashMap<String, String> {
        let mut out = HashMap::new();
        let docs = self.dir.path().join("docs");
        if docs.exists() {
            walk_dir(&docs, &docs, &mut out);
        }
        out
    }

    pub fn root(&self) -> &Path {
        self.dir.path()
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn find_binary() -> PathBuf {
    // Prefer CARGO_BIN_EXE_product (set when building tests under this crate).
    if let Some(bin) = option_env!("CARGO_BIN_EXE_product") {
        let p = PathBuf::from(bin);
        if p.exists() {
            return p;
        }
    }
    // Fall back: walk up from the test executable path to target/debug/product.
    if let Ok(exe) = std::env::current_exe() {
        let mut p = exe.clone();
        p.pop(); // remove test binary name
        p.pop(); // remove deps/
        p.push("product");
        if p.exists() {
            return p;
        }
    }
    // Final fallback: assume cwd is the project root.
    PathBuf::from("target/debug/product")
}

fn extract_frontmatter(body: &str) -> Option<&str> {
    let trimmed = body.trim_start();
    let after_first = trimmed.strip_prefix("---\n")?;
    let end = after_first.find("\n---")?;
    Some(&after_first[..end])
}

fn parse_apply_json(out: &Output) -> ApplyResult {
    // Strip any prefix warnings before the JSON object begins.
    let start = out.stdout.find('{').unwrap_or(0);
    let slice = &out.stdout[start..];
    let v: serde_json::Value = serde_json::from_str(slice)
        .unwrap_or_else(|e| panic!(
            "failed to parse apply JSON (exit={} err={})\nstdout:\n{}\nstderr:\n{}",
            out.exit_code, e, out.stdout, out.stderr
        ));

    let applied = v.get("applied").and_then(|x| x.as_bool()).unwrap_or(false);
    let created: Vec<AssignedArtifact> = v
        .get("created")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| AssignedArtifact {
                    ref_name: c
                        .get("ref_name")
                        .and_then(|s| s.as_str())
                        .map(str::to_string),
                    id: c
                        .get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    file: c
                        .get("file")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let changed: Vec<ChangedArtifact> = v
        .get("changed")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .map(|c| ChangedArtifact {
                    id: c
                        .get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    mutations: c
                        .get("mutations")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(0) as usize,
                    file: c
                        .get("file")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();
    let findings: Vec<Finding> = v
        .get("findings")
        .and_then(|x| x.as_array())
        .map(|arr| {
            arr.iter()
                .map(|f| Finding {
                    code: f
                        .get("code")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    severity: f
                        .get("severity")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    message: f
                        .get("message")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                    location: f
                        .get("location")
                        .and_then(|s| s.as_str())
                        .unwrap_or_default()
                        .to_string(),
                })
                .collect()
        })
        .unwrap_or_default();

    ApplyResult {
        applied,
        created,
        changed,
        findings,
        stdout: out.stdout.clone(),
        stderr: out.stderr.clone(),
        exit_code: out.exit_code,
    }
}

fn walk_dir(base: &Path, dir: &Path, out: &mut HashMap<String, String>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let p = entry.path();
        if p.is_dir() {
            walk_dir(base, &p, out);
        } else if p.is_file() {
            if let Ok(rel) = p.strip_prefix(base) {
                let rel_s = rel.to_string_lossy().to_string();
                let bytes = std::fs::read(&p).unwrap_or_default();
                let digest = sha256_hex(&bytes);
                out.insert(rel_s, digest);
            }
        }
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut s = String::with_capacity(64);
    for b in out.iter() {
        s.push_str(&format!("{:02x}", b));
    }
    s
}
