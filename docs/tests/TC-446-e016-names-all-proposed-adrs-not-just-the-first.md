---
id: TC-446
title: E016 names all proposed ADRs not just the first
type: scenario
status: passing
validates:
  features: [FT-036]
  adrs: [ADR-034]
phase: 1
runner: cargo-test
runner-args: "tc_446_e016_names_all_proposed_adrs_not_just_the_first"
last-run: 2026-04-15T10:35:59.328815871+00:00
last-run-duration: 0.2s
---

## Description

Create a feature linked to two ADRs, both with `status: proposed`. Run `product verify FT-XXX`. Assert:

1. Exit code is 1.
2. Stderr contains `error[E016]` listing both ADR IDs (not just the first one found).
3. The developer sees the full scope of the problem in a single verify run.