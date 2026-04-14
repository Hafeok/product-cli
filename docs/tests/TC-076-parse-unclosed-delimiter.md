---
id: TC-076
title: parse_unclosed_delimiter
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-004
  - FT-015
  adrs:
  - ADR-016
phase: 1
runner: cargo-test
runner-args: "tc_076_parse_unclosed_delimiter"
last-run: 2026-04-14T10:46:07.489682314+00:00
---

parse a file with `⟦Γ:Invariants⟧{ ... ` (no closing `}`). Assert E001 with line number. Assert subsequent blocks in the same file are still parsed.