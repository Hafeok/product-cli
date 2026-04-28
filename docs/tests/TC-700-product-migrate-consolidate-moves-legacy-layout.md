---
id: TC-700
title: product_migrate_consolidate_moves_legacy_layout
type: scenario
status: passing
validates:
  features:
  - FT-057
  adrs:
  - ADR-048
phase: 5
runner: cargo-test
runner-args: "--test sessions tc_700_product_migrate_consolidate_moves_legacy_layout"
last-run: 2026-04-28T10:28:42.987969930+00:00
last-run-duration: 17.5s
---

**Test Type:** scenario

**Setup:**

1. Create a tempdir laid out in the legacy form: `product.toml`
   at root with default `[paths]`, `docs/features/FT-001-*.md`,
   `docs/adrs/ADR-001-*.md`, `docs/tests/TC-001-*.md`,
   `gaps.json` at root, `requests.jsonl` at root,
   `benchmarks/prompts/implement-v1.md`.
2. Initialize git so `product migrate consolidate` can run its
   dirty-tree guard. No staged or unstaged changes.

**Execution (dry-run):**

1. Run `product migrate consolidate` (no `--apply`).
2. Capture stdout.

**Expected (dry-run):**

- Output lists every planned move (legacy → canonical) and
  every `[paths]` rewrite.
- Process exits 0.
- Filesystem unchanged: `product.toml`, `docs/`, `gaps.json`,
  `requests.jsonl`, `benchmarks/` all still present at their
  legacy paths. `.product/` either does not exist or contains
  only `sessions/` (legacy state allowed there).

**Execution (apply):**

1. Run `product migrate consolidate --apply`.
2. Inspect the filesystem and the rewritten config.

**Expected (apply):**

- `.product/config.toml` exists; old `product.toml` is gone.
- `.product/{features,adrs,tests,dependencies,graph}/` exist
  with the migrated artifacts; `docs/features/` etc. are gone
  (or at least empty).
- `.product/prompts/implement-v1.md` exists; old
  `benchmarks/prompts/` is gone.
- `.product/{gaps.json,requests.jsonl}` exist; old root copies
  gone.
- `[paths]` in `.product/config.toml` matches the new defaults
  (or the user's explicit overrides if any were configured).
- `.gitignore` contains `.product/graph/` and
  `.product/sessions/`.
- `.product/requests.jsonl` ends with one new `migrate` entry,
  sentinel `consolidate-paths`, listing every path moved.
- `product graph check` exits 0.
- `product feature list` returns the same FT-IDs as before the
  migration.

**Idempotency:**

- Re-running `product migrate consolidate --apply` on the
  already-migrated repo succeeds with output indicating no
  moves needed. No new log entry. No filesystem changes.

**Dirty-tree guard:**

- Re-set up the legacy layout, edit one file in
  `docs/features/` without committing, run
  `product migrate consolidate --apply`, expect non-zero exit
  and an error referring to uncommitted changes. Re-run with
  `--force-uncommitted` and expect success.