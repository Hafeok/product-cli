---
id: TC-018
title: context_bundle_header
type: scenario
status: passing
validates:
  features:
  - FT-011
  adrs:
  - ADR-006
phase: 1
runner: cargo-test
runner-args: "tc_018_context_bundle_header"
last-run: 2026-04-14T13:57:28.405167723+00:00
---

assert the context bundle header block contains the correct feature ID, phase, status, and linked artifact ID lists.