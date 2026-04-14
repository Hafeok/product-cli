---
id: TC-378
title: single_responsibility_passes
type: scenario
status: passing
validates:
  features:
  - FT-031
  adrs:
  - ADR-029
phase: 1
runner: cargo-test
runner-args: "tc_378_single_responsibility_passes"
last-run: 2026-04-14T16:41:17.424364011+00:00
---

all files begin with single-sentence `//!` doc comment without "and". Assert exit 0.