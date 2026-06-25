---
id: TC-1033
title: automation trigger must watch a view
type: invariant
status: passing
validates:
  features:
  - FT-149
  adrs:
  - ADR-091
phase: 1
runner: cargo-test
runner-args: "tc_1033_automation_trigger_must_watch_a_view"
last-run: 2026-06-25T10:21:08.278592637+00:00
last-run-duration: 0.5s
---

## Description

§3.2.0 Automation — an automated trigger that watches no View is a finding
(observe, then act). Watching a declared read model satisfies the Automation
pattern shape and emits a `pf:watches` edge in the Turtle export.