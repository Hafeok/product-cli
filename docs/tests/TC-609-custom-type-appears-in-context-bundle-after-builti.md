---
id: TC-609
title: custom_type_appears_in_context_bundle_after_builtins
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_609_custom_type_appears_in_context_bundle_after_builtins"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.2s
---

## Session: ST-188 — custom-type-appears-in-context-bundle-after-builtins

### Given
A feature with one TC of every category: an `exit-criteria`, an `invariant`,
a `chaos`, a `scenario`, a `benchmark`, and one custom-type TC (`contract`).

### When
`product context FT-XXX` is invoked.

### Then
- The TCs in the rendered bundle appear in this exact order:
  `exit-criteria → invariant → chaos → scenario → benchmark → contract`.
- All built-in types precede the custom type regardless of TC ID order in
  the front-matter.