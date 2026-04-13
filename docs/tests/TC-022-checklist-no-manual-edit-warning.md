---
id: TC-022
title: checklist_no_manual_edit_warning
type: scenario
status: passing
validates:
  features:
  - FT-017
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_022_checklist_no_manual_edit_warning"
---

assert the generated checklist begins with a comment block warning against manual editing.