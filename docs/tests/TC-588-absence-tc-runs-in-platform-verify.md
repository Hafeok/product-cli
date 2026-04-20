---
id: TC-588
title: absence_tc_runs_in_platform_verify
type: scenario
status: passing
validates:
  features:
  - FT-047
  adrs:
  - ADR-041
phase: 1
runner: cargo-test
runner-args: tc_588_absence_tc_runs_in_platform_verify
last-run: 2026-04-20T08:01:08.284116371+00:00
last-run-duration: 0.2s
---

## Session: ST-142 — absence-tc-runs-in-platform-verify

### Given
A repository with two scenario TCs (feature-scoped) and one absence TC
(cross-cutting, validates an ADR only).

### When
`product verify --platform` is invoked.

### Then
- The absence TC is included in the platform verify run.
- The two feature-scoped scenario TCs are NOT included.
- The CI JSON output (`--ci`) lists the absence TC under stage 6
  (platform-tcs).