//! File write safety — atomic writes with advisory locking (ADR-015)

use crate::error::{ProductError, Result};
use std::fs;
use std::io::Write;

/// Check for uncommitted changes in artifact directories and print a warning.
/// Returns the count of modified files. Does not block on failure (non-git repos just return 0).
pub fn warn_uncommitted_changes(repo_root: &std::path::Path) -> usize {
    let output = std::process::Command::new("git")
        .args(["status", "--porcelain", "--", "docs/"])
        .current_dir(repo_root)
        .output();
    match output {
        Ok(out) if out.status.success() => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let modified: Vec<&str> = stdout.lines().filter(|l| !l.is_empty()).collect();
            if !modified.is_empty() {
                eprintln!(
                    "warning: {} modified file(s) in docs/ are uncommitted",
                    modified.len()
                );
                for line in &modified {
                    eprintln!("  {}", line);
                }
            }
            modified.len()
        }
        _ => 0, // not a git repo or git not available — silently skip
    }
}
use std::path::Path;

/// Write a file atomically: temp file + fsync + rename
pub fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let tmp_name = format!(".{}.product-tmp.{}", file_name, std::process::id());
    let tmp_path = path.with_file_name(tmp_name);

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| ProductError::WriteError {
            path: path.to_path_buf(),
            message: format!("failed to create directory: {}", e),
        })?;
    }

    // Write to temp file
    let mut file = fs::File::create(&tmp_path).map_err(|e| ProductError::WriteError {
        path: path.to_path_buf(),
        message: format!("failed to create temp file: {}", e),
    })?;

    file.write_all(content.as_bytes())
        .map_err(|e| ProductError::WriteError {
            path: path.to_path_buf(),
            message: format!("failed to write: {}", e),
        })?;

    file.sync_all().map_err(|e| ProductError::WriteError {
        path: path.to_path_buf(),
        message: format!("failed to fsync: {}", e),
    })?;

    // Atomic rename
    fs::rename(&tmp_path, path).map_err(|e| {
        // Clean up temp file on failure
        let _ = fs::remove_file(&tmp_path);
        ProductError::WriteError {
            path: path.to_path_buf(),
            message: format!("failed to rename: {}", e),
        }
    })?;

    Ok(())
}

/// Write multiple files atomically as a batch: create all temp files first,
/// fsync each, then rename all. If any step fails, clean up temps and return error.
/// This minimises the window for partial state (ADR-015, TC-366).
pub fn write_batch_atomic(writes: &[(&Path, &str)]) -> Result<usize> {
    if writes.is_empty() {
        return Ok(0);
    }

    // Phase 1: Create all temp files, write content, fsync each
    let mut staged: Vec<(std::path::PathBuf, std::path::PathBuf)> = Vec::new(); // (tmp_path, final_path)

    for (path, content) in writes {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| ProductError::WriteError {
                path: path.to_path_buf(),
                message: format!("failed to create directory: {}", e),
            })?;
        }

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let tmp_name = format!(".{}.product-tmp.{}", file_name, std::process::id());
        let tmp_path = path.with_file_name(&tmp_name);

        match write_and_sync_tmp(&tmp_path, content) {
            Ok(()) => staged.push((tmp_path, path.to_path_buf())),
            Err(e) => {
                // Clean up all temp files on failure
                for (tmp, _) in &staged {
                    let _ = fs::remove_file(tmp);
                }
                return Err(e);
            }
        }
    }

    // Phase 2: Rename all temp files to final destinations
    let mut completed = 0;
    for (tmp, final_path) in &staged {
        if let Err(e) = fs::rename(tmp, final_path) {
            // Clean up remaining temp files
            for (remaining_tmp, _) in staged.iter().skip(completed) {
                let _ = fs::remove_file(remaining_tmp);
            }
            return Err(ProductError::WriteError {
                path: final_path.clone(),
                message: format!("batch rename failed: {}", e),
            });
        }
        completed += 1;
    }

    Ok(completed)
}

/// Write content to a temp file and fsync it (helper for batch writes)
fn write_and_sync_tmp(tmp_path: &Path, content: &str) -> Result<()> {
    let mut file = fs::File::create(tmp_path).map_err(|e| ProductError::WriteError {
        path: tmp_path.to_path_buf(),
        message: format!("failed to create temp file: {}", e),
    })?;
    file.write_all(content.as_bytes())
        .map_err(|e| ProductError::WriteError {
            path: tmp_path.to_path_buf(),
            message: format!("failed to write: {}", e),
        })?;
    file.sync_all().map_err(|e| ProductError::WriteError {
        path: tmp_path.to_path_buf(),
        message: format!("failed to fsync: {}", e),
    })?;
    Ok(())
}

/// Clean up leftover .product-tmp.* files in a directory
pub fn cleanup_tmp_files(dir: &Path) {
    if !dir.exists() {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let name = entry.file_name();
            if let Some(name_str) = name.to_str() {
                if name_str.contains(".product-tmp.") {
                    let _ = fs::remove_file(entry.path());
                }
            }
        }
    }
}

/// Advisory lock on .product.lock file (ADR-015)
pub struct RepoLock {
    _lock_file: fs::File,
    lock_path: std::path::PathBuf,
}

impl RepoLock {
    /// Acquire an exclusive advisory lock with a 3-second timeout.
    /// If a stale lock is detected (holding PID no longer running), it is cleared.
    pub fn acquire(repo_root: &Path) -> Result<Self> {
        let lock_path = repo_root.join(".product.lock");
        clear_stale_lock(&lock_path);
        try_acquire_lock(&lock_path)
    }
}

/// If the lock file exists and the holding PID is no longer running, remove it.
fn clear_stale_lock(lock_path: &Path) {
    if lock_path.exists() {
        if let Ok(content) = fs::read_to_string(lock_path) {
            if let Some(pid) = parse_lock_pid(&content) {
                if !is_pid_alive(pid) {
                    let _ = fs::remove_file(lock_path);
                }
            }
        }
    }
}

/// Retry creating the lock file up to 30 times (100ms apart, ~3s timeout).
fn try_acquire_lock(lock_path: &std::path::PathBuf) -> Result<RepoLock> {
    let max_attempts = 30;
    for attempt in 0..max_attempts {
        match fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(lock_path)
        {
            Ok(file) => {
                let _ = std::io::Write::write_all(
                    &mut &file,
                    format!(
                        "pid={}\nstarted={}\n",
                        std::process::id(),
                        chrono::Utc::now().to_rfc3339()
                    )
                    .as_bytes(),
                );
                return Ok(RepoLock {
                    _lock_file: file,
                    lock_path: lock_path.clone(),
                });
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                if attempt < max_attempts - 1 {
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }
            Err(e) => {
                return Err(ProductError::LockError {
                    message: format!("failed to create lock file: {}", e),
                });
            }
        }
    }
    let holder = fs::read_to_string(lock_path).unwrap_or_default();
    Err(ProductError::LockError {
        message: format!(
            "another Product process is running on this repository\n  {}\n  wait for it to complete, or delete .product.lock if the process has died",
            holder.trim()
        ),
    })
}

impl Drop for RepoLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.lock_path);
    }
}

fn parse_lock_pid(content: &str) -> Option<u32> {
    for line in content.lines() {
        if let Some(pid_str) = line.strip_prefix("pid=") {
            return pid_str.trim().parse().ok();
        }
    }
    None
}

/// Check if a process with the given PID is still running
fn is_pid_alive(pid: u32) -> bool {
    // On Unix, kill(pid, 0) checks existence without sending a signal
    #[cfg(unix)]
    {
        unsafe { libc::kill(pid as i32, 0) == 0 }
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        true // Assume alive on non-Unix — don't clear lock
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn atomic_write_creates_file() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.md");
        write_file_atomic(&path, "hello world").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "hello world");
    }

    #[test]
    fn atomic_write_no_tmp_leftover() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.md");
        write_file_atomic(&path, "content").unwrap();
        let entries: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].file_name().to_str().unwrap(), "test.md");
    }

    #[test]
    fn cleanup_removes_tmp_files() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join(".test.product-tmp.123"), "old").unwrap();
        fs::write(dir.path().join("keep.md"), "keep").unwrap();
        cleanup_tmp_files(dir.path());
        assert!(!dir.path().join(".test.product-tmp.123").exists());
        assert!(dir.path().join("keep.md").exists());
    }
}
