---
id: TC-846
title: ft_073_exit_criteria_pattern_authoring
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-073
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_846_ft_073_exit_criteria_pattern_authoring
---

## Description

Consolidated exit-criteria for FT-073:

1. **TC-839..TC-845** all pass.
2. `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, `cargo build` all green.
3. `product_prompts_list` returns `author-pattern` (version 1).
4. `product_prompts_get author-pattern` returns the registered
   prompt body.
5. `product graph check` exits 0 against a clean fixture
   exercising the new authoring affordances.
6. Every TC linked to FT-073 carries `observes:` per ADR-051
   (the F3 contract; FT-073 dogfoods even though F3 is
   independent).
7. AGENTS.md documents the `author-pattern` session prompt and
   the `--pattern` flag.

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.
