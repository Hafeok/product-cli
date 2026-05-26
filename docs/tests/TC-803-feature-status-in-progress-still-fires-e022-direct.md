---
id: TC-803
title: feature status in-progress still fires E022 directly
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_803_feature_status_in_progress_still_fires_e022
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Scenario

A feature `FT-XXX` is `planned` and has one linked TC with no
runner config. The user invokes the **status-transition** gate
directly, not through `product implement`.

## When

The user runs `product feature status FT-XXX in-progress`.

## Then

The status transition gate fires `error[E022]: TC runner
configuration missing` naming the offending TC, exactly as it does
today (pre-FT-068). The feature's status on disk remains
`planned`. The exit code is 22.

This proves the auto-fill from FT-068 is scoped exclusively to
`product implement`. The other four enforcement gates
(`feature status`, `preflight` when invoked directly, `request
apply`, `graph check`, `verify`) remain **strict** — they refuse to
auto-fill, because their callers are not in the implement
pipeline's chicken-and-egg situation. The five-gate defense-in-
depth design from FT-058 / ADR-021 is preserved.