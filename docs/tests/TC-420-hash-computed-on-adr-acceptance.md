---
id: TC-420
title: Hash computed on ADR acceptance
type: scenario
status: passing
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_420_hash_computed_on_adr_acceptance"
last-run: 2026-04-14T14:44:11.097422144+00:00
---

## Description

Create a new ADR via `product adr new`. Verify it has no `content-hash` field. Set its status to `accepted` via `product adr status ADR-XXX accepted`. Verify the file now contains a `content-hash` field with a `sha256:` prefix and 64 hex characters. Verify the hash matches a manual SHA-256 computation over the normalized body text and protected front-matter fields (title).