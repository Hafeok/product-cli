---
id: TC-860
title: default_acknowledge_clears_preflight_gap
type: scenario
status: passing
validates:
  features:
  - FT-104
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_860_default_acknowledge_clears_preflight_gap
observes:
- file
- exit-code
- stdout
---

## Description

`[features].default-acknowledged-cross-cutting` in `product.toml`
lets the operator declare a set of cross-cutting ADRs that every
feature is presumed to acknowledge by default. This TC verifies:

1. A feature with no explicit link to a cross-cutting ADR fails
   preflight (exit 1, "NOT COVERED").
2. After adding the ADR to the default-ack list, preflight passes
   (exit 0) and the rendered row reads `default-acknowledged`.
3. The feature's frontmatter is **not** mutated — the
   acknowledgement is config-driven, not file-driven.
4. Removing the entry restores the original gap.

The config-driven path matters because hand-editing every feature
to acknowledge a near-universal cross-cutting ADR (e.g.
`error-handling`) creates noise; one config line scales to the
whole repo.

## Formal specification

⟦Λ:Scenario⟧
Given a repo with cross-cutting ADR-001 and feature FT-001
that does not link it,
When `product preflight FT-001` is run with no default-ack
list,
Then exit code is 1 and "NOT COVERED" is rendered.
When `[features].default-acknowledged-cross-cutting = ["ADR-001"]`
is added to `product.toml`,
Then `product preflight FT-001` exits 0 and renders
"default-acknowledged",
And FT-001's frontmatter is unchanged on disk,
And removing the entry restores exit 1.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩
