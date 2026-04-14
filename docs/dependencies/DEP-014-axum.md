---
id: DEP-014
title: axum
type: library
source: "https://crates.io/crates/axum"
version: "0.7"
status: active
features:
  - FT-021
adrs:
  - ADR-020
availability-check: "cargo check"
breaking-change-risk: medium
---

# axum

Web framework built on tokio and tower. Provides HTTP routing, JSON extraction, and request/response handling for the MCP HTTP server transport. Used with `macros` feature.
