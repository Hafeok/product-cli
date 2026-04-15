---
id: DEP-019
title: assert_cmd
type: library
source: "https://crates.io/crates/assert_cmd"
version: "2"
status: active
features:
  - FT-015
adrs:
  - ADR-018
availability-check: "cargo check"
breaking-change-risk: low
---

# assert_cmd

Command-line application testing. Dev dependency that provides `Command::cargo_bin()` for spawning the product binary in integration tests and asserting exit codes, stdout, and stderr.
