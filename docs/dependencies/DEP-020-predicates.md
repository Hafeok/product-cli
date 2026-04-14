---
id: DEP-020
title: predicates
type: library
source: "https://crates.io/crates/predicates"
version: "3"
status: active
features:
  - FT-015
adrs: []
availability-check: "cargo check"
breaking-change-risk: low
---

# predicates

Composable assertion predicates. Dev dependency used with assert_cmd for expressive output matching in integration tests (e.g., `predicate::str::contains()`).
