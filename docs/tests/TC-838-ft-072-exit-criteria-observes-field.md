---
id: TC-838
title: ft_072_exit_criteria_observes_field
type: exit-criteria
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_838_ft_072_exit_criteria_observes_field
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.2s
---

## Description

Consolidated exit-criteria for FT-072. The feature ships when:

1. **TC-830..TC-837** all pass.
2. `cargo t` reports zero failures.
3. `cargo clippy -- -D warnings -D clippy::unwrap_used` is clean.
4. `cargo build` succeeds.
5. `product graph check` exits 0 against the live product-cli
   repository after FT-072 lands — confirming grandfathering
   correctly exempts the existing TC corpus.
6. The implement and author-feature prompts in `docs/prompts/`
   reference ADR-051.
7. AGENTS.md documents the new field and the allowed surface
   vocabulary.
8. Every TC linked to FT-072 carries `observes:` (the F3
   feature dogfoods the new field even before its own gate
   lands — once it lands, the gate is satisfied by construction).

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.