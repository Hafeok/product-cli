---
id: DEP-021
title: proptest
type: library
source: "https://crates.io/crates/proptest"
version: "1"
status: active
features:
  - FT-015
  - FT-025
adrs: []
availability-check: "cargo check"
breaking-change-risk: low
---

# proptest

Property-based testing framework. Dev dependency that generates random inputs to test invariants across init TOML validity, parser robustness, graph algorithm correctness, and fileops safety.
