---
id: TC-835
title: tc_observes_custom_surface_accepted_via_config
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_835_tc_observes_custom_surface_accepted_via_config
observes:
- file
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo whose `product.toml` file declares
`[tc-observability].custom = ["my_custom_surface"]`. Author a
scenario TC file at phase 5 declaring `observes:
[my_custom_surface]` with body text referencing the surface
name. Run `product graph check` against the on-disk file.

Assert:

1. The command exits 0 — the custom surface is accepted.
2. Writing `observes: [my_custom_surface]` through
   `product_request_apply` succeeds (no E026).
3. Removing the entry from `[tc-observability].custom` and
   re-running `graph check` causes the custom value to be
   rejected (regression in both directions).

## Formal specification

⟦Λ:Scenario⟧
Given `[tc-observability].custom = ["my_custom_surface"]` and a
  TC declaring `observes: [my_custom_surface]`,
When the user runs `product graph check`,
Then the command exits 0,
And submitting the same `observes:` value through
  `product_request_apply` succeeds,
And removing the custom config entry causes the value to be
  rejected.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩