---
id: FT-051
title: Relative Paths in the Request Log
phase: 5
status: complete
depends-on:
- FT-041
- FT-042
adrs:
- ADR-038
- ADR-039
tests:
- TC-623
- TC-624
- TC-625
domains:
- data-model
- observability
- security
domains-acknowledged:
  ADR-043: Implementation is a new pure helper under `src/request_log/` paired with the existing append adapters; it follows the slice + adapter split by construction.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: Request-log writes are the record-side of the LLM boundary already owned by FT-041 / FT-042; relativisation is a pure string transform at the same boundary and adds no new pipeline stage.
  ADR-041: Path normalisation is orthogonal to absence-TC and ADR removes/deprecates lifecycle; the transform applies to all entry types uniformly.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Log entries carry no tc-type partition; the transform is independent of the TC vocabulary defined by ADR-042.
---

## Description

`requests.jsonl` is committed to the repository as a tamper-evident audit
of every `product request` apply. Today every `file` field inside a
`request` / `result.created` / `result.changed` entry carries the
**absolute path** the write landed at — e.g.
`/home/hafeok/projects/product-cli/docs/features/FT-043-….md`.

That leaks the committer's username and filesystem layout into the git
history every time the log is appended. Any GitHub search for
`/home/USER` across the repo turns up the log. For a log whose purpose is
to be shared across collaborators and mirrored into CI, this is a privacy
leak, a reproducibility wart (a clone under a different path replays
differently), and a noisy diff (two contributors cannot agree on the same
entry because their home directories differ).

This feature normalises every path-typed field in the log to a
repo-root-relative, POSIX-style path (`docs/features/FT-043-….md`). New
entries are written relative; existing absolute-path entries are rewritten
via a one-shot migration entry that preserves hash-chain integrity
(`type: migrate`, ADR-039 decision 4).

Originates from GitHub issue #6 ("use relative paths in the product
request log as to not expose machine names").

---

## Depends on

- **FT-041** — product request unified write interface. Path emission
  lives in the request planner; this feature normalises the strings before
  they hit the log writer.
- **FT-042** — request-log hash chain and replay. The migration entry is
  implemented using the hash-chain's existing `migrate` entry type rather
  than rewriting history.

---

## Scope of this feature

### In

1. **`path_relativize(path, repo_root) -> String`** helper in
   `request_log::canonical` (or a new sibling `paths.rs`). Contract:
    - If `path` is under `repo_root`, return the POSIX-joined suffix.
    - If `path` is already relative, pass through unchanged
      (re-normalised to POSIX separators on Windows).
    - If `path` escapes `repo_root` (not expected inside Product), return
      the original absolute string and emit a W-class warning so the
      regression is visible.
   Uses `Path::strip_prefix` after both sides are canonicalised; backed
   by `pathdiff` only if the stdlib cannot express the diff.
2. **Emission.** `append_apply_entry`, `append_undo_entry`, and
   `append_migrate_entry` route every user-visible `file:` field (the
   `request.created[].file`, `request.changed[].file`, and
   `result.created[].file` positions if we ever add them) through
   `path_relativize` before serialising. The `request_log` module owns
   this transform; callers continue to pass absolute paths.
3. **Migration entry.** `product request-log migrate-paths` subcommand
   (and an auto-run from `product request-log verify` when it detects an
   entry with a leading `/` or drive letter under `file`) appends one
   `migrate` entry listing the affected entry IDs and rewrites each
   offending line's `file` field to the relative form. Hashes for the
   rewritten lines are **not** recomputed (that would break the chain);
   instead the migrate entry documents the rewrite and the verify command
   accepts historical absolute paths as pre-migration content. The
   migration sentinel mirrors `MIGRATE_LOG_SENTINEL` (`path-relativize`).
4. **Hash-chain verification update.** `request_log::verify` learns a new
   mode: "accept absolute `file:` on lines before the `path-relativize`
   migrate entry; require relative `file:` on lines after". The happy
   path for a repo without legacy entries is unchanged.
5. **CLI surface.** `product request-log verify` continues to exit 0 on a
   clean log; it grows one new warning (W-path-absolute) if an
   unmigrated absolute path is found.
6. **Unit + integration tests.** One per: happy-path relativize, escape
   case, migrate command rewrites old entries, verify accepts the
   post-migration log, verify warns on absolute path in a fresh log.

### Out

- **Retroactive hash recomputation.** The existing hash chain is not
  rewritten; the migrate entry is the canonical record of the change.
  Alternatives (full chain rewrite with a schema-upgrade entry) are more
  disruptive and were rejected — see ADR-039 decision 2 (entry-hash is
  computed once and never mutated).
- **Path redaction beyond relativisation.** Branch names, CI identifiers,
  and `applied-by` strings are not touched. If a project wants to scrub
  those further, that is a separate feature.
- **Windows case-folding.** Normalisation is POSIX separators only; we
  do not canonicalise drive-letter case. Windows CI is not a supported
  surface today (ADR-040).
- **Renaming `.product/request-log.jsonl` historical files.** The legacy
  log file (pre-FT-042) is left alone; FT-042's own `log-path` migration
  handles the move.

---

## Commands

- `product request-log verify` — gains W-path-absolute warning for
  unmigrated entries.
- `product request-log migrate-paths` — new subcommand that writes the
  one-shot `migrate` entry and rewrites offending lines in place.

---

## Implementation notes

- **`src/request_log/paths.rs`** — new file with `path_relativize` plus
  unit tests. Must stay under the 400-line file-length budget. The first
  `//!` doc line must not contain "and" (SRP fitness test).
- **`src/request_log/append.rs`** — inject relativisation at the
  boundary. Do **not** push the repo-root argument through every caller;
  instead thread it via `ApplyEntryParams` (new field `repo_root:
  &Path`) and the other params structs.
- **`src/request_log/verify.rs`** — teach the verifier about the new
  `path-relativize` sentinel. Pre-migration lines can contain absolute
  paths without triggering a warning; post-migration lines must not.
- **`src/request_log/migrate.rs`** — extend with `rewrite_paths` that
  appends the migrate entry and rewrites offending lines. File locks are
  already acquired by the append path; re-use them.
- **`src/commands/request_log.rs`** (or wherever the subcommands live) —
  wire `migrate-paths` into `dispatch()`.
- **Back-compat for hashes.** The rewrite intentionally makes
  pre-migration entries' `canonical_for_hash()` no longer match their
  stored `entry-hash`. The verifier's "accept absolute before migrate"
  rule is how we preserve exit-0 semantics. Document this in the
  function's doc comment and in ADR-039's follow-on note.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Apply a `product request` in a fresh clone located at
   `/tmp/clone-a/product-cli` and observe every `file:` field in the new
   `requests.jsonl` entry is `docs/features/FT-XXX-….md` (no leading
   `/`, no username) (TC-623).
2. Re-clone the repo at `/home/alice/work/product-cli`, apply another
   request, and observe the `file:` values are byte-identical to the
   first clone's entry — the path no longer depends on where the repo
   lives (TC-623, companion invariant).
3. Run `product request-log migrate-paths` on a repo whose historical
   log contains absolute paths and observe:
    - A new `migrate` entry with sentinel `path-relativize` listing the
      rewritten line IDs.
    - The historical lines have `file:` values rewritten to repo-relative
      form in place.
    - `product request-log verify` exits 0 — the migrate entry is
      accepted as the authority for the rewrite (TC-624).
4. Run `product request-log verify` on a freshly-initialised repo whose
   log only has entries authored after this feature and observe no
   W-path-absolute warnings (TC-624).
5. Author a custom request that, hypothetically, writes outside
   `repo_root` (escape path): observe the log keeps the absolute path
   **and** emits W-path-absolute — this is deliberately loud because
   writes outside the repo are a bug elsewhere (TC-624).
6. Run `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` and observe all pass.

See TC-625 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Audit other log-like surfaces.** `product implement` writes
  progress logs under `.product/sessions/`; those should get the same
  treatment once they become repo-committed. Currently session logs are
  per-clone so the issue is lower-severity.
- **Full-chain rewrite tool.** If we ever want a cryptographically clean
  log (no pre-migration absolute-path residue), a one-shot `rewrite`
  command that reassigns all hashes and stamps a new genesis could be
  offered. Deliberately out of scope because it breaks external
  references to entry IDs.

---

## Description

`requests.jsonl` is committed to the repository as a tamper-evident audit of every `product request apply`. Today every `file` field inside a `request` / `result.created` / `result.changed` entry carries the **absolute path** the write landed at — e.g. `/home/hafeok/projects/product-cli/docs/features/FT-043-….md`.

That leaks the committer's username and filesystem layout into the git history every time the log is appended. Any GitHub search for `/home/USER` across the repo turns up the log. For a log whose purpose is to be shared across collaborators and mirrored into CI, this is a privacy leak, a reproducibility wart (a clone under a different path replays differently), and a noisy diff (two contributors cannot agree on the same entry because their home directories differ).

This feature normalises every path-typed field in the log to a repo-root-relative, POSIX-style path (`docs/features/FT-043-….md`). New entries are written relative; existing absolute-path entries are rewritten via a one-shot migration entry that preserves hash-chain integrity (`type: migrate`, ADR-039 decision 4).

## Functional Specification

### Inputs

- **`product request apply`** — the apply pipeline passes `ApplyEntryParams` (including a new `repo_root: &Path` field) to the log writer. Callers continue to supply absolute paths; the transform is internal.
- **`product request-log migrate-paths`** — new subcommand with no required arguments. Reads the existing `requests.jsonl`, identifies all entries containing an absolute `file:` value, and appends a `migrate` entry.
- **`product request-log verify`** — existing command. Gains a new optional `--check-paths` mode; path-absolute detection fires in the default run as well, emitting W-path-absolute.
- **Repo root** — determined from `ProductConfig::discover` at invocation time; passed into the log-append path via `ApplyEntryParams.repo_root`.

### Outputs

- **New log entries from `apply`** — every `file:` field in the appended JSONL entry is a repo-root-relative, POSIX-style path (e.g. `docs/features/FT-051-….md`). No absolute prefix, no drive letter.
- **Rewritten historical entries** — after `product request-log migrate-paths`, every historical entry that contained an absolute `file:` has its `file:` field replaced with the relative form in place. The entry-hash is **not** recomputed (ADR-039 decision 2: hashes are computed once and never mutated); the migrate entry documents the rewrite.
- **Migrate log entry** — one new entry of type `migrate` with `sentinel: "path-relativize"` listing the IDs of all rewritten entries.
- **`product request-log verify` warnings** — W-path-absolute warning (exit 2) when an unmigrated absolute path is found. No warning when all entries are relative or when absolute paths appear only in lines before the `path-relativize` migrate entry.

### State

The feature modifies one persistent artifact: `requests.jsonl`. No separate state file is introduced. The `path-relativize` migrate sentinel in the log is the persistent record of the migration having run. The verifier's awareness of this sentinel is the mechanism by which pre-migration absolute paths are accepted without triggering W-path-absolute.

### Behaviour

1. **Path relativisation on every new log entry.** `src/request_log/paths.rs::path_relativize(path, repo_root) -> String` is called for every user-visible `file:` field before serialisation. Contract:
   - If `path` is under `repo_root`, return the POSIX-joined suffix (the part after `repo_root/`).
   - If `path` is already relative, pass through unchanged (re-normalise POSIX separators on Windows).
   - If `path` escapes `repo_root` (unexpected for in-repo writes), return the original absolute string and emit W-path-absolute — deliberately loud so the regression is visible.
2. **One-shot migration.** `product request-log migrate-paths` scans every line of `requests.jsonl`. For each line whose `file:` fields contain an absolute path, it rewrites those fields to the relative form and records all rewritten line IDs in a new `migrate` entry. The command is idempotent — subsequent runs emit no additional migrate entry if no absolute paths remain.
3. **Verifier mode.** `request_log::verify` inspects each entry's `file:` fields. Lines appearing **before** the `path-relativize` migrate sentinel are permitted to contain absolute paths (pre-migration content). Lines appearing **after** that sentinel must use relative paths; any violation is W-path-absolute.
4. **CLI wiring.** `product request-log migrate-paths` is wired into the `dispatch()` match in `src/commands/request_log.rs`. `product request-log verify` continues to exit 0 on a clean log.

### Invariants

- For every log entry appended after FT-051 ships: every `file:` value starts with neither `/` nor a Windows drive letter (`C:\` etc.) and does not contain the repo-root absolute prefix (TC-623).
- Two clones of the same repo at different absolute paths, after applying identical requests, produce byte-identical `file:` values in their respective new log entries (TC-623).
- After `product request-log migrate-paths` runs, `product request-log verify` exits 0 (TC-624).
- `path_relativize` on a path that is already relative returns the input unchanged (unit test).
- The migrate entry's entry-hash is computed over the entry itself; it does not recompute hashes for the lines it documents as rewritten (ADR-039 decision 2).

### Error handling

- **Path outside repo root.** `path_relativize` cannot strip the repo-root prefix. Returns the original absolute path and emits W-path-absolute. No E-class error — an out-of-repo write is a bug elsewhere in the call stack and must not silently corrupt the log with a wrong relative path.
- **W-path-absolute.** Fired by `product request-log verify` when an unmigrated absolute-path entry is detected after the `path-relativize` sentinel (or in any entry if no sentinel exists). Exit 2; does not prevent `product graph check` from running.
- **`migrate-paths` on a repo with no historical absolute paths.** No-op. No migrate entry is written; command exits 0 with a "no entries to migrate" message.
- **Hash recomputation refused.** The migration intentionally does not recompute `entry-hash` for rewritten lines. The verifier's "accept absolute before migrate" rule is how exit-0 semantics are preserved for legacy repos.

### Boundaries

- **In scope:** `file:` fields under `request.created[].file`, `request.changed[].file`, and `result.created[].file` in log entries produced by `product request apply` and `product request undo`.
- **Out of scope:** `applied-by` strings, branch names, CI identifiers, commit SHAs, and any free-text field in the log. Those are not path-typed and are not relativised.
- **Out of scope:** Windows case-folding of drive letters. Normalisation is POSIX separators only.
- **Out of scope:** Session logs under `.product/sessions/`. Those are per-clone and not committed; the privacy concern does not apply.
- **Out of scope:** The legacy log path migration handled by FT-042. This feature's scope is path content, not file location.

## Out of scope

- **Retroactive hash recomputation.** The existing hash chain is not rewritten; the migrate entry is the canonical record of the change. Full chain rewrite (reassigning all hashes with a new genesis) is deliberately excluded because it breaks external references to entry IDs and is more disruptive than the migrate-sentinel approach (ADR-039 decision 2).
- **Path redaction beyond relativisation.** Branch names, CI identifiers, and `applied-by` strings are not touched. Scrubbing those further is a separate feature.
- **Windows case-folding.** Normalisation is POSIX separators only; drive-letter case is not canonicalised. Windows CI is not a supported surface (ADR-040).
- **Renaming `.product/request-log.jsonl` historical files.** The legacy log file (pre-FT-042) is left alone; FT-042's own `log-path` migration handles the move.
