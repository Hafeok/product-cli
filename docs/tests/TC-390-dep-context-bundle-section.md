---
id: TC-390
title: dep_context_bundle_section
type: scenario
status: passing
validates:
  features:
  - FT-032
  adrs:
  - ADR-030
phase: 1
runner: cargo-test
runner-args: "tc_390_dep_context_bundle_section"
last-run: 2026-04-14T17:03:27.857859122+00:00
---

feature uses DEP-001 and DEP-005. Assert context bundle contains a "Dependencies" section with both entries, interface block included for DEP-005.