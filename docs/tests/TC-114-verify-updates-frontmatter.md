---
id: TC-114
title: verify_updates_frontmatter
type: scenario
status: passing
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
runner: cargo-test
runner-args: tc_114_verify_updates_frontmatter
last-run: 2026-04-13T14:07:16.920985096+00:00
---

run verify. Assert `last-run` timestamp is written to TC files. Assert `failure-message` written for failing TCs.