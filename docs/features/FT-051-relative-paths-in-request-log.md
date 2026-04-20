---
id: FT-051
title: Relative Paths in the Request Log
phase: 5
status: planned
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
- security
- observability
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
