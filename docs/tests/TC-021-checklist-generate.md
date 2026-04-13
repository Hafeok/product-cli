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
---

set three features to `in-progress`, `complete`, `planned`. Run `product checklist generate`. Assert the checklist contains the correct status markers and no YAML front-matter.