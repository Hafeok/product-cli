---
id: TC-114
title: verify_updates_frontmatter
type: scenario
status: unimplemented
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

run verify. Assert `last-run` timestamp is written to TC files. Assert `failure-message` written for failing TCs.