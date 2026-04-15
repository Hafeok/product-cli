---
id: TC-476
title: W019 suppressed when responsibility field absent
type: scenario
status: passing
validates:
  features:
  - FT-039
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_476_w019_suppressed_when_responsibility_field_absent"
last-run: 2026-04-15T11:22:02.279019545+00:00
last-run-duration: 0.3s
---

**Given** a repository without `[product].responsibility` in product.toml AND features exist with any titles
**When** `product graph check` runs validation
**Then** no W019 warnings are emitted for any feature — the check is inert when responsibility is absent