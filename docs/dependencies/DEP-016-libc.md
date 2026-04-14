---
id: DEP-016
title: libc
type: library
source: "https://crates.io/crates/libc"
version: "0.2"
status: active
features:
  - FT-010
adrs:
  - ADR-001
availability-check: "cargo check"
breaking-change-risk: low
---

# libc

Raw FFI bindings to platform C libraries. Used in `main.rs` for SIGPIPE signal handling (`signal(SIGPIPE, SIG_DFL)`) to ensure graceful behavior when CLI output is piped to commands like `head`.
