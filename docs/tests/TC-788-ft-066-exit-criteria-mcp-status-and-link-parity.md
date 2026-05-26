---
id: TC-788
title: ft_066_exit_criteria_mcp_status_and_link_parity
type: exit-criteria
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_788_ft_066_exit_criteria_mcp_status_and_link_parity
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.2s
---

## Description

Consolidated exit-criteria for FT-066. The feature ships when every
item below holds in the same repo at the same commit:

1. **TC-778..TC-784** all pass — status write parity is implemented
   over MCP for features and TCs, the no-op `handle_status_update`
   stub is deleted, and the legacy "Use CLI for status updates with
   full side-effects" string is absent from `src/`.
2. **TC-785..TC-787** all pass — `product_feature_link` reciprocates
   `validates.features` and ADR `features` back-references, and
   returns a structured `writes` / `reciprocated` report instead of
   the legacy `linked: bool`.
3. `cargo t` (the `--no-fail-fast` alias) reports zero failures
   across all six test binaries.
4. `cargo clippy -- -D warnings -D clippy::unwrap_used` reports zero
   warnings.
5. `cargo build` succeeds.
6. `product graph check` exits 0 after the feature lands — no new
   warnings or errors are introduced and no existing ones are
   reactivated by the MCP changes.
7. Every TC linked to FT-066 has `runner: cargo-test` and
   `runner-args` matching the Rust test function name (CLAUDE.md
   policy).
8. AGENTS.md's "Key MCP Tools" table reflects the new behaviour:
   `product_feature_status`, `product_test_status`, and
   `product_feature_link` are documented as writing through the
   slice layer with full parity to the CLI.