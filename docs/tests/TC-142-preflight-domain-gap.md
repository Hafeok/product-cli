---
id: TC-142
title: preflight_domain_gap
type: scenario
status: passing
validates:
  features:
  - FT-019
  - FT-027
  adrs:
  - ADR-026
phase: 1
runner: cargo-test
runner-args: "tc_142_preflight_domain_gap"
last-run: 2026-04-13T14:12:26.396687298+00:00
---

FT-009 declares `domains: [security]`, no security ADRs linked or acknowledged. Assert preflight reports security gap with the top-2 security ADRs by centrality named.