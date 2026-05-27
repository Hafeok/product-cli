---
id: TC-859
title: ft_075_exit_criteria_seed_pattern_catalog
type: exit-criteria
status: passing
validates:
  features:
  - FT-075
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_859_ft_075_exit_criteria_seed_pattern_catalog
last-run: 2026-05-27T15:36:27.790359954+00:00
last-run-duration: 7.2s
---

## Description

Consolidated exit-criteria for FT-075:

1. **TC-854..TC-858** all pass.
2. `cargo t`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, `cargo build` all green.
3. `product pattern list` returns the three seeds.
4. `product graph check` exits 0 against the post-seed repo.
5. `product context FT-066 --depth 1` includes seeds in topo
   order.
6. `product graph central --include patterns` includes the
   three seeds in its ranking.
7. Every TC linked to FT-075 carries `observes:` per ADR-051.

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.