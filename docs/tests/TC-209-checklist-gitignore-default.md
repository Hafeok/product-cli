---
id: TC-209
title: checklist_gitignore_default
type: scenario
status: passing
validates:
  features: 
  - FT-014
  - FT-017
  adrs:
  - ADR-007
phase: 1
runner: cargo-test
runner-args: "tc_209_checklist_gitignore_default"
last-run: 2026-04-14T15:02:41.236412349+00:00
---

run `product init` on a new repository. Assert `checklist.md` appears in `.gitignore` by default.