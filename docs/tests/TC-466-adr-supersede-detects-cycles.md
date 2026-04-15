---
id: TC-466
title: adr supersede detects cycles
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Create ADR-A, ADR-B, ADR-C. Set ADR-B supersedes ADR-A, ADR-C supersedes ADR-B. Now run `product adr supersede ADR-A --supersedes ADR-C`. Assert exit code 1 and error E004 (supersession cycle detected). Assert no files were modified.