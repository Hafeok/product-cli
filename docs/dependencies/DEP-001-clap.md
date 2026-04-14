---
id: DEP-001
title: clap
type: library
source: "https://crates.io/crates/clap"
version: "4"
status: active
features:
  - FT-010
  - FT-014
adrs:
  - ADR-001
availability-check: "cargo check"
breaking-change-risk: medium
---

# clap

Command-line argument parser with derive macros. Provides the entire CLI interface including subcommand routing, argument parsing, help generation, and environment variable support. Used with `derive` and `env` features.
