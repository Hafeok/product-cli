---
id: TC-441
title: verify succeeds when all linked ADRs are accepted
type: scenario
status: unimplemented
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
---

## Description

Create a feature linked to an ADR. Accept the ADR via `product adr status ADR-XXX accepted`. Add a passing TC with runner config. Run `product verify FT-XXX`. Assert:

1. Exit code is 0.
2. No E016 in stderr.
3. Feature status is updated to `complete`.
4. TC status is updated to `passing` with `last-run` timestamp.