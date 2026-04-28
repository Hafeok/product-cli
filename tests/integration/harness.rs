//! Integration test harness: temp-dir Harness, Output assertions, fixtures.
//! Used by every topic file under `tests/integration/`.

#![allow(clippy::unwrap_used)]
#![allow(dead_code)]

pub use std::path::{Path, PathBuf};
pub use std::process::{Command, Stdio};

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

pub fn fixture_minimal() -> Harness {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ncontent-hash: sha256:041d699c4fbf6ed027d18d01345d5dbc758c222150d9ae85257d83e98ccf3ede\n---\n\nDecision body.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n");
    h
}

pub fn fixture_broken_link() -> Harness {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-999]\ntests: []\n---\n\nBroken.\n");
    h
}

pub fn fixture_dep_cycle() -> Harness {
    let h = Harness::new();
    h.write("docs/features/FT-001-a.md", "---\nid: FT-001\ntitle: A\nphase: 1\nstatus: planned\ndepends-on: [FT-002]\nadrs: []\ntests: []\n---\n");
    h.write("docs/features/FT-002-b.md", "---\nid: FT-002\ntitle: B\nphase: 1\nstatus: planned\ndepends-on: [FT-001]\nadrs: []\ntests: []\n---\n");
    h
}

pub fn fixture_orphaned_adr() -> Harness {
    let h = Harness::new();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n");
    h.write("docs/adrs/ADR-001-orphan.md", "---\nid: ADR-001\ntitle: Orphan\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n");
    h
}

pub fn fixture_error_and_warning() -> Harness {
    let h = Harness::new();
    // Feature references non-existent ADR-999 → 1 error (E002)
    // Also links to existing TC-001 with exit-criteria type → no W002/W003
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-999]\ntests: [TC-001]\n---\n",
    );
    // Orphaned ADR (not linked from any feature) → 1 warning (W001)
    h.write(
        "docs/adrs/ADR-001-orphan.md",
        "---\nid: ADR-001\ntitle: Orphan\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ncontent-hash: sha256:86de87e1ad0426749f8302ae1e203fe3f8c3453a8619a4187faf78583f23c433\n---\n",
    );
    // TC linked from FT-001 with exit-criteria type
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n",
    );
    h
}

pub const MINIMAL_CONFIG: &str = "name = \"test\"\nschema-version = \"1\"\n[paths]\nfeatures = \"docs/features\"\nadrs = \"docs/adrs\"\ntests = \"docs/tests\"\ngraph = \"docs/graph\"\nchecklist = \"docs/checklist.md\"\n[prefixes]\nfeature = \"FT\"\nadr = \"ADR\"\ntest = \"TC\"";

pub fn start_mcp_http(h: &Harness, port: u16, extra_args: &[&str]) -> std::process::Child {
    use std::process::{Command, Stdio};

    let mut cmd = Command::new(&h.bin);
    cmd.args(["mcp", "--http", "--port", &port.to_string(), "--bind", "127.0.0.1"])
        .args(extra_args)
        .current_dir(h.dir.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let child = cmd.spawn().expect("spawn mcp http");

    // Wait for server to be ready by polling the port
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).is_ok() {
            return child;
        }
    }
    child
}

pub fn http_post(port: u16, body: &str, auth_header: Option<&str>) -> (String, String, String) {
    use std::io::{Read, Write};

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("connect to mcp http");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();

    let mut request = format!(
        "POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n",
        port, body.len()
    );
    if let Some(auth) = auth_header {
        request.push_str(&format!("Authorization: {}\r\n", auth));
    }
    request.push_str("Connection: close\r\n\r\n");
    request.push_str(body);

    stream.write_all(request.as_bytes()).expect("write request");
    stream.flush().expect("flush");

    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);

    // Parse status line, headers, body
    let parts: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();
    let header_section = parts.first().unwrap_or(&"");
    let body_section = parts.get(1).unwrap_or(&"").to_string();
    let mut lines = header_section.lines();
    let status_line = lines.next().unwrap_or("").to_string();
    let headers: String = lines.collect::<Vec<_>>().join("\n");

    (status_line, headers, body_section)
}

pub fn http_options(port: u16, origin: &str) -> (String, String, String) {
    use std::io::{Read, Write};

    let mut stream = std::net::TcpStream::connect(format!("127.0.0.1:{}", port))
        .expect("connect to mcp http");
    stream.set_read_timeout(Some(std::time::Duration::from_secs(10))).ok();

    let request = format!(
        "OPTIONS /mcp HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nOrigin: {}\r\nAccess-Control-Request-Method: POST\r\nAccess-Control-Request-Headers: authorization,content-type\r\nConnection: close\r\n\r\n",
        port, origin
    );

    stream.write_all(request.as_bytes()).expect("write request");
    stream.flush().expect("flush");

    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);

    let parts: Vec<&str> = response.splitn(2, "\r\n\r\n").collect();
    let header_section = parts.first().unwrap_or(&"");
    let body_section = parts.get(1).unwrap_or(&"").to_string();
    let mut lines = header_section.lines();
    let status_line = lines.next().unwrap_or("").to_string();
    let headers: String = lines.collect::<Vec<_>>().join("\n");

    (status_line, headers, body_section)
}

pub fn unique_port() -> u16 {
    use std::sync::atomic::{AtomicU16, Ordering};
    static PORT: AtomicU16 = AtomicU16::new(17700);
    PORT.fetch_add(1, Ordering::SeqCst)
}

pub fn run_mcp_stdio(h: &Harness, input: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(&h.bin)
        .args(["mcp"])
        .current_dir(h.dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp");

    if let Some(ref mut stdin) = child.stdin {
        let _ = writeln!(stdin, "{}", input);
    }
    // Close stdin to signal EOF
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn fixture_checklist_three_features() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-alpha.md",
        "---\nid: FT-001\ntitle: Alpha Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nAlpha body.\n",
    );
    h.write(
        "docs/features/FT-002-beta.md",
        "---\nid: FT-002\ntitle: Beta Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-002]\n---\n\nBeta body.\n",
    );
    h.write(
        "docs/features/FT-003-gamma.md",
        "---\nid: FT-003\ntitle: Gamma Feature\nphase: 2\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nGamma body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test Decision\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-alpha-test.md",
        "---\nid: TC-001\ntitle: Alpha Test\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-beta-test.md",
        "---\nid: TC-002\ntitle: Beta Test\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-002]\n  adrs: []\nphase: 1\n---\n\nTest body.\n",
    );
    h
}

pub const CONFIG_WITH_DOMAINS: &str = r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
[domains]
security = "Auth, authz, secrets, trust boundaries"
storage = "Persistence, durability, volumes"
networking = "mDNS, mTLS, DNS, service discovery"
error-handling = "Error model, diagnostics, exit codes"
[features]
required-sections = []
functional-spec-subsections = []
"#;

pub fn harness_with_domains() -> Harness {
    let h = Harness::new();
    h.write("product.toml", CONFIG_WITH_DOMAINS);
    h
}

pub fn fixture_abandon() -> Harness {
    let h = Harness::new();
    h.write("docs/features/FT-001-test-feature.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: [TC-001, TC-002]\ndomains: []\ndomains-acknowledged: {}\n---\n\nFeature body.\n");
    h.write("docs/tests/TC-001-test-one.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest one.\n");
    h.write("docs/tests/TC-002-test-two.md",
        "---\nid: TC-002\ntitle: Test Two\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nTest two.\n");
    h
}

pub fn fixture_gap_g001() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Decision:** Use caching.\n\n## Test coverage\n\nPerformance under load must stay below 200ms.\n\n**Rejected alternatives:**\n- No caching\n",
    );
    h
}

pub fn fixture_gap_clean() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Decision:** Use caching.\n\n**Rejected alternatives:**\n- No caching\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n",
    );
    h
}

pub fn git_init(h: &Harness) {
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(h.dir.path())
        .output()
        .expect("git init");
    std::process::Command::new("git")
        .args(["config", "user.email", "test@test.com"])
        .current_dir(h.dir.path())
        .output()
        .expect("git config email");
    std::process::Command::new("git")
        .args(["config", "user.name", "Test"])
        .current_dir(h.dir.path())
        .output()
        .expect("git config name");
    // Disable commit signing so tests work in CI and environments with signing configured
    std::process::Command::new("git")
        .args(["config", "commit.gpgsign", "false"])
        .current_dir(h.dir.path())
        .output()
        .expect("git config gpgsign");
}

pub fn fixture_implement_gap() -> Harness {
    let h = Harness::new();
    // Feature with ADR that has a testable claim but no linked TC → triggers G001
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Decision:** Use caching.\n\n## Test coverage\n\nPerformance under load must stay below 200ms.\n\n**Rejected alternatives:**\n- No caching\n",
    );
    h
}

pub fn fixture_verify_passing() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Test Two\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass2.sh\n---\n\nTest body.\n",
    );
    // Passing test scripts
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("pass2.sh", "#!/bin/bash\nexit 0\n");
    // Make scripts executable
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "pass2.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");
    h
}

pub fn compute_adr_content_hash(title: &str, body: &str) -> String {
    use sha2::{Digest, Sha256};
    let normalized = body.replace("\r\n", "\n").trim().to_string();
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(b"\n");
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

pub fn compute_tc_content_hash(title: &str, test_type: &str, adrs: &[&str], body: &str) -> String {
    use sha2::{Digest, Sha256};
    let normalized = body.replace("\r\n", "\n").trim().to_string();
    let mut sorted_adrs: Vec<&str> = adrs.to_vec();
    sorted_adrs.sort();
    let mut hasher = Sha256::new();
    hasher.update(title.as_bytes());
    hasher.update(b"\n");
    hasher.update(test_type.as_bytes());
    hasher.update(b"\n");
    hasher.update(sorted_adrs.join(",").as_bytes());
    hasher.update(b"\n");
    hasher.update(normalized.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{:x}", result)
}

pub fn run_mcp_stdio_write(h: &Harness, input: &str) -> String {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new(&h.bin)
        .args(["mcp", "--write"])
        .current_dir(h.dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mcp");

    if let Some(ref mut stdin) = child.stdin {
        let _ = writeln!(stdin, "{}", input);
    }
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("wait");
    String::from_utf8_lossy(&output.stdout).to_string()
}

pub fn fixture_dep_library() -> Harness {
    let h = Harness::new();
    h.write("docs/adrs/ADR-002-openraft.md", "---\nid: ADR-002\ntitle: openraft\nstatus: accepted\nfeatures: [FT-001]\n---\n\nRationale.\n");
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nFeature body.\n");
    h.write("docs/dependencies/DEP-001-openraft.md", "---\nid: DEP-001\ntitle: openraft\ntype: library\nsource: crates.io\nversion: \">=0.9,<1.0\"\nstatus: active\nfeatures: [FT-001]\nadrs: [ADR-002]\navailability-check: ~\nbreaking-change-risk: medium\n---\n\nRaft consensus library.\n");
    h
}

pub fn fixture_dep_service() -> Harness {
    let h = fixture_dep_library();
    h.write("docs/features/FT-007-events.md", "---\nid: FT-007\ntitle: Event Store\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nEvent store.\n");
    h.write("docs/adrs/ADR-015-postgres.md", "---\nid: ADR-015\ntitle: PostgreSQL\nstatus: accepted\nfeatures: [FT-007]\n---\n\nDecision.\n");
    h.write("docs/dependencies/DEP-005-postgresql.md", "---\nid: DEP-005\ntitle: PostgreSQL Event Store\ntype: service\nversion: \">=14\"\nstatus: active\nfeatures: [FT-007]\nadrs: [ADR-015]\navailability-check: \"true\"\nbreaking-change-risk: low\ninterface:\n  protocol: tcp\n  port: 5432\n  auth: md5\n  connection-string-env: DATABASE_URL\n---\n\nPostgreSQL for events.\n");
    h
}

pub fn fixture_agent_context() -> Harness {
    let h = Harness::new();
    // Add domains to product.toml
    h.write("product.toml", r#"name = "test"
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
[domains]
security = "Authentication and authorization"
storage = "Data persistence"
networking = "Network protocols"
"#);
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n");
    h.write("docs/features/FT-002-complete.md", "---\nid: FT-002\ntitle: Complete Feature\nphase: 1\nstatus: complete\ndepends-on: []\nadrs: []\ntests: [TC-002]\n---\n\nComplete.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\nDecision body.\n");
    h.write("docs/adrs/ADR-002-proposed.md", "---\nid: ADR-002\ntitle: Proposed ADR\nstatus: proposed\nfeatures: []\n---\n\nProposed.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: exit-criteria\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n");
    h.write("docs/tests/TC-002-failing.md", "---\nid: TC-002\ntitle: Failing TC\ntype: scenario\nstatus: failing\nvalidates:\n  features: [FT-002]\nphase: 1\n---\n\nFailing test.\n");
    h.write("docs/tests/TC-003-unimpl.md", "---\nid: TC-003\ntitle: Unimplemented TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\nphase: 1\n---\n\nUnimplemented.\n");
    h
}

pub fn fixture_lifecycle_gate_proposed() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");
    h
}

pub fn git_init_with_commit(h: &Harness) {
    git_init(h);
    let dir = h.dir.path();
    std::process::Command::new("git").args(["add", "-A"]).current_dir(dir)
        .stdout(Stdio::null()).stderr(Stdio::null()).output().expect("git add");
    std::process::Command::new("git").args(["commit", "-m", "initial commit"])
        .current_dir(dir).stdout(Stdio::null()).stderr(Stdio::null()).output().expect("git commit");
}

pub fn git_add_commit(h: &Harness, msg: &str) {
    let dir = h.dir.path();
    std::process::Command::new("git").args(["add", "-A"]).current_dir(dir)
        .stdout(Stdio::null()).stderr(Stdio::null()).output().expect("git add");
    std::process::Command::new("git").args(["commit", "-m", msg, "--allow-empty"])
        .current_dir(dir).stdout(Stdio::null()).stderr(Stdio::null()).output().expect("git commit");
}

pub fn fixture_verify_with_git() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test Feature\nphase: 1\nstatus: in-progress\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001, TC-002]\n---\n\nFeature body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-test.md",
        "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Rejected alternatives:**\n- None\n",
    );
    h.write(
        "docs/tests/TC-001-test.md",
        "---\nid: TC-001\ntitle: Test One\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass.sh\n---\n\nTest body.\n",
    );
    h.write(
        "docs/tests/TC-002-test.md",
        "---\nid: TC-002\ntitle: Test Two\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\nrunner: bash\nrunner-args: pass2.sh\n---\n\nTest body.\n",
    );
    h.write("pass.sh", "#!/bin/bash\nexit 0\n");
    h.write("pass2.sh", "#!/bin/bash\nexit 0\n");
    std::process::Command::new("chmod")
        .args(["+x", "pass.sh", "pass2.sh"])
        .current_dir(h.dir.path())
        .output()
        .expect("chmod");
    git_init_with_commit(&h);
    h
}

pub fn fixture_with_responsibility() -> Harness {
    let h = Harness::new();
    h.write("product.toml", r#"name = "test"
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
[product]
name = "picloud"
responsibility = "A private cloud platform for Raspberry Pi clusters"
"#);
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Cluster Node Discovery\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nNode discovery for Raspberry Pi clusters.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: accepted\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\ncontent-hash: sha256:041d699c4fbf6ed027d18d01345d5dbc758c222150d9ae85257d83e98ccf3ede\n---\n\nDecision body.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: exit-criteria\nstatus: unimplemented\nvalidates:\n  features: [FT-001]\n  adrs: [ADR-001]\nphase: 1\n---\n\nTest body.\n");
    h
}

pub fn fixture_with_domains() -> Harness {
    let h = Harness::new();
    h.write("product.toml", r#"name = "test"
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
[domains]
api = "CLI surface, MCP tools"
security = "Authentication, authorisation, secrets"
networking = "mDNS, mTLS, DNS"
error-handling = "Error model, diagnostics"
storage = "Persistence, durability"
[mcp]
write = true
[verify.prerequisites]
build = "cargo build --quiet"
lint = "cargo clippy --quiet"
"#);
    h
}

pub fn fixture_bundle_summary() -> Harness {
    let h = Harness::new();
    h.write(
        "docs/features/FT-001-alpha.md",
        "---\nid: FT-001\ntitle: Alpha\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-001]\n---\n\nAlpha body.\n",
    );
    h.write(
        "docs/features/FT-002-beta.md",
        "---\nid: FT-002\ntitle: Beta\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-002]\n---\n\nBeta body.\n",
    );
    h.write(
        "docs/features/FT-003-gamma.md",
        "---\nid: FT-003\ntitle: Gamma\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: [TC-003]\n---\n\nGamma body.\n",
    );
    h.write(
        "docs/adrs/ADR-001-shared.md",
        "---\nid: ADR-001\ntitle: Shared ADR\nstatus: accepted\nfeatures: [FT-001, FT-002, FT-003]\n---\n\nADR body.\n",
    );
    h.write(
        "docs/tests/TC-001-a.md",
        "---\nid: TC-001\ntitle: T1\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-001]\n  adrs: []\nphase: 1\n---\n\nt1.\n",
    );
    h.write(
        "docs/tests/TC-002-b.md",
        "---\nid: TC-002\ntitle: T2\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-002]\n  adrs: []\nphase: 1\n---\n\nt2.\n",
    );
    h.write(
        "docs/tests/TC-003-c.md",
        "---\nid: TC-003\ntitle: T3\ntype: scenario\nstatus: passing\nvalidates:\n  features: [FT-003]\n  adrs: []\nphase: 1\n---\n\nt3.\n",
    );
    h
}

pub fn extract_tokens_approx(content: &str) -> usize {
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("tokens-approx:") {
            return rest.trim().parse().unwrap_or(0);
        }
    }
    0
}

pub fn fixture_request() -> Harness {
    let h = fixture_with_domains();
    // Seed feature + ADR for change-test scenarios.
    h.write(
        "docs/features/FT-001-seed.md",
        "---\nid: FT-001\ntitle: Seed Feature\nphase: 1\nstatus: planned\ndepends-on: []\nadrs:\n- ADR-001\ntests: []\ndomains:\n- api\ndomains-acknowledged: {}\n---\n\n## Description\n\nSeed.\n",
    );
    h.write(
        "docs/adrs/ADR-001-seed.md",
        "---\nid: ADR-001\ntitle: Seed ADR\nstatus: proposed\nfeatures:\n- FT-001\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: feature-specific\n---\n\n## Context\n\nSeed.\n",
    );
    h
}

pub fn write_req(h: &Harness, name: &str, body: &str) -> String {
    h.write(name, body);
    name.to_string()
}

pub fn fixture_log() -> Harness {
    fixture_with_domains()
}

pub fn log_lines(h: &Harness) -> Vec<String> {
    let content = h.read("requests.jsonl");
    content.lines().filter(|l| !l.is_empty()).map(String::from).collect()
}

pub fn log_line_json(h: &Harness, idx: usize) -> serde_json::Value {
    let lines = log_lines(h);
    serde_json::from_str(&lines[idx]).expect("valid json")
}

pub fn write_log_req(h: &Harness, name: &str, reason: &str, title: &str) -> String {
    let body = format!(
        "type: create\nschema-version: 1\nreason: \"{}\"\nartifacts:\n  - type: feature\n    title: {}\n    phase: 1\n    domains: [api]\n",
        reason, title
    );
    h.write(name, &body);
    name.to_string()
}

pub fn walkdir(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for e in entries.flatten() {
            let p = e.path();
            if p.is_dir() {
                out.extend(walkdir(&p));
            } else {
                out.push(p);
            }
        }
    }
    out
}

pub fn ft048_tc_types(custom: &[&str]) -> Harness {
    let h = Harness::new();
    let mut toml = std::fs::read_to_string(h.dir.path().join("product.toml"))
        .expect("read product.toml");
    toml.push_str("\n[tc-types]\ncustom = [");
    toml.push_str(
        &custom
            .iter()
            .map(|c| format!("\"{}\"", c))
            .collect::<Vec<_>>()
            .join(", "),
    );
    toml.push_str("]\n");
    std::fs::write(h.dir.path().join("product.toml"), toml).expect("write toml");
    h
}

pub fn ft048_write_feature(h: &Harness, id: &str, phase: u32, tests: &[&str]) {
    let tests_inline = format!("[{}]", tests.join(", "));
    let content = format!(
        "---\nid: {}\ntitle: Feature {}\nphase: {}\nstatus: planned\ndepends-on: []\nadrs: []\ntests: {}\n---\n\nBody\n",
        id, id, phase, tests_inline
    );
    h.write(&format!("docs/features/{}.md", id), &content);
}

pub fn ft048_write_tc(h: &Harness, id: &str, title: &str, tc_type: &str, status: &str, feature: &str, phase: u32) {
    let content = format!(
        "---\nid: {}\ntitle: {}\ntype: {}\nstatus: {}\nvalidates:\n  features: [{}]\n  adrs: []\nphase: {}\n---\n\nBody\n",
        id, title, tc_type, status, feature, phase
    );
    h.write(&format!("docs/tests/{}.md", id), &content);
}

pub fn collect_file_values_from_json(v: &serde_json::Value, out: &mut Vec<String>) {
    match v {
        serde_json::Value::Object(map) => {
            for (k, inner) in map.iter() {
                if k == "file" {
                    if let Some(s) = inner.as_str() {
                        out.push(s.to_string());
                        continue;
                    }
                }
                collect_file_values_from_json(inner, out);
            }
        }
        serde_json::Value::Array(arr) => {
            for x in arr {
                collect_file_values_from_json(x, out);
            }
        }
        _ => {}
    }
}

pub fn fixture_planning(date_line: Option<&str>) -> Harness {
    let h = fixture_with_domains();
    let dd = date_line.map(|d| format!("due-date: \"{}\"\n", d)).unwrap_or_default();
    h.write(
        "docs/features/FT-009-payments.md",
        &format!(
            "---\nid: FT-009\ntitle: Payments\nphase: 1\nstatus: in-progress\n{}depends-on: []\nadrs:\n- ADR-045\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\n## Description\n\nSeed.\n",
            dd
        ),
    );
    h.write(
        "docs/adrs/ADR-045-planning.md",
        "---\nid: ADR-045\ntitle: Planning ADR\nstatus: accepted\nfeatures:\n- FT-009\nsupersedes: []\nsuperseded-by: []\ndomains:\n- api\nscope: cross-cutting\n---\n\n## Context\n\nSeed.\n",
    );
    h
}

pub fn ct_write_feature(h: &Harness, id: &str, status: &str) {
    let fname = format!("docs/features/{}-{}.md", id, id.to_lowercase());
    let content = format!(
        "---\nid: {}\ntitle: {}\nphase: 1\nstatus: {}\ndepends-on: []\nadrs: []\ntests: []\ndomains:\n- api\ndomains-acknowledged: {{}}\n---\n\nSeed.\n",
        id, id, status
    );
    h.write(&fname, &content);
}

pub fn ct_tag_at(h: &Harness, id: &str, event: &str, iso_ts: &str) {
    let tag = format!("product/{}/{}", id, event);
    let msg = format!("{} {}", id, event);
    std::process::Command::new("git")
        .args(["tag", "-a", &tag, "-m", &msg])
        .env("GIT_COMMITTER_DATE", iso_ts)
        .env("GIT_AUTHOR_DATE", iso_ts)
        .current_dir(h.dir.path())
        .output()
        .expect("git tag");
}

pub fn ct_fixture(entries: &[(&str, &str, Option<&str>, Option<&str>)]) -> Harness {
    let h = fixture_with_domains();
    git_init(&h);
    for (id, status, _s, _c) in entries {
        ct_write_feature(&h, id, status);
    }
    std::process::Command::new("git")
        .args(["add", "-A"])
        .current_dir(h.dir.path())
        .output()
        .expect("git add");
    std::process::Command::new("git")
        .args(["commit", "-m", "seed"])
        .current_dir(h.dir.path())
        .output()
        .expect("git commit");

    for (id, _status, s, c) in entries {
        if let Some(st) = s {
            ct_tag_at(&h, id, "started", st);
        }
        if let Some(cp) = c {
            ct_tag_at(&h, id, "complete", cp);
        }
    }
    h
}

pub const CONFIG_W030_DEFAULT: &str = r#"name = "test"
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
"#;

pub const COMPLETE_BODY: &str = "## Description\n\nProse describing the feature.\n\n## Functional Specification\n\n### Inputs\n\n- foo\n\n### Outputs\n\n- bar\n\n### State\n\nStateless.\n\n### Behaviour\n\n1. Do thing.\n\n### Invariants\n\n- always holds.\n\n### Error handling\n\nReturn error.\n\n### Boundaries\n\n- edge case.\n\n## Out of scope\n\n- nothing.\n";

