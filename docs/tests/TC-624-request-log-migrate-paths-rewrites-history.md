---
id: TC-624
title: request_log_migrate_paths_rewrites_history
type: scenario
status: unimplemented
validates:
  features:
  - FT-051
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: "tc_624_request_log_migrate_paths_rewrites_history"
---

## Session — request-log-migrate-paths-rewrites-history

### Given

A fixture repo whose `requests.jsonl` contains three legacy entries
created before FT-051 shipped: each with `file:` values like
`/home/alice/work/product-cli/docs/features/FT-001-….md`.

### When

The user runs `product request-log migrate-paths`.

### Then

- A new `migrate` entry is appended whose `sources` list contains the IDs
  of the three rewritten lines, and whose reason mentions the
  `path-relativize` sentinel.
- The three legacy lines now carry `file:` values of the form
  `docs/features/FT-001-….md` — the absolute prefix has been stripped.
- `product request-log verify` exits 0: the migrate entry is accepted as
  the authority for the in-place rewrite, and hash mismatches on the
  rewritten lines are tolerated because of the sentinel.

### And

On a second run of `product request-log migrate-paths` with no
outstanding absolute paths, the command is a no-op: no new migrate entry
is appended (the check is: "does any pre-current-migrate line still
contain an absolute `file:` value?" — if no, exit 0, nothing written).
