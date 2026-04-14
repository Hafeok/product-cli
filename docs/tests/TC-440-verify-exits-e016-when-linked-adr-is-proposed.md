---
id: TC-440
title: verify exits E016 when linked ADR is proposed
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

## Description

Create a feature linked to a proposed ADR. Add a TC with `runner: cargo-test` that passes. Run `product verify FT-XXX`. Assert:

1. Exit code is 1.
2. Stderr contains `error[E016]` with the ADR ID and its `proposed` status.
3. Feature status is unchanged (not promoted to `complete`).
4. No TCs were executed (verify stops before running tests).