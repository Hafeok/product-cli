---
id: TC-605
title: custom_type_valid_when_in_toml
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_605_custom_type_valid_when_in_toml"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Session: ST-184 — custom-type-valid-when-in-toml

### Given
A repository with `product.toml` containing `[tc-types].custom = ["contract"]`
and a TC declaring `type: contract`.

### When
`product graph check` and `product context` are invoked.

### Then
- Graph check exits 0 (no E006).
- The TC appears in the context bundle for its linked feature.
- The TC's status is tracked in front-matter as for any other TC.