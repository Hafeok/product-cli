---
id: TC-464
title: adr scope validates enum values
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product adr scope ADR-XXX invalid-scope`. Assert exit code 1 and error E001. Run with each valid value: `cross-cutting`, `domain`, `feature-specific`. Assert exit code 0 for each and the `scope` field in front-matter matches the set value.