---
id: DEP-013
title: tokio
type: library
source: "https://crates.io/crates/tokio"
version: "1"
status: active
features:
  - FT-021
adrs:
  - ADR-020
availability-check: "cargo check"
breaking-change-risk: medium
---

# tokio

Asynchronous runtime for Rust. Powers the async HTTP transport for the MCP server. Used with `full` feature for complete async I/O, timers, and task spawning.
