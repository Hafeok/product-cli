## Overview

Artifact Authoring provides the write-side commands for the Product knowledge graph. Where Phase 1 commands read and navigate the graph, FT-004 adds the ability to scaffold new artifacts, link them together, and update their status — all from the command line. Every write operation uses atomic file writes (ADR-015) and validates graph integrity so that the knowledge graph remains consistent after every mutation.

## Tutorial

This tutorial walks you through creating a feature, linking it to a decision and a test criterion, and updating statuses as work progresses.

### Step 1: Scaffold a new feature

```bash
product feature new "Cluster Foundation"
```

Product assigns the next available ID (e.g., `FT-031`) and creates a file at `docs/features/FT-031-cluster-foundation.md` with all required front-matter fields pre-filled. The ID is always one higher than the current maximum — gaps in the sequence are never filled.

### Step 2: Scaffold a related ADR and test criterion

```bash
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario
```

Each command creates a new file with the appropriate prefix (`ADR-XXX`, `TC-XXX`) and populates the front-matter with sensible defaults.

### Step 3: Link the artifacts together

Connect the feature to its governing decision and its test criterion:

```bash
product feature link FT-031 --adr ADR-027
product feature link FT-031 --test TC-156
```

Product validates that no dependency cycles are introduced. If the link would create a cycle, the command exits with error E003 and the file is not modified.

### Step 4: Update status as work progresses

```bash
product adr status ADR-027 accepted
product feature status FT-031 in-progress
```

When you mark an ADR as superseded, Product automatically runs impact analysis so you can see the blast radius before committing:

```bash
product adr status ADR-027 superseded --by ADR-028
```

### Step 5: Verify and complete

Once implementation is done, verify the feature against its test criteria and mark it complete:

```bash
product verify FT-031
```

If all linked TCs pass, `product verify` updates the feature status to complete and regenerates `CHECKLIST.md` automatically.

## How-to Guide

### Scaffold a new artifact

1. Choose the artifact type: `feature`, `adr`, or `test`.
2. Run the appropriate `new` command with a descriptive title:
   ```bash
   product feature new "My Feature Title"
   product adr new "My Decision Title"
   product test new "My Test Title" --type scenario
   ```
3. The new file is created in the configured directory with auto-incremented ID and default front-matter.

### Link a feature to ADRs and test criteria

1. Identify the feature ID and the target artifact IDs.
2. Run the link command for each relationship:
   ```bash
   product feature link FT-005 --adr ADR-003
   product feature link FT-005 --test TC-010
   ```
3. Confirm the link was created by checking the feature's front-matter or running `product context FT-005`.

### Update an artifact's status

1. Run the status command with the artifact ID and the new status value:
   ```bash
   product feature status FT-005 complete
   product adr status ADR-003 accepted
   product test status TC-010 passing
   ```
2. For ADR supersession, specify the replacement:
   ```bash
   product adr status ADR-003 superseded --by ADR-015
   ```
3. Review the impact summary that is printed automatically before the change is committed.

### Check for problems after authoring

1. Run graph health checks to catch broken links or cycles:
   ```bash
   product graph check
   ```
2. Run gap analysis and drift detection:
   ```bash
   product gap check
   product drift check
   ```

### Recover from a concurrent write conflict

If two Product processes attempt writes simultaneously, one will fail with error E010:

1. Wait for the other process to finish (the lock has a 3-second timeout).
2. Re-run your command.
3. If the lock is stale (the holding process crashed), Product detects the dead PID and clears the lock automatically on the next invocation.

## Reference

### Commands

#### `product feature new <title>`

Scaffolds a new feature file with auto-incremented ID.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<title>` — human-readable feature title |
| **Output** | Path to the created file |
| **ID assignment** | `max(existing FT IDs) + 1`; gaps are not filled |
| **File location** | Configured `features` path in `product.toml` |

#### `product adr new <title>`

Scaffolds a new ADR file with auto-incremented ID.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<title>` — human-readable decision title |
| **Output** | Path to the created file |
| **ID assignment** | `max(existing ADR IDs) + 1` |

#### `product test new <title> --type <type>`

Scaffolds a new test criterion file.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<title>` — test criterion title |
| **Flags** | `--type` — one of `scenario`, `invariant`, `chaos`, `exit-criteria` (ADR-011) |
| **ID assignment** | `max(existing TC IDs) + 1` |

#### `product feature link <id> --adr <adr-id>` / `--test <tc-id>`

Adds a graph edge by mutating the feature's front-matter.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<id>` — the feature to modify |
| **Flags** | `--adr <adr-id>` or `--test <tc-id>` — the target artifact |
| **Validation** | Rejects links that would introduce a `depends-on` cycle (E003) |
| **Write safety** | Front-matter updated via `fileops::atomic_write` (ADR-015) |

#### `product feature status <id> <status>`

Updates a feature's status in its front-matter.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<id>` — feature ID; `<status>` — new status value |

#### `product adr status <id> <status> [--by <adr-id>]`

Updates an ADR's status. When setting `superseded`, use `--by` to record the replacement.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<id>` — ADR ID; `<status>` — new status value |
| **Flags** | `--by <adr-id>` — the superseding ADR (required with `superseded`) |
| **Side effect** | Prints impact analysis summary before committing the change |

#### `product test status <id> <status>`

Updates a test criterion's status in its front-matter.

| Aspect | Detail |
|--------|--------|
| **Arguments** | `<id>` — TC ID; `<status>` — new status value |

### Error codes

| Code | Meaning |
|------|---------|
| **E003** | Cycle detected — a link would introduce a `depends-on` cycle |
| **E008** | Schema version mismatch — binary does not support the repository's schema version |
| **E009** | Atomic write failure — temp file could not be written or renamed |
| **E010** | Repository locked — another Product process holds the advisory lock |

### Warning codes

| Code | Meaning |
|------|---------|
| **W007** | Schema upgrade available — `product migrate schema` can upgrade the repository |

### File write mechanics

All write operations follow the atomic write protocol defined in ADR-015:

1. Compute full file content in memory.
2. Write to a temporary file: `.<filename>.product-tmp.<pid>`.
3. `fsync` the temporary file.
4. Rename (atomic on POSIX) to the target path.
5. On failure: delete the temp file, surface E009.

The advisory lock file `.product.lock` serialises concurrent writes with a 3-second timeout. Read-only commands never acquire the lock.

### ID assignment rules

- IDs are prefixed numeric: `FT-XXX`, `ADR-XXX`, `TC-XXX` (ADR-005).
- The next ID is always `max(existing) + 1`. Gaps from deleted or abandoned artifacts are not reused.
- Once assigned, an ID is permanent. Artifacts are never renumbered.
- Prefixes are configurable in `product.toml`.

## Explanation

### Why atomic writes matter

Product manages long-lived project artifacts — feature specs, architectural decisions, and test criteria that accumulate over months or years. A torn write (partial file content due to an interrupted process) corrupts YAML front-matter, which silently breaks the knowledge graph. The atomic temp-file-plus-rename pattern (ADR-015) guarantees that a file is either fully written or untouched. This is the same approach used by git, package managers, and text editors.

### Advisory locking vs. strict locking

The advisory lock on `.product.lock` only serialises concurrent Product invocations. It does not prevent editors, git, or other tools from modifying artifact files. This is intentional — Product should not interfere with the developer's normal workflow. The lock exists to prevent two simultaneous `product` commands from silently discarding each other's writes (ADR-015).

Stale lock detection checks whether the PID recorded in the lock file is still running. If the holding process has crashed, Product clears the lock automatically, so developers rarely need to manually delete `.product.lock`.

### ID scheme design

The prefixed numeric scheme (ADR-005) balances readability, stability, and sortability. `FT-001` in a commit message or code comment is immediately meaningful. Sequential assignment means IDs are never reused — external references in code comments, Slack messages, or commit messages remain valid indefinitely. Gaps are intentionally left unfilled so that an artifact's ID is a permanent identifier, not a position in a sequence.

### Front-matter as source of truth

All graph relationships are declared in YAML front-matter within each artifact file (ADR-002). There is no separate graph database or index file. The graph is rebuilt from front-matter on every CLI invocation (ADR-003). This eliminates synchronisation problems — the graph cannot drift from the documents because it is always derived from them. When `product feature link` adds an edge, it mutates the front-matter directly, and the next command that reads the graph will see the change.

### Cycle detection on link

The `product feature link` command validates that no `depends-on` cycles are introduced before writing. This is checked against the full graph, not just the immediate link. A cycle in the dependency graph would make topological sorting impossible (ADR-012), which would break `product feature next` and dependency-ordered context assembly. The validation runs in-memory before any file is modified, so a rejected link has no side effects.

### Impact analysis on ADR supersession

When an ADR is marked as superseded, the blast radius can be significant — every feature governed by that decision and every test that validates it may need re-evaluation. Product automatically computes and displays this impact set using reverse-graph reachability (ADR-012) before committing the status change. This gives the developer visibility into downstream consequences at the moment the decision is made, not after the fact.
