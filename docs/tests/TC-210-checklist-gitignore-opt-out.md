---
id: TC-210
title: checklist_gitignore_opt_out
type: scenario
status: unimplemented
validates:
  features: 
  - FT-014
  - FT-017
  adrs:
  - ADR-007
phase: 1
---

set `checklist-in-gitignore = false` in `product.toml`. Assert `checklist.md` does NOT appear in `.gitignore`.