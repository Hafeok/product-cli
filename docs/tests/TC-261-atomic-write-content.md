---
id: TC-261
title: atomic_write_content
type: scenario
status: unimplemented
validates:
  features: 
  - FT-004
  - FT-005
  adrs:
  - ADR-015
phase: 1
---

write a feature file atomically. Assert the file contains the expected content. Assert no `.product-tmp.*` files remain.