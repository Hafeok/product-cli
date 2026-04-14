---
id: DEP-009
title: chrono
type: library
source: "https://crates.io/crates/chrono"
version: "0.4"
status: active
features:
  - FT-033
  - FT-034
  - FT-022
adrs:
  - ADR-032
availability-check: "cargo check"
breaking-change-risk: low
---

# chrono

Date and time library. Generates RFC 3339 timestamps for AGENT.md generation, content hash amendment history, metrics recording, authoring sessions, verify runs, and gap analysis baselines. Used with `serde` feature.
