---
id: TC-427
title: Hash verify checks content-hashes independently
type: scenario
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_427_hash_verify_checks_content_hashes_independently"
---

## Description

Set up a repo with one accepted ADR (valid hash) and one tampered accepted ADR (invalid hash). Run `product hash verify`. Verify it reports E014 for the tampered ADR and passes the valid one — without running the full `graph check` suite (no orphan warnings, no broken link checks, etc.). Run `product hash verify ADR-XXX` for the specific tampered ADR and verify the same E014 output. Verify exit codes match: 0 for all-valid, 1 for any mismatch.
