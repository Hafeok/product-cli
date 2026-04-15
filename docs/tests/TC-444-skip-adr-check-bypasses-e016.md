---
id: TC-444
title: skip-adr-check bypasses E016
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_444_skip_adr_check_bypasses_e016"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature linked to a proposed ADR with a passing TC. Run `product verify FT-XXX --skip-adr-check`. Assert:

1. Exit code is 0.
2. No E016 in stderr.
3. Feature status is updated to `complete` despite the proposed ADR.
4. The flag is intended for migration scenarios only.