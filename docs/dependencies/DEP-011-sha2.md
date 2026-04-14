---
id: DEP-011
title: sha2
type: library
source: "https://crates.io/crates/sha2"
version: "0.10"
status: active
features:
  - FT-034
adrs:
  - ADR-032
availability-check: "cargo check"
breaking-change-risk: low
---

# sha2

SHA-256 hashing. Computes content hashes for accepted ADR immutability enforcement, test criterion sealing, and gap finding ID generation via `hash.rs`.
