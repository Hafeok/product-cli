---
id: TC-863
title: ft_104_exit_criteria
type: exit-criteria
status: passing
validates:
  features:
  - FT-104
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_863_ft_104_exit_criteria
---

## Description

Consolidated exit-criteria for FT-104:

1. **TC-860..TC-862** all pass.
2. `cargo t`, `cargo clippy --lib --bin product -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` are green.
3. The `product feature reject` verb writes
   `adrs-rejected:` atomically and rejects empty reasons.
4. `product preflight --format json` exposes the four new
   statuses (`linked`, `acknowledged`, `default-acknowledged`,
   `intentional`).
5. `product graph check` emits W036/W037/W038 on the three
   drift forms, exit code stays warning-level (never 1).

## Formal specification

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩

Aggregator; omits `observes:` per ADR-051.
