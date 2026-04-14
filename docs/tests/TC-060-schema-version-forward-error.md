---
id: TC-060
title: schema_version_forward_error
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
runner-args: "tc_060_schema_version_forward_error"
last-run: 2026-04-14T14:25:40.415822949+00:00
---

write `schema-version = "99"` to `product.toml`. Run any command. Assert exit code 1 and error E008 with the upgrade hint.