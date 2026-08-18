//! Support for the registry-generator fixtures: parameters, targets, teardown.

#![allow(dead_code, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

/// The `product` binary the fixtures drive.
pub(crate) fn bin() -> PathBuf {
    let mut path = std::env::current_exe().expect("current_exe");
    path.pop();
    path.pop();
    path.push("product");
    if !path.exists() {
        path = PathBuf::from("target/debug/product");
    }
    path
}

/// The workspace root, for reading the template's own files.
pub(crate) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

/// Read a file relative to the workspace root.
pub(crate) fn read(rel: &str) -> String {
    std::fs::read_to_string(workspace_root().join(rel)).expect("read workspace file")
}

/// What a command run produced.
pub(crate) struct Run {
    pub(crate) stdout: String,
    pub(crate) stderr: String,
    pub(crate) exit_code: i32,
}

/// One generation's parameters.
#[derive(Clone)]
pub(crate) struct Args {
    pub(crate) owner: String,
    pub(crate) repo: String,
    pub(crate) ratifier: String,
    pub(crate) display_name: String,
    pub(crate) base_iri: String,
    pub(crate) date: String,
    pub(crate) generated_by: String,
}

/// The parameters the G0 handoff pins — the same five, the same mint date.
pub(crate) fn g0_args() -> Args {
    Args {
        owner: "Hafeok".into(),
        repo: "ground-registry-g0".into(),
        ratifier: "emil@okkels-klein.dk".into(),
        display_name: "Ground Registry (G0 validation)".into(),
        base_iri: "tag:emil@okkels-klein.dk,2026-08-17:ground/".into(),
        date: "2026-08-17".into(),
        generated_by: "the registry-generator fixture".into(),
    }
}

impl Args {
    /// Run `product registry generate` into `out`, returning what it printed.
    pub(crate) fn run(&self, out: &Path) -> Run {
        let output = Command::new(bin())
            .args(["registry", "generate"])
            .args(["--owner", &self.owner])
            .args(["--repo", &self.repo])
            .args(["--ratifier", &self.ratifier])
            .args(["--display-name", &self.display_name])
            .args(["--base-iri", &self.base_iri])
            .args(["--date", &self.date])
            .args(["--generated-by", &self.generated_by])
            .arg("--out")
            .arg(out)
            .output()
            .expect("run product registry generate");
        Run {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}

/// A generated instance in its own temporary directory. Dropping it removes
/// the directory, which is the whole of the generation's footprint.
pub(crate) struct Instance {
    dir: tempfile::TempDir,
    stdout: String,
}

/// Generate an instance, asserting the run succeeded.
pub(crate) fn generate(args: &Args) -> Instance {
    let dir = tempfile::tempdir().expect("tempdir");
    let out = dir.path().join("instance");
    let run = args.run(&out);
    assert_eq!(run.exit_code, 0, "generation failed:\n{}{}", run.stdout, run.stderr);
    Instance { dir, stdout: run.stdout }
}

impl Instance {
    /// The instance root.
    pub(crate) fn path(&self) -> PathBuf {
        self.dir.path().join("instance")
    }

    /// The directory the instance was generated into.
    pub(crate) fn parent(&self) -> PathBuf {
        self.dir.path().to_path_buf()
    }

    /// What the generation printed.
    pub(crate) fn stdout(&self) -> &str {
        &self.stdout
    }

    /// One of the instance's files.
    pub(crate) fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path().join(rel)).expect("read instance file")
    }

    /// Every tracked file, sorted.
    pub(crate) fn files(&self) -> Vec<String> {
        let mut out: Vec<String> = self
            .git(&["ls-files"])
            .lines()
            .map(|l| l.to_string())
            .filter(|l| !l.is_empty())
            .collect();
        out.sort();
        out
    }

    /// The birth commit's object id.
    pub(crate) fn commit(&self) -> String {
        self.git(&["rev-parse", "HEAD"]).trim().to_string()
    }

    /// Run git inside the instance.
    pub(crate) fn git(&self, args: &[&str]) -> String {
        let out = Command::new("git")
            .arg("-C")
            .arg(self.path())
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .output()
            .expect("run git");
        String::from_utf8_lossy(&out.stdout).to_string()
    }

    /// Run `product registry check` over the instance.
    pub(crate) fn check(&self) -> Run {
        let output = Command::new(bin())
            .args(["registry", "check"])
            .arg(self.path())
            .output()
            .expect("run product registry check");
        Run {
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code().unwrap_or(-1),
        }
    }
}
