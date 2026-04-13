---
id: TC-081
title: title
type: scenario
status: passing
validates:
  features:
  - FT-020
  adrs:
  - ADR-017
phase: 1
runner: cargo-test
runner-args: "tc_081_title"
---

Migration strips leading numbers from headings ("5. Products and IAM" becomes "Products and IAM") and preserves plain titles as-is.