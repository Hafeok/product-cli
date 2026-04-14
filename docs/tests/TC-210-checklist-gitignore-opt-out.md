---
id: TC-210
title: checklist_gitignore_opt_out
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
runner-args: "tc_210_checklist_gitignore_opt_out"
last-run: 2026-04-14T15:02:41.236412349+00:00
---

set `checklist-in-gitignore = false` in `product.toml`. Assert `checklist.md` does NOT appear in `.gitignore`.