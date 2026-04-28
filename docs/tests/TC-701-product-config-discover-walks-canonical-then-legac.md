---
id: TC-701
title: product_config_discover_walks_canonical_then_legacy
type: scenario
status: passing
validates:
  features:
  - FT-057
  adrs:
  - ADR-048
phase: 5
runner: cargo-test
runner-args: "--test sessions tc_701a_canonical_layout_loads"
last-run: 2026-04-28T10:28:42.987969930+00:00
last-run-duration: 0.2s
---

**Test Type:** scenario

**Setup:**

Three sibling tempdir repos:

- **Repo A (canonical):** `.product/config.toml` only.
- **Repo B (alias):** `.product/product.toml` only (named with
  the legacy filename inside the canonical directory).
- **Repo C (legacy):** `product.toml` at root only.

Each repo has the minimal config to run `product feature list`
(empty graph is fine).

**Execution:**

For each repo:

1. `cd` into the repo (or a subdirectory of it).
2. Run `product feature list`.
3. Capture exit code and stdout.

**Expected:**

- All three runs exit 0 with the same (empty) feature listing.
- `ProductConfig::discover` returns the path of the matched
  config file (assertable via a unit test or via
  `product config show --path` if that subcommand exists).

**Precedence test:**

- Set up a fourth repo (Repo D) containing **both**
  `.product/config.toml` AND `product.toml` at root, with
  deliberately different `name` fields.
- Run `product feature list` and a `name`-revealing command.
- Expect the canonical config (the `.product/config.toml`
  `name`) to win, never the root one.

**Walk-up test:**

- From a deep subdirectory of Repo A
  (e.g. `<repo>/src/commands/`), running `product feature list`
  succeeds — discovery walks up until it finds
  `.product/config.toml`.