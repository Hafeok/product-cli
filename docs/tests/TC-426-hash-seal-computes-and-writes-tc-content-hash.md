---
id: TC-426
title: Hash seal computes and writes TC content-hash
type: scenario
status: passing
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_426_hash_seal_computes_and_writes_tc_content_hash"
last-run: 2026-04-14T14:44:11.097422144+00:00
---

## Description

Create a TC with body content. Verify it has no `content-hash`. Run `product hash seal TC-XXX`. Verify the file now contains `content-hash: sha256:...` matching a manual SHA-256 over normalized body + protected fields (title, type, validates.adrs). Also test `product hash seal --all-unsealed` with multiple TCs — verify all unsealed TCs get hashes and already-sealed TCs are not modified.