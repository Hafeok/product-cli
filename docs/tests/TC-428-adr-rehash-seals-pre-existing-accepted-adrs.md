---
id: TC-428
title: ADR rehash seals pre-existing accepted ADRs
type: scenario
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_428_adr_rehash_seals_pre_existing_accepted_adrs"
---

## Description

Create multiple ADR files manually with `status: accepted` but no `content-hash` (simulating pre-existing ADRs). Run `product adr rehash ADR-XXX` for one — verify it gets a `content-hash` and no `amendments` array (initial sealing, not an amendment). Run `product adr rehash --all` — verify all remaining accepted ADRs without hashes are sealed. Verify ADRs with `status: proposed` are not touched. Verify already-sealed ADRs are not modified.
