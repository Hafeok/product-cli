---
id: TC-119
title: adr_review_structural_missing_section
type: scenario
status: passing
validates:
  features:
  - FT-022
  adrs:
  - ADR-022
phase: 1
runner: cargo-test
runner-args: "tc_119_adr_review_structural_missing_section"
last-run: 2026-04-18T10:41:48.879855342+00:00
last-run-duration: 0.2s
---

review an ADR missing the Rejected alternatives section. Assert finding printed with file path and section name.