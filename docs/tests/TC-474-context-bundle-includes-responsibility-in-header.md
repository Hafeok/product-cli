---
id: TC-474
title: context bundle includes responsibility in header
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Given** a repository with `[product].responsibility` set AND a feature FT-001 exists
**When** `product context FT-001` assembles the bundle
**Then** the bundle header contains `product≜<name>:Product` AND `responsibility≜"<statement>"` lines before the `feature≜` line

**Given** a repository with `[product].responsibility` set
**When** `bundle_feature()` generates the AISP header
**Then** the `product` and `responsibility` fields appear as the first two lines inside the bundle header block