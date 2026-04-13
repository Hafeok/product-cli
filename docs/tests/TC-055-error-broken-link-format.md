---
id: TC-055
title: error_broken_link_format
type: scenario
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
---

parse a feature with a broken ADR reference. Assert stderr contains the file path, line number, offending content, and a hint. Assert stdout is empty. Assert exit code 1.