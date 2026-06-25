---
id: TC-1036
title: unreifiable aio is a recorded coverage gap
type: invariant
status: passing
validates:
  features:
  - FT-152
  adrs:
  - ADR-094
phase: 1
runner: cargo-test
runner-args: "tc_1036_unreifiable_aio_is_a_recorded_coverage_gap"
last-run: 2026-06-25T11:51:41.309096894+00:00
last-run-duration: 43.1s
---

## Description

§4.5 — an unreifiable rule must be a recorded gap: it names a real AIO, a
recognised interaction class, and a rationale. A rule with no rationale is a
silent omission and is rejected; a complete rule is captured and emits a
`pf:unreifiableIn` edge in the Turtle export.