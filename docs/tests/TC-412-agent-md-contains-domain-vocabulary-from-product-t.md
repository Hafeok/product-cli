---
id: TC-412
title: AGENT.md contains domain vocabulary from product.toml
type: scenario
status: unimplemented
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
---

## Description

Run `product agent-init`. Assert the "Domain Vocabulary" section lists all domain keys from `[domains]` in `product.toml`. Add a new domain to `product.toml`, re-run `product agent-init`, assert the new domain appears.
