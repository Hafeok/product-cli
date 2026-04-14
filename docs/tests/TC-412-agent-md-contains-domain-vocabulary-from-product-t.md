---
id: TC-412
title: AGENT.md contains domain vocabulary from product.toml
type: scenario
status: passing
validates:
  features:
  - FT-033
  adrs:
  - ADR-031
phase: 3
runner: cargo-test
runner-args: "tc_412_agent_md_contains_domain_vocabulary_from_product_toml"
last-run: 2026-04-14T17:21:07.545864789+00:00
---

## Description

Run `product agent-init`. Assert the "Domain Vocabulary" section lists all domain keys from `[domains]` in `product.toml`. Add a new domain to `product.toml`, re-run `product agent-init`, assert the new domain appears.