## Overview

FT-005 ensures that every file write performed by Product is safe against corruption and data loss. All writes use atomic temp-file-plus-rename, and an advisory lock on `.product.lock` serialises concurrent write commands within the same repository. This protects long-lived project artifacts (feature files, ADR files, checklists) from torn writes, silent overwrites, and leftover temporary files. The implementation follows ADR-015.

## Tutorial

### Seeing atomic writes in action

Product handles atomic writes transparently. Every command that modifies a file uses the safe-write path automatically. Try it by updating a feature status:

```bash
product feature status FT-001 in-progress
```

Behind the scenes, Product:

1. Computed the new file content in memory
2. Wrote it to a temporary file (`.FT-001-core-concepts.md.product-tmp.<pid>`)
3. Called `fsync` on the temporary file
4. Renamed the temporary file over the target (atomic on POSIX)

You never see the temporary file because it exists only for the duration of the write. If the process is killed mid-write, the original file remains untouched.

### Observing the advisory lock

Open two terminals in the same repository. In the first terminal, run a write command:

```bash
product checklist generate
```

While that is running, immediately run another write command in the second terminal:

```bash
product feature status FT-002 in-progress
```

If the first command is still holding the lock, the second command waits up to 3 seconds. If the lock is not released in time, you see:

```
error[E010]: repository locked
  another Product process is running on this repository
  lock held by PID 48291 (started 2026-04-14T09:14:22Z)
  wait for it to complete, or delete .product.lock if the process has died
```

This ensures two Product processes never write to the same files simultaneously.

### Automatic cleanup of stale state

If a previous Product invocation crashed, it may have left behind a `.product.lock` file or `.product-tmp.*` files. Product detects and cleans both automatically:

```bash
# Even a read-only command cleans leftover temp files
product feature list
```

Stale lock files (where the holding PID is no longer running) are cleared automatically when any write command runs.

## How-to Guide

### Resolve a "repository locked" error

1. Check whether another Product process is actually running. The error message includes the PID:
   ```
   lock held by PID 48291 (started 2026-04-14T09:14:22Z)
   ```
2. If the process is still running, wait for it to finish and retry.
3. If the process has died (e.g., was killed), Product normally detects this and clears the lock automatically. If it does not, delete the lock file manually:
   ```bash
   rm .product.lock
   ```
4. Re-run your command.

### Clean up leftover temporary files

Leftover `.product-tmp.*` files are cleaned automatically on any Product invocation, including read-only commands. To trigger cleanup explicitly:

```bash
product feature list
```

If you want to verify no temporary files remain:

```bash
find docs/ -name '*.product-tmp.*'
```

### Run concurrent CI jobs safely

Product's advisory lock serialises write commands within the same repository clone. In CI pipelines with parallel jobs:

1. If jobs share the same checkout directory, the lock ensures only one write command runs at a time. The other job receives error E010 if it times out.
2. If each job has its own checkout (recommended), no lock contention occurs.

For parallel CI, prefer separate checkouts to avoid lock contention entirely.

## Reference

### Atomic write sequence

Every file write follows this sequence:

| Step | Action | Failure behavior |
|------|--------|------------------|
| 1 | Compute full file content in memory | Error surfaced before any disk I/O |
| 2 | Write to `.<filename>.product-tmp.<pid>` in the same directory | Temp file deleted, error E009 |
| 3 | `fsync` the temporary file | Temp file deleted, error E009 |
| 4 | Rename temp file to target path (atomic on POSIX) | Temp file deleted, error E009 |

The temporary file naming pattern is `.<original-filename>.product-tmp.<pid>`, where `<pid>` is the current process ID.

### Advisory lock

| Property | Value |
|----------|-------|
| Lock file | `.product.lock` (same directory as `product.toml`) |
| Timeout | 3 seconds |
| Error on timeout | E010 |
| Stale detection | Checks if holding PID is still running |
| Implementation | `fd-lock` crate |

**Commands that acquire the lock** (write commands):

- `product feature status`
- `product feature link`
- `product adr new`
- `product checklist generate`
- `product graph rebuild`
- `product migrate schema`
- `product verify`

**Commands that do not acquire the lock** (read-only commands):

- `product feature list`
- `product context`
- `product graph check`
- `product gap check`
- `product drift check`

### Lock file contents

The `.product.lock` file stores:

- PID of the holding process
- Start timestamp of the holding process

This information is used in the E010 error message and for stale lock detection.

### Temporary file cleanup

On startup, Product scans repository directories for files matching `*.product-tmp.*` and deletes them. This runs on every invocation, including read-only commands.

### Error codes

| Code | Meaning |
|------|---------|
| E009 | Atomic write failed (temp file write, fsync, or rename error) |
| E010 | Repository locked by another Product process |

### Implementation module

The atomic write and locking logic lives in `src/fileops.rs`, exposed as `fileops::atomic_write()`.

## Explanation

### Why atomic writes matter

Product manages long-lived project artifacts: feature specs, ADRs, test criteria, and checklists. A torn write (partial file content due to an interrupted process) can silently corrupt YAML front-matter, which breaks the knowledge graph on the next invocation. Atomic rename ensures that a file is either fully written or completely unchanged -- there is no intermediate state visible to other processes or subsequent invocations.

This is the same pattern used by git, package managers, and text editors. ADR-015 adopted it as the standard write mechanism for all Product file operations.

### Why advisory locking instead of alternatives

The advisory lock serialises concurrent Product invocations but does not prevent other tools (editors, git, scripts) from modifying files. This is intentional -- Product should coexist with the developer's normal workflow, not block it. The lock only prevents two Product processes from racing on the same files.

Alternatives considered and rejected (per ADR-015):

- **No locking (last-write-wins):** Silent data loss when two processes write the same file.
- **Per-file exclusive locks:** Acquiring N locks for N files introduces partial failure and rollback complexity.
- **SQLite as write store:** Would make artifact files non-human-editable binary blobs, contradicting the file-based design (ADR-002).
- **Process mutex via socket:** Requires a listening socket and introduces cleanup problems on process death.

### The 3-second timeout trade-off

The lock timeout is deliberately short. A developer who accidentally runs two write commands simultaneously gets an immediate error rather than a silent hang. Three seconds is long enough to tolerate brief system load spikes but short enough that waiting feels instantaneous in the failure case. If the holding process is legitimately long-running (e.g., a large `checklist generate`), the second process fails fast with an actionable error message.

### Stale lock recovery

If a Product process is killed (SIGKILL, power loss, OOM), the `.product.lock` file persists on disk. On the next write command, Product checks whether the PID recorded in the lock file is still running. If not, it assumes the lock is stale, clears it, and proceeds. This means developers rarely need to manually delete `.product.lock` -- the common crash scenario is handled automatically.
