---
id: TC-564
title: gap_bundle_changed_scopes_correctly
type: scenario
status: passing
validates:
  features:
  - FT-045
  adrs:
  - ADR-019
  - ADR-040
phase: 1
runner: cargo-test
runner-args: tc_564_gap_bundle_changed_scopes_correctly
last-run: 2026-04-18T14:07:23.114790413+00:00
last-run-duration: 0.2s
---

## Session: ST-121 — gap-bundle-changed-scopes-correctly

**Validates:** FT-045, ADR-019 (amended), ADR-040

### Given

A temp repository initialised as a git repo with three ADRs: ADR-001, ADR-002, ADR-003. The previous commit contains ADR-001 and ADR-002 only. The current working tree added ADR-003 and modified ADR-002.

### When

`product gap bundle --changed` is run.

### Then

- The output is a concatenation of bundles (or a newline-separated multi-document) that includes bundles for **ADR-002** (modified) and **ADR-003** (added) and their 1-hop ADR neighbours.
- ADR-001 is only included if it shares a feature with ADR-002 or ADR-003 (per the expansion rule in ADR-019).
- ADRs untouched and unrelated to the change set are not included.
- Exit code is `0`.