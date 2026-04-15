---
id: DEP-018
title: tempfile
type: library
source: "https://crates.io/crates/tempfile"
version: "3"
status: active
features:
  - FT-015
adrs:
  - ADR-018
availability-check: "cargo check"
breaking-change-risk: low
---

# tempfile

Temporary file and directory creation. Dev dependency used in integration tests to create isolated temporary directories for test fixtures without side effects.
