---
id: TC-853
title: ft_074_exit_criteria_implement_patterns_and_observes
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-074
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_853_ft_074_exit_criteria_implement_patterns_and_observes
---

## Description

Consolidated exit-criteria for FT-074:

1. **TC-847..TC-852** all pass.
2. `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, `cargo build` all green.
3. The implement prompt version under `docs/prompts/` is bumped
   to v2 with the new reference.
4. Every TC linked to FT-074 carries `observes:` per ADR-051.
5. `product implement FT-074 --dry-run` against this very
   feature's complete state produces a bundle containing
   patterns (any seed PATs from F6), observes inline lines, and
   the hard-constraint line (full dogfood).
6. AGENTS.md documents the implement bundle enhancement.

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.
