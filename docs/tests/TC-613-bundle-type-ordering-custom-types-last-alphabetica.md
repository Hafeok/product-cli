---
id: TC-613
title: bundle_type_ordering_custom_types_last_alphabetical
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_613_bundle_type_ordering_custom_types_last_alphabetical"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Session: ST-192 — bundle-type-ordering-custom-types-last-alphabetical

### Given
A feature with two scenario TCs and three custom-type TCs of types
`migration`, `contract`, `smoke` (declared in `[tc-types].custom`).

### When
`product context FT-XXX` is invoked.

### Then
- All scenario TCs precede all custom-type TCs in the rendered bundle.
- Custom-type TCs appear in alphabetical order:
  `contract → migration → smoke`.
- Removing one custom type from `product.toml` does not reorder the others
  (verified by re-running with `[tc-types].custom = ["contract", "smoke"]`).