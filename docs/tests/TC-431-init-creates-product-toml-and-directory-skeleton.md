---
id: TC-431
title: init creates product.toml and directory skeleton
type: scenario
status: passing
validates:
  features: [FT-035]
  adrs: [ADR-033]
phase: 1
runner: cargo-test
runner-args: "tc_431_init_creates_product_toml_and_directory_skeleton"
last-run: 2026-04-14T14:52:43.866547207+00:00
---

## Description

Run `product init --yes` in an empty temporary directory. Assert:

1. `product.toml` exists and contains `name`, `schema-version`, `[paths]`, `[prefixes]`, `[phases]`, `[domains]`, and `[mcp]` sections.
2. The `name` field defaults to the directory name.
3. `schema-version` equals the current `CURRENT_SCHEMA_VERSION`.
4. Directories `docs/features/`, `docs/adrs/`, `docs/tests/`, `docs/graph/` all exist.
5. Exit code is 0.
6. Stdout contains a summary of created files.