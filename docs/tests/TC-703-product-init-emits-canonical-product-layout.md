---
id: TC-703
title: product_init_emits_canonical_product_layout
type: scenario
status: planned
validates:
  features:
  - FT-057
  adrs:
  - ADR-048
phase: 5
runner: cargo-test
runner-args: "tc_703_product_init_emits_canonical_product_layout"
---

**Test Type:** scenario

**Why this TC exists:**

When FT-057 first landed, the migration command (`product migrate
consolidate`) and the `ProductConfig::discover` fallback shipped, but
`product init` was not actually updated to emit the canonical layout
— it kept writing `product.toml` at the root with a `docs/...`
skeleton, contradicting the FT-057 acceptance-criterion that "a
fresh empty directory + `product init` produces `.product/config.toml`
plus the canonical skeleton". TC-703 codifies that acceptance check
so the next regression is caught by the suite, not by the user.

**Setup:**

1. Create an empty tempdir.

**Execution (default canonical layout):**

1. Run `product init --yes --name canonical-test` in the tempdir.

**Expected (default canonical layout):**

- Exit code 0.
- File `.product/config.toml` exists; root `product.toml` does **not**.
- `[paths]` in `.product/config.toml` reads:
  - `features = ".product/features"`
  - `adrs = ".product/adrs"`
  - `tests = ".product/tests"`
  - `graph = ".product/graph"`
  - `checklist = ".product/checklist.md"`
  - `dependencies = ".product/dependencies"`
  - `requests = ".product/requests.jsonl"`
  - `prompts = ".product/prompts"`
  - `gaps = ".product/gaps.json"`
- Directories exist: `.product/features/`, `.product/adrs/`,
  `.product/tests/`, `.product/graph/`. The legacy `docs/` skeleton
  is **not** created.
- `.gitignore` contains `.product/graph/` and `.product/sessions/`
  (and `.product/checklist.md` when `checklist-in-gitignore = true`).
  It does **not** contain `docs/graph/`.
- Re-running `product init --yes` without `--force` exits non-zero
  and reports that `.product/config.toml` already exists.
- `ProductConfig::discover()` from the tempdir finds the canonical
  config (round-trip via `find_config_in_dir` returns
  `.product/config.toml`).

**Execution (legacy opt-in):**

1. Create a second empty tempdir.
2. Run `product init --yes --legacy-layout --name legacy-test`.

**Expected (legacy opt-in):**

- Exit code 0.
- File `product.toml` exists at the root; `.product/config.toml`
  does **not** exist.
- `[paths]` reads the legacy values (`docs/features`, `docs/adrs`,
  `docs/tests`, `docs/graph`, `docs/checklist.md`).
- Directories exist: `docs/features/`, `docs/adrs/`, `docs/tests/`,
  `docs/graph/`. `.product/` is **not** created.
- `.gitignore` contains `docs/graph/`. It does **not** contain
  `.product/graph/` or `.product/sessions/`.

**Notes:**

- This test guards against silently shipping the migration command
  and discovery fallback while leaving `init` on the old layout —
  the failure mode that prompted TC-703.
- `--force` semantics are unchanged: with `--force`, an existing
  config at the canonical path is overwritten; without it, the
  command refuses.
