---
id: TC-022
title: checklist_no_manual_edit_warning
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
runner-args: "tc_022_checklist_no_manual_edit_warning"
last-run: 2026-04-14T14:18:28.985359737+00:00
---

assert the generated checklist begins with a comment block warning against manual editing.