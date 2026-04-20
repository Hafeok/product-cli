---
id: TC-611
title: e017_fires_at_startup_not_lazily
type: scenario
status: passing
validates:
  features:
  - FT-048
  adrs:
  - ADR-042
phase: 1
runner: cargo-test
runner-args: "tc_611_e017_fires_at_startup_not_lazily"
last-run: 2026-04-20T08:00:23.695686320+00:00
last-run-duration: 0.1s
---

## Session: ST-190 — e017-fires-at-startup-not-lazily

### Given
A repository whose `product.toml` contains `[tc-types].custom = ["invariant"]`
and whose `docs/` directory contains zero TCs.

### When
`product --help` is invoked.

### Then
- Product exits 1 with E017 before printing help text.
- The behaviour is identical for `product feature list`, `product
  graph check`, and any other subcommand.
- No file I/O against `docs/` occurs (verified by strace or by the absence
  of a parse-time log line).