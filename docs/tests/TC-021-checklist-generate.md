---
id: TC-021
title: checklist_generate
type: scenario
status: passing
validates:
  features:
  - FT-017
  - FT-014
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_021_checklist_generate"
last-run: 2026-04-14T15:02:41.236412349+00:00
---

set three features to `in-progress`, `complete`, `planned`. Run `product checklist generate`. Assert the checklist contains the correct status markers and no YAML front-matter.