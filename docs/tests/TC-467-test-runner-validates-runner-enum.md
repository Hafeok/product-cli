---
id: TC-467
title: test runner validates runner enum
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_467_test_runner_validates_runner_enum"
last-run: 2026-04-18T10:42:03.345580667+00:00
last-run-duration: 0.2s
---

Run `product test runner TC-XXX --runner invalid-runner --args "test_name"`. Assert exit code 1 and error E001. Run with each valid runner: `cargo-test`, `bash`, `pytest`, `custom`. Assert exit code 0 for each and the `runner` field in front-matter matches.