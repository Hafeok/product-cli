---
id: TC-441
title: verify succeeds when all linked ADRs are accepted
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_441_verify_succeeds_when_all_linked_adrs_are_accepted"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature linked to an ADR. Accept the ADR via `product adr status ADR-XXX accepted`. Add a passing TC with runner config. Run `product verify FT-XXX`. Assert:

1. Exit code is 0.
2. No E016 in stderr.
3. Feature status is updated to `complete`.
4. TC status is updated to `passing` with `last-run` timestamp.