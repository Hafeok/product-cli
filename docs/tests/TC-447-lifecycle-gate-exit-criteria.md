---
id: TC-447
title: lifecycle gate exit criteria
type: exit-criteria
status: unimplemented
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
---

## Description

All lifecycle gate scenarios pass: E016 blocks verify on proposed ADR (TC-440), verify succeeds after acceptance (TC-441), W017 fires in graph check (TC-442), planned features are exempt (TC-443), skip flag works (TC-444), superseded/abandoned satisfy invariant (TC-445), all proposed ADRs are named (TC-446).