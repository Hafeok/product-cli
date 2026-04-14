---
id: DEP-002
title: clap_complete
type: library
source: "https://crates.io/crates/clap_complete"
version: "4"
status: active
features:
  - FT-010
adrs:
  - ADR-001
availability-check: "cargo check"
breaking-change-risk: low
---

# clap_complete

Shell completion generator for clap-based CLIs. Generates bash, zsh, and fish completions via the `product completions` subcommand.
