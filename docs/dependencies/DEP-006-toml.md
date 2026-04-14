---
id: DEP-006
title: toml
type: library
source: "https://crates.io/crates/toml"
version: "0.8"
status: active
features:
  - FT-002
  - FT-035
adrs:
  - ADR-002
availability-check: "cargo check"
breaking-change-risk: low
---

# toml

TOML parsing and serialization. Used to load `product.toml` repository configuration (paths, prefixes, thresholds, phases, domains, MCP settings) via `config.rs`.
