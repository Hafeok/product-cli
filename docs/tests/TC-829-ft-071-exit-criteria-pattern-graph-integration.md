---
id: TC-829
title: ft_071_exit_criteria_pattern_graph_integration
type: exit-criteria
status: passing
validates:
  features:
  - FT-071
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_829_ft_071_exit_criteria_pattern_graph_integration
last-run: 2026-05-27T13:37:22.762619987+00:00
last-run-duration: 0.3s
---

## Description

Consolidated exit-criteria for FT-071. The feature ships when:

1. **TC-820..TC-828** all pass.
2. `cargo t` reports zero failures across all binaries.
3. `cargo clippy -- -D warnings -D clippy::unwrap_used` is clean.
4. `cargo build` succeeds.
5. `product graph check` exits 0 against a clean fixture
   containing live patterns with full body sections and no cycles.
6. Every TC linked to FT-071 carries `runner: cargo-test`,
   `runner-args: tc_NNN_<snake_case>`, and `observes:` per
   ADR-051.
7. The new error / warning codes are documented in the agent
   context (regenerated via `product agent-init`), alongside the
   existing E-codes and W-codes.
8. The `bundle.patterns` metric is documented in the schema
   surface.

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.