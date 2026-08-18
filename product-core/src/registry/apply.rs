//! Committing a verified generation plan to a fresh local directory.
//!
//! This is the only part of the slice that touches the world, and it touches it
//! locally: it writes the tree, initialises a git repository, records the birth
//! provenance in one commit. **Publishing is a separate act** — no remote is
//! configured, no network call is made, no repository is created anywhere. The
//! report carries the two commands that would publish it, for a human to run.
//!
//! Minting identity and publishing it answer to different authorities, and only
//! the local half can be a fixture; a generator that did both would put the
//! untested half on the tested half's back.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::{ProductError, Result};

use super::plan::GenerationPlan;

/// The email recorded on the birth commit. RFC 2606 reserves `.invalid`: no
/// address was chosen, and none is implied. The *agent* is `--generated-by`.
pub const COMMIT_EMAIL: &str = "registry-generator@invalid";

/// A path git reads as an empty global config, so the developer's identity,
/// default branch, signing settings, line-ending rules never reach the birth
/// commit — the commit depends on the parameters alone.
#[cfg(unix)]
const NO_GLOBAL_CONFIG: &str = "/dev/null";
#[cfg(not(unix))]
const NO_GLOBAL_CONFIG: &str = "NUL";

/// What a generation did.
#[derive(Debug, serde::Serialize)]
pub struct GenerationReport {
    /// Where the instance was written.
    pub out: PathBuf,
    /// Files written, in template order.
    pub files: Vec<String>,
    /// The birth commit's object id.
    pub commit: String,
    /// The commands that would publish the instance — not run here.
    pub publish: Vec<String>,
}

/// Write a verified plan into `out`, then record the birth commit.
pub fn apply_generation(plan: &GenerationPlan, out: &Path) -> Result<GenerationReport> {
    ensure_free(out)?;
    std::fs::create_dir_all(out).map_err(|e| io_fault(out, &e.to_string()))?;
    for file in &plan.files {
        write_one(out, &file.path, &file.contents, file.executable)?;
    }
    let commit = birth_commit(plan, out)?;
    Ok(GenerationReport {
        out: out.to_path_buf(),
        files: plan.files.iter().map(|f| f.path.clone()).collect(),
        commit,
        publish: vec![
            format!(
                "git -C {} remote add origin git@github.com:{}/{}.git",
                out.display(),
                plan.params.owner,
                plan.params.repo
            ),
            format!("git -C {} push -u origin main", out.display()),
        ],
    })
}

/// The target must not already hold anything — a generation never merges into
/// an existing tree.
fn ensure_free(out: &Path) -> Result<()> {
    if !out.exists() {
        return Ok(());
    }
    if !out.is_dir() {
        return Err(fault(&format!("{} exists but is not a directory", out.display())));
    }
    let mut entries = std::fs::read_dir(out).map_err(|e| io_fault(out, &e.to_string()))?;
    if entries.next().is_some() {
        return Err(fault(&format!(
            "{} is not empty — generation never writes into an existing tree",
            out.display()
        )));
    }
    Ok(())
}

/// Write one planned file beneath `out`, refusing any path that escapes it.
fn write_one(out: &Path, rel: &str, contents: &str, executable: bool) -> Result<()> {
    if rel.starts_with('/') || rel.split('/').any(|seg| seg == ".." || seg.is_empty()) {
        return Err(fault(&format!("template path '{rel}' does not stay inside the target")));
    }
    let path = out.join(rel);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_fault(parent, &e.to_string()))?;
    }
    std::fs::write(&path, contents).map_err(|e| io_fault(&path, &e.to_string()))?;
    if executable {
        set_executable(&path)?;
    }
    Ok(())
}

#[cfg(unix)]
fn set_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
        .map_err(|e| io_fault(path, &e.to_string()))
}

#[cfg(not(unix))]
fn set_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Initialise the repository, then record the birth provenance in one commit.
///
/// The identity is inline, the dates are the mint date, the system config is
/// ignored: the commit depends on the parameters alone, so the same parameters
/// through the same generator produce the same commit id.
fn birth_commit(plan: &GenerationPlan, out: &Path) -> Result<String> {
    let stamp = format!("{}T00:00:00Z", plan.params.mint_date);
    git(out, &["init", "-q", "-b", "main"], &[])?;
    git(out, &["add", "-A"], &[])?;
    let message = commit_message(plan);
    git(
        out,
        &[
            "-c",
            &format!("user.name={}", plan.params.generated_by),
            "-c",
            &format!("user.email={COMMIT_EMAIL}"),
            "-c",
            "commit.gpgsign=false",
            "commit",
            "-q",
            "-m",
            &message,
        ],
        &[("GIT_AUTHOR_DATE", &stamp), ("GIT_COMMITTER_DATE", &stamp)],
    )?;
    Ok(git(out, &["rev-parse", "HEAD"], &[])?.trim().to_string())
}

/// The birth commit's message — the parameters, in the record.
fn commit_message(plan: &GenerationPlan) -> String {
    let p = &plan.params;
    format!(
        "Birth provenance — {}\n\n\
         Generated from the registry template {} by the product-cli registry\n\
         generator {}. The founding decision is not filed here: it is the\n\
         instance's first ratified content, filed by its ratifier.\n\n\
         owner:     {}\n\
         repository: {}\n\
         ratifier:  {}\n\
         base IRI:  {}\n\
         mint date: {}\n\
         generated by: {}\n",
        p.display_name,
        plan.template_version,
        plan.generator_version,
        p.owner,
        p.repo,
        p.ratifier,
        p.base_iri,
        p.mint_date,
        p.generated_by,
    )
}

/// Run git inside the generated tree only, reading neither the system config
/// nor the developer's identity.
fn git(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<String> {
    let mut cmd = Command::new("git");
    cmd.arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_GLOBAL", NO_GLOBAL_CONFIG);
    for (k, v) in env {
        cmd.env(k, v);
    }
    let out = cmd.output().map_err(|e| fault(&format!("git {:?} could not run: {e}", args)))?;
    if !out.status.success() {
        return Err(fault(&format!(
            "git {:?} failed: {}",
            args,
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn fault(message: &str) -> ProductError {
    ProductError::ConfigError(format!("registry generate: {message}"))
}

fn io_fault(path: &Path, message: &str) -> ProductError {
    ProductError::WriteError { path: path.to_path_buf(), message: message.to_string() }
}
