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
last-run: 2026-04-14T13:57:28.405167723+00:00
---

link a superseded ADR to a feature. Assert it appears in the bundle with a `[SUPERSEDED by ADR-XXX]` annotation.