---
id: DEP-004
title: serde_json
type: library
source: "https://crates.io/crates/serde_json"
version: "1"
status: active
features:
  - FT-010
  - FT-021
  - FT-029
adrs:
  - ADR-002
  - ADR-020
availability-check: "cargo check"
breaking-change-risk: low
---

# serde_json

JSON serialization and deserialization. Used for `--format json` CLI output, metrics.jsonl export, MCP server JSON-RPC protocol, and gap analysis baselines.
