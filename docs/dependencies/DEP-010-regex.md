---
id: DEP-010
title: regex
type: library
source: "https://crates.io/crates/regex"
version: "1"
status: active
features:
  - FT-003
  - FT-020
  - FT-005
adrs:
  - ADR-005
availability-check: "cargo check"
breaking-change-risk: low
---

# regex

Regular expression engine. Used for artifact ID format validation (`^[A-Z]+-\d{3,}$`), markdown heading and phase detection in migration helpers, and formal specification block parsing.
