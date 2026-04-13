---
id: TC-019
title: context_bundle_superseded_adr
type: scenario
status: passing
validates:
  features:
  - FT-011
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_019_context_bundle_superseded_adr"
---

link a superseded ADR to a feature. Assert it appears in the bundle with a `[SUPERSEDED by ADR-XXX]` annotation.