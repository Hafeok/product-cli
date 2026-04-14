---
id: ADR-015
title: File Write Safety — Atomic Writes and Advisory Locking
status: accepted
features: []
supersedes: []
superseded-by: []
domains: [storage]
scope: domain
content-hash: sha256:0fbe0d71baab6fcbae5ec8f9f0a4b768a0202503fdae91dbba4cc1c9b1f70605
---

**Status:** Accepted

**Context:** Product mutates files in two scenarios: authoring commands (`product feature status`, `product feature link`, `product adr new`) and generation commands (`product checklist generate`, `product graph rebuild`, `product migrate schema`). Two failure modes are possible:

1. **Torn writes:** a command writes partially to a file and is interrupted (process kill, power loss, disk full). The file is left in a corrupt state — truncated YAML front-matter, incomplete markdown.

2. **Concurrent writes:** two invocations of Product run simultaneously (common in CI with parallel jobs, or a developer running a command while a CI check runs). Both read the same file, both compute updates, and the last writer silently discards the first writer's changes.

Neither failure mode is acceptable for a tool that manages long-lived project artifacts. A corrupt front-matter file silently breaks the graph. Silent data loss from concurrent writes is worse than a conflict error.

**Decision:** All file writes use atomic temp-file-plus-rename. An advisory lock on `product.toml` serialises concurrent Product invocations on the same repository. Reads never acquire the lock.

---

### Atomic Writes

Every file write in Product follows this sequence:

1. Compute the full new file content in memory
2. Write to a temporary file in the same directory: `.<filename>.product-tmp.<pid>`
3. `fsync` the temporary file
4. Rename the temporary file to the target filename (atomic on POSIX systems)
5. On failure at any step: delete the temporary file, surface error E009

```rust
fn write_file_atomic(path: &Path, content: &str) -> Result<(), ProductError> {
    let tmp = path.with_file_name(format!(
        ".{}.product-tmp.{}",
        path.file_name().unwrap().to_str().unwrap(),
        std::process::id()
    ));
    std::fs::write(&tmp, content)?;
    // fsync before rename
    let file = std::fs::File::open(&tmp)?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

Rename is atomic on all POSIX filesystems. On Windows (where rename over an existing file requires an explicit move), `std::fs::rename` is used with the understanding that Windows atomic rename semantics differ; a Windows-specific implementation may be needed if Windows support is added.

---

### Advisory Lock

Product acquires an exclusive advisory lock on a `.product.lock` file in the same directory as `product.toml` before any write operation. The lock is released on process exit (including on signal).

Read-only commands (`product feature list`, `product context`, `product graph check`) do not acquire the lock.

Write commands acquire the lock with a **3-second timeout**. If the lock is not acquired within 3 seconds, Product exits with error E010:

```
error[E010]: repository locked
  another Product process is running on this repository
  lock held by PID 48291 (started 2026-04-11T09:14:22Z)
  wait for it to complete, or delete .product.lock if the process has died
```

The lock file contains the PID and start time of the holding process, enabling the error message to be informative. If the holding PID is not running (stale lock from a crashed process), Product detects this and acquires the lock without the timeout.

**Implementation:** `fd-lock` crate — cross-platform advisory file locking with no external dependencies.

---

### Temporary File Cleanup

On startup, Product scans for `*.product-tmp.*` files in the repository directories and deletes them. These are always safe to delete — they are either complete (and were renamed) or incomplete (and should be discarded). This handles the case where a previous invocation was killed after writing the temp file but before the rename.

---

**Rationale:**
- Atomic rename is the standard POSIX pattern for safe file writes. It is used by git, package managers, and text editors for exactly this reason. Implementing it in Product follows established practice.
- Advisory locking is advisory — a non-Product process can still modify files. This is intentional: Product should not prevent editors, git operations, or other tools from working. It only serialises concurrent Product invocations.
- The 3-second lock timeout is short enough to fail fast (a developer running two commands simultaneously gets an immediate error, not a hang) but long enough to survive brief system load spikes.
- Stale lock detection (PID not running) prevents the lock file from becoming a permanent block after a crash. The developer should not need to manually delete `.product.lock` under normal failure conditions.

**Rejected alternatives:**
- **No locking, accept last-write-wins** — silent data loss. Rejected.
- **Exclusive lock on every file written** — too granular. Would require acquiring N locks for a command that writes N files, with partial failure and rollback complexity.
- **SQLite as the write store** — SQLite handles locking internally. Rejected because it would make all artifact files non-human-editable binary blobs, contradicting the foundational design decision (ADR-002).
- **Process mutex via socket** — more reliable than file locking on some systems. Rejected because it requires a listening socket and introduces a cleanup problem on process death.