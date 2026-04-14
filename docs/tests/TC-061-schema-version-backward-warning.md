---
id: TC-061
title: schema_version_backward_warning
type: scenario
status: passing
validates:
  features:
  - FT-003
  - FT-008
  - FT-020
  adrs:
  - ADR-014
phase: 1
runner: cargo-test
runner-args: "tc_061_schema_version_backward_warning"
last-run: 2026-04-14T14:25:40.415822949+00:00
---

write `schema-version = "0"` to `product.toml` (simulating an old repo). Run `product graph check`. Assert W007 appears on stderr and the command still completes successfully.