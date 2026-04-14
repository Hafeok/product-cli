---
id: TC-057
title: error_no_panic_on_bad_yaml
type: scenario
status: passing
validates:
  features:
  - FT-010
  adrs:
  - ADR-013
phase: 1
runner: cargo-test
runner-args: "tc_057_error_no_panic_on_bad_yaml"
last-run: 2026-04-14T13:40:28.280537041+00:00
---

feed a file with completely invalid YAML as front-matter. Assert exit code 1, structured error on stderr, no panic.