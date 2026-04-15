---
id: TC-440
title: verify exits E016 when linked ADR is proposed
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_440_verify_exits_e016_when_linked_adr_is_proposed"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature linked to a proposed ADR. Add a TC with `runner: cargo-test` that passes. Run `product verify FT-XXX`. Assert:

1. Exit code is 1.
2. Stderr contains `error[E016]` with the ADR ID and its `proposed` status.
3. Feature status is unchanged (not promoted to `complete`).
4. No TCs were executed (verify stops before running tests).