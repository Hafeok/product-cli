---
id: TC-110
title: implement_dry_run
type: scenario
status: unimplemented
validates:
  features:
  - FT-023
  adrs:
  - ADR-021
phase: 1
---

run `product implement FT-001 --dry-run`. Assert temp file is created and its path printed. Assert no agent is invoked.