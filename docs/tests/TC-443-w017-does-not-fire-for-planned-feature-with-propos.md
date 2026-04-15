---
id: TC-443
title: W017 does not fire for planned feature with proposed ADR
type: scenario
status: unimplemented
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
---

## Description

Create a feature with `status: planned` linked to an ADR with `status: proposed`. Run `product graph check`. Assert:

1. No W017 warning appears for this feature.
2. Linking a proposed ADR to a planned feature is valid forward-planning, not a lifecycle violation.