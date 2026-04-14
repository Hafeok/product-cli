---
id: DEP-015
title: tower-http
type: library
source: "https://crates.io/crates/tower-http"
version: "0.5"
status: active
features:
  - FT-021
adrs:
  - ADR-020
availability-check: "cargo check"
breaking-change-risk: low
---

# tower-http

HTTP middleware layer for tower/axum. Provides `CorsLayer` for cross-origin request support on the MCP HTTP server. Used with `cors` feature.
