---
id: TC-445
title: superseded and abandoned ADRs satisfy lifecycle invariant
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_445_superseded_and_abandoned_adrs_satisfy_lifecycle_invariant"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature linked to two ADRs: one with `status: superseded`, one with `status: abandoned`. Add a passing TC. Run `product verify FT-XXX`. Assert:

1. Exit code is 0 — no E016.
2. Feature status is updated to `complete`.
3. Only `proposed` status blocks completion; `superseded` and `abandoned` satisfy the invariant.