---
id: TC-159
title: FT-017 checklist generation idempotent
type: exit-criteria
status: passing
validates:
  features:
  - FT-017
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_159_checklist_generation_idempotent"
---

## Description

Running `product checklist generate` twice in succession on an unchanged graph produces byte-identical output. This validates that checklist generation is a pure function of the current graph state with no accumulated side effects.