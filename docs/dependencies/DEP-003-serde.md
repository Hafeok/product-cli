---
id: DEP-003
title: serde
type: library
source: "https://crates.io/crates/serde"
version: "1"
status: active
features:
  - FT-003
  - FT-011
adrs:
  - ADR-002
availability-check: "cargo check"
breaking-change-risk: low
---

# serde

Serialization and deserialization framework. Provides `Serialize` and `Deserialize` derive macros used on all artifact types (Feature, Adr, TestCriterion, Dependency, ProductConfig). Used with `derive` feature.
