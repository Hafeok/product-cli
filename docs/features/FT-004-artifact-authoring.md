---
id: FT-004
title: Artifact Authoring
phase: 2
status: complete
depends-on:
- FT-003
- FT-016
adrs:
- ADR-002
- ADR-005
- ADR-015
tests:
- TC-005
- TC-006
- TC-007
- TC-008
- TC-013
- TC-014
- TC-015
- TC-071
- TC-072
- TC-073
- TC-074
- TC-075
- TC-076
- TC-077
- TC-078
- TC-079
- TC-155
- TC-160
domains:
- api
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  api: Authoring commands (feature new, adr new, test new, dep new) define CLI subcommands but the API surface is governed by ADR-002 (front-matter schema) and ADR-005 (ID scheme), both already linked.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

Scaffold, link, and update artifacts from the command line. These commands are the write-side counterpart to the read-only navigation commands in Phase 1.

### Scaffold

```
product feature new "Cluster Foundation"   # scaffold FT-XXX with next auto-incremented ID
product adr new "Use openraft for consensus"
product test new "Raft leader election" --type scenario
```

Scaffolded files include all required front-matter fields with sensible defaults. The ID is auto-incremented from the highest existing ID of that artifact type.

### Link

```
product feature link FT-001 --adr ADR-002   # add edge (mutates front-matter)
product feature link FT-001 --test TC-002
```

Linking validates that no `depends-on` cycles are introduced (E003). Front-matter is updated atomically using `fileops::atomic_write`.

### Status Update

```
product adr status ADR-002 accepted
product test status TC-002 passing
product feature status FT-001 complete
```

ADR supersession triggers an impact report. Front-matter validation on write — type checking, ID format, unknown fields preserved.

---

## Description

FT-004 provides the write-side CLI commands for creating and modifying artifacts: scaffolding new Features, ADRs, and Test Criteria with auto-incremented IDs; adding graph edges by linking artifacts; and updating status fields. These commands are the counterpart to the read-only navigation commands in Phase 1 (FT-010). All writes are performed atomically via `fileops::atomic_write` (ADR-015), and concurrent writes are serialised by an advisory lock on `product.toml`. The ID auto-increment rule (ADR-005) guarantees that newly scaffolded artifacts receive the next unused numeric suffix for their prefix type. Linking commands validate the resulting graph for DAG cycles (E003) before writing. Status updates on ADRs trigger an automatic impact report when the new status is `superseded`.

## Functional Specification

### Inputs

- CLI subcommand with arguments:
  - `product feature new "<title>"` — title string; optional `--phase N`, `--status STATUS`.
  - `product adr new "<title>"` — title string; optional `--status STATUS`.
  - `product test new "<title>" --type TYPE` — title string and required `type` (`scenario | invariant | chaos | exit-criteria | benchmark`).
  - `product feature link FT-XXX --adr ADR-XXX` or `--test TC-XXX` — source feature ID and target artifact ID.
  - `product feature status FT-XXX STATUS` — feature ID and new status value.
  - `product adr status ADR-XXX STATUS [--by ADR-YYY]` — ADR ID, new status, and optional superseding ADR.
  - `product test status TC-XXX STATUS` — TC ID and new status value.
- The in-memory graph, rebuilt from all current front-matter before any write command executes.

### Outputs

- A new `.md` file written atomically to the appropriate configured directory with a complete, valid front-matter block and a scaffold body. The assigned ID is printed to stdout.
- For link commands: the source artifact's front-matter is updated in place to add the new edge; the modified file is written atomically.
- For status commands: the target artifact's front-matter `status` field is updated in place and the modified file is written atomically.
- For `product adr status ADR-XXX superseded`: an impact report is printed to stdout showing all affected features and tests before the write completes.

### State

Write operations acquire an exclusive advisory lock on `.product.lock` (ADR-015) before modifying any file. The lock is released on process exit. Read-only preflight (ID selection, cycle check) executes on the in-memory graph before the lock is acquired. No state is persisted between invocations beyond the modified artifact files.

### Behaviour

1. **Scaffold (`feature new`, `adr new`, `test new`)**: Product scans the configured directory for all existing artifact files, finds the highest numeric suffix for the prefix type, and assigns `highest + 1` as the new ID (zero-padded to three digits). If no artifacts exist yet, the first ID is `001`. The scaffolded file is written to `<directory>/<ID>-<slugified-title>.md` with all required front-matter fields set to their defaults and a placeholder body.
2. **Link (`feature link`)**: Product loads the source feature's front-matter, appends the target artifact ID to the appropriate list (`adrs` or `tests`), and validates that no `depends-on` cycle is introduced (for `--dep` links). If the target ID does not exist in the graph, E002 is reported and the write is aborted. The updated feature file is then written atomically.
3. **Status update (`feature status`, `adr status`, `test status`)**: Product reads the artifact's current front-matter, sets the `status` field to the new value (validated against the closed enum), and writes the file atomically. For `adr status ADR-XXX superseded --by ADR-YYY`, Product additionally sets `superseded-by: [ADR-YYY]` and `supersedes` on ADR-YYY, running impact analysis first and printing the report before writing.
4. All writes use `fileops::write_file_atomic` (temp-file + rename + fsync), preventing torn writes (ADR-015).
5. Concurrent invocations are serialised by an advisory lock with a 3-second timeout; a second process that cannot acquire the lock exits with E010 (ADR-015).

### Invariants

- The auto-incremented ID for a new artifact is always strictly greater than all existing IDs of the same prefix type. Gap-filling (reusing a retired ID) is not performed; retired artifacts are marked `abandoned`, not deleted (ADR-005).
- A link command must not introduce a cycle in the `depends-on` DAG; the cycle check runs on the in-memory graph before any file is written.
- All file writes are atomic: the target file is either fully written or unchanged; partial writes cannot occur (ADR-015).
- Unknown front-matter fields in a file being updated are preserved verbatim (ADR-014).
- The advisory lock is always released, including on SIGINT and SIGTERM (via `fd-lock` RAII).

### Error handling

- Target artifact ID does not exist → E002, write aborted; no file is modified.
- `depends-on` cycle would be introduced by the link → E003, write aborted.
- ADR supersession cycle would be introduced → E004, write aborted.
- Invalid status value for the artifact type → E007.
- Repository locked by another process → E010 with PID and start time of the lock holder (ADR-015).
- Disk write failure during atomic write → the temp file is deleted; the original file is unchanged; error is reported on stderr.

### Boundaries

- Authoring commands only scaffold and edit files in the configured `features`, `adrs`, and `tests` directories. They do not modify `product.toml`, `checklist.md`, or `index.ttl`.
- The `feature link` command only adds edges; it does not remove them. Removing a link requires manual front-matter editing.
- Status transitions are unconstrained by the CLI — any valid status value may be set in any order. Semantic correctness (e.g., not marking a feature `complete` with failing TCs) is the developer's responsibility; `product verify` provides the automated gate.

## Out of scope

- Interactive authoring sessions with LLM-assisted content generation — covered by `product author feature` / `product author adr` (FT-022).
- TC inference and transitive link propagation during migration — covered by FT-027 (`product migrate link`).
- Checklist regeneration — automatically triggered by `product verify`; manually triggered by `product checklist generate` (FT-017).
- Validation that a feature is ready for implementation before invoking an agent — covered by the pre-flight and implement pipeline (FT-026, FT-030).
