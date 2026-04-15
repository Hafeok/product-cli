---
id: TC-444
title: skip-adr-check bypasses E016
type: scenario
status: unimplemented
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
---

## Description

Create a feature linked to a proposed ADR with a passing TC. Run `product verify FT-XXX --skip-adr-check`. Assert:

1. Exit code is 0.
2. No E016 in stderr.
3. Feature status is updated to `complete` despite the proposed ADR.
4. The flag is intended for migration scenarios only.