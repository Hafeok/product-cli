---
id: TC-474
title: context bundle includes responsibility in header
type: scenario
status: passing
validates:
  features:
  - FT-039
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_474_context_bundle_includes_responsibility_in_header"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.3s
---

**Given** a repository with `[product].responsibility` set AND a feature FT-001 exists
**When** `product context FT-001` assembles the bundle
**Then** the bundle header contains `product≜<name>:Product` AND `responsibility≜"<statement>"` lines before the `feature≜` line

**Given** a repository with `[product].responsibility` set
**When** `bundle_feature()` generates the AISP header
**Then** the `product` and `responsibility` fields appear as the first two lines inside the bundle header block