---
id: DEP-012
title: fd-lock
type: library
source: "https://crates.io/crates/fd-lock"
version: "4"
status: active
features:
  - FT-031
adrs:
  - ADR-001
availability-check: "cargo check"
breaking-change-risk: low
---

# fd-lock

Cross-platform advisory file descriptor locking. Provides file-level locking for atomic write operations in `fileops.rs` to prevent concurrent write corruption.
