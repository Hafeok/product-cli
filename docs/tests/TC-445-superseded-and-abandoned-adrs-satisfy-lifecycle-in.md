---
id: TC-445
title: superseded and abandoned ADRs satisfy lifecycle invariant
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Description

Create a feature linked to two ADRs: one with `status: superseded`, one with `status: abandoned`. Add a passing TC. Run `product verify FT-XXX`. Assert:

1. Exit code is 0 — no E016.
2. Feature status is updated to `complete`.
3. Only `proposed` status blocks completion; `superseded` and `abandoned` satisfy the invariant.