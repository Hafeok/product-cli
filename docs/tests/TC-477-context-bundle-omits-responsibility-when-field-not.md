---
id: TC-477
title: context bundle omits responsibility when field not configured
type: scenario
status: passing
validates:
  features:
  - FT-039
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_477_context_bundle_omits_responsibility_when_field_not_configured"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.3s
---

**Given** a repository without `[product].responsibility` in product.toml AND a feature FT-001 exists
**When** `product context FT-001` assembles the bundle
**Then** the bundle header does NOT contain `product≜` or `responsibility≜` lines — the bundle format is unchanged from pre-FT-039 behavior