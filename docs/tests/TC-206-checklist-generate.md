---
id: TC-206
title: checklist_generate
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

set three features to `in-progress`, `complete`, `planned`. Run `product checklist generate`. Assert the checklist contains the correct status markers and no YAML front-matter.