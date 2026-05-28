---
id: TC-862
title: default_ack_drift_emits_warnings
type: scenario
status: passing
validates:
  features:
  - FT-104
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_862_default_ack_drift_emits_warnings
observes:
- stdout
- exit-code
---

## Description

`[features].default-acknowledged-cross-cutting` lives in
`product.toml`, the rejected ADRs live in feature frontmatter,
and the ADRs themselves live in `docs/adrs/`. Each is editable
independently, so the three can drift out of agreement. This TC
asserts that `product graph check` surfaces all three drift
forms as warnings (not errors) without masking each other:

- **W036** — the default-ack list names an ADR that no longer
  exists on disk.
- **W037** — the default-ack list names an ADR whose scope has
  changed away from `cross-cutting`.
- **W038** — a feature's `adrs-rejected:` names an ADR that is
  not in the default-ack list (the rejection has no effect and
  is almost certainly a leftover).

Drift findings stay warnings — never errors — so `graph check`
exits 0 (or 2 with warnings) regardless. Each warning names the
offending id so the operator can fix one without re-running.

## Formal specification

⟦Λ:Scenario⟧
Given a repo with three cross-cutting ADRs (ADR-001..003) in
the default-ack list and a feature FT-001 that rejects
ADR-001 and a stray ADR-STRAY,
When ADR-002 is removed from disk,
And ADR-003 is rescoped from `cross-cutting` to `domain`,
And `product graph check` is run,
Then the combined output contains W036 naming ADR-002,
And contains W037 naming ADR-003,
And contains W038 naming ADR-STRAY,
And the exit code is not 1 (warnings only).

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
