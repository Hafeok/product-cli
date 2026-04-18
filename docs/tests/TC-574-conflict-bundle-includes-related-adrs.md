---
id: TC-574
title: conflict_bundle_includes_related_adrs
type: scenario
status: failing
validates:
  features:
  - FT-045
  adrs:
  - ADR-022
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_574_conflict_bundle_includes_related_adrs
last-run: 2026-04-18T10:42:24.877521871+00:00
last-run-duration: 0.1s
failure-message: "No matching test function found (0 tests ran)"
---

## Session: ST-131 — conflict-bundle-includes-related-adrs

**Validates:** FT-045, ADR-022 (amended), ADR-040

### Given

A temp repository with:
- A proposed ADR-031 in the `consensus` domain.
- At least two cross-cutting ADRs (`scope: cross-cutting`).
- At least one other ADR in the `consensus` domain.
- At least five other ADRs (for the top-5 by centrality).

### When

`product adr conflict-bundle ADR-031` is run.

### Then

- stdout is a markdown document with sections: `## Instructions`, `## Proposed ADR`, `## Existing ADRs to Check Against`.
- The Proposed ADR section contains the full body of ADR-031.
- The Existing ADRs section contains every cross-cutting ADR, every same-domain ADR, and the top-5 by betweenness centrality — no other ADRs.
- Each included ADR appears exactly once (sets are unioned, not duplicated).
- Exit code is `0`.
- No LLM call was made.