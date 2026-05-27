---
id: TC-819
title: ft_070_exit_criteria_pattern_crud_parity
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-070
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_819_ft_070_exit_criteria_pattern_crud_parity
---

## Description

Consolidated exit-criteria for FT-070. The feature ships when every
item below holds at the same commit:

1. **TC-812..TC-818** all pass — pattern CRUD plumbing is in place,
   bidirectional `examples:` ↔ `feature.patterns:` materialisation
   works, MCP / CLI parity holds.
2. `cargo t` (the `--no-fail-fast` alias) reports zero failures
   across all six test binaries.
3. `cargo clippy -- -D warnings -D clippy::unwrap_used` reports zero
   warnings.
4. `cargo build` succeeds.
5. `product graph check` exits 0 after the feature lands — no new
   errors or warnings introduced.
6. Every TC linked to FT-070 has `runner: cargo-test`,
   `runner-args: tc_NNN_<snake_case>`, and a non-empty `observes:`
   list (CLAUDE.md + ADR-051 policy; FT-070 dogfoods the observes
   field even though F3 is the feature that enforces it).
7. The string "stub" and the FT-066 anti-pattern advisory text
   ("Use CLI for ...") are absent from `src/pattern/` and
   `src/mcp/` files added by this feature.
8. `product.toml` carries `[paths].patterns` and
   `[prefixes].pattern` after `product init` runs against a fresh
   directory (and the existing repo's `product.toml` is updated by
   the request-apply that lands this feature).
9. AGENTS.md's "Front-Matter Schemas" section documents the pattern
   schema and the `feature.patterns` field addition.

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

This TC is an aggregator (ADR-042); it does not itself observe a
surface and therefore omits `observes:` per ADR-051.
