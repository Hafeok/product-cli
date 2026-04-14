---
id: DEP-005
title: serde_yaml
type: library
source: "https://crates.io/crates/serde_yaml"
version: "0.9"
status: active
features:
  - FT-003
  - FT-004
adrs:
  - ADR-002
availability-check: "cargo check"
breaking-change-risk: medium
---

# serde_yaml

YAML parsing and generation. Core dependency for reading and writing YAML front-matter in all artifact markdown files (features, ADRs, test criteria, dependencies). The front-matter parser in `parser.rs` depends on this crate.
