---
id: TC-135
title: acknowledgement_requires_reason
type: scenario
status: passing
validates:
  features:
  - FT-018
  - FT-019
  adrs:
  - ADR-025
phase: 1
runner: cargo-test
runner-args: "tc_135_acknowledgement_requires_reason"
last-run: 2026-04-30T09:22:52.644540918+00:00
last-run-duration: 0.3s
---

feature front-matter has `domains-acknowledged: { security: "" }`. Assert E011 with file path and field name.