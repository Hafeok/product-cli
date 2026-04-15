---
id: TC-446
title: E016 names all proposed ADRs not just the first
type: scenario
status: unimplemented
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
---

## Description

Create a feature linked to two ADRs, both with `status: proposed`. Run `product verify FT-XXX`. Assert:

1. Exit code is 1.
2. Stderr contains `error[E016]` listing both ADR IDs (not just the first one found).
3. The developer sees the full scope of the problem in a single verify run.