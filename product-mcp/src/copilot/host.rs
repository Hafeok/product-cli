//! Locate the Copilot CLI binary for the SDK-hosted session.
//!
//! The SDK's own resolver never scans `PATH`, and its build-time CLI
//! download is disabled repo-wide (`COPILOT_SKIP_CLI_DOWNLOAD` in
//! `.cargo/config.toml`), so the host owns resolution: `COPILOT_CLI_PATH`
//! wins, then a `PATH` scan for `copilot` — parity with the previous
//! `Command::new("copilot")` launch.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use product_core::error::{ProductError, Result};

/// Resolve the Copilot CLI binary from the process environment.
pub fn resolve_cli() -> Result<PathBuf> {
    resolve_cli_from(std::env::var_os("COPILOT_CLI_PATH"), std::env::var_os("PATH"))
}

/// Resolution over explicit inputs (testable without touching the process
/// environment): an explicit path that exists wins; otherwise the first
/// executable `copilot` on the search path.
fn resolve_cli_from(explicit: Option<OsString>, path: Option<OsString>) -> Result<PathBuf> {
    if let Some(p) = explicit {
        let p = PathBuf::from(p);
        if p.is_file() {
            return Ok(p);
        }
        return Err(ProductError::IoError(format!(
            "COPILOT_CLI_PATH is set but does not point at a file: {}",
            p.display()
        )));
    }
    for dir in path.iter().flat_map(std::env::split_paths) {
        for name in cli_names() {
            let candidate = dir.join(name);
            if is_executable(&candidate) {
                return Ok(candidate);
            }
        }
    }
    Err(ProductError::IoError(
        "could not find the Copilot CLI. Install it and either put `copilot` on PATH \
         or set COPILOT_CLI_PATH to the binary."
            .to_string(),
    ))
}

fn cli_names() -> &'static [&'static str] {
    if cfg!(windows) {
        &["copilot.exe", "copilot.cmd", "copilot.bat"]
    } else {
        &["copilot"]
    }
}

#[cfg(unix)]
fn is_executable(p: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;
    p.is_file()
        && std::fs::metadata(p)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
}

#[cfg(not(unix))]
fn is_executable(p: &Path) -> bool {
    p.is_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    fn touch_executable(dir: &Path, name: &str) -> PathBuf {
        use std::os::unix::fs::PermissionsExt;
        let p = dir.join(name);
        std::fs::write(&p, "#!/bin/sh\n").expect("write");
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).expect("chmod");
        p
    }

    #[test]
    fn explicit_path_wins() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let p = tmp.path().join("copilot-custom");
        std::fs::write(&p, "").expect("write");
        let got = resolve_cli_from(Some(p.clone().into()), None).expect("resolved");
        assert_eq!(got, p);
    }

    #[test]
    fn explicit_path_that_is_missing_errors_rather_than_falling_through() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let missing = tmp.path().join("nope");
        let err = resolve_cli_from(Some(missing.into()), None).expect_err("should error");
        assert!(format!("{err}").contains("COPILOT_CLI_PATH"));
    }

    #[cfg(unix)]
    #[test]
    fn path_scan_finds_an_executable_copilot() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let bin = touch_executable(tmp.path(), "copilot");
        let path_var = std::env::join_paths([tmp.path()]).expect("join");
        let got = resolve_cli_from(None, Some(path_var)).expect("resolved");
        assert_eq!(got, bin);
    }

    #[cfg(unix)]
    #[test]
    fn path_scan_skips_a_non_executable_file() {
        let tmp = tempfile::tempdir().expect("tempdir");
        std::fs::write(tmp.path().join("copilot"), "").expect("write");
        let path_var = std::env::join_paths([tmp.path()]).expect("join");
        let err = resolve_cli_from(None, Some(path_var)).expect_err("should error");
        assert!(format!("{err}").contains("Copilot CLI"));
    }

    #[test]
    fn empty_environment_yields_the_install_hint() {
        let err = resolve_cli_from(None, None).expect_err("should error");
        let msg = format!("{err}");
        assert!(msg.contains("copilot"), "hint names the binary: {msg}");
        assert!(msg.contains("COPILOT_CLI_PATH"), "hint names the env var: {msg}");
    }
}
