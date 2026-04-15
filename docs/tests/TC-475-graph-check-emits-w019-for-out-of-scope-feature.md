---
id: TC-475
title: graph check emits W019 for out-of-scope feature
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Given** a repository with `[product].responsibility = "A private cloud platform for Raspberry Pi"` AND a feature FT-099 titled "Grocery List Management" exists
**When** `product graph check` runs validation
**Then** stderr contains `warning[W019]: feature outside product responsibility` referencing FT-099

**Given** a repository with `[product].responsibility` set AND all features are clearly within scope
**When** `product graph check` runs
**Then** no W019 warnings are emitted