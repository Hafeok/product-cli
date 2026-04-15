---
id: TC-467
title: test runner validates runner enum
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Run `product test runner TC-XXX --runner invalid-runner --args "test_name"`. Assert exit code 1 and error E001. Run with each valid runner: `cargo-test`, `bash`, `pytest`, `custom`. Assert exit code 0 for each and the `runner` field in front-matter matches.