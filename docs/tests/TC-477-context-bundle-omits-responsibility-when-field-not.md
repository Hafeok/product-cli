---
id: TC-477
title: context bundle omits responsibility when field not configured
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

**Given** a repository without `[product].responsibility` in product.toml AND a feature FT-001 exists
**When** `product context FT-001` assembles the bundle
**Then** the bundle header does NOT contain `product≜` or `responsibility≜` lines — the bundle format is unchanged from pre-FT-039 behavior