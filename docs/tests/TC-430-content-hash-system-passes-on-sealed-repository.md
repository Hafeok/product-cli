---
id: TC-430
title: Content hash system passes on sealed repository
type: exit-criteria
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_430_content_hash_system_passes_on_sealed_repository"
---

## Description

After running `product adr rehash --all` and `product hash seal --all-unsealed` on a repository with accepted ADRs and finalized TCs:

1. `product graph check` produces zero E014, E015, or W016 diagnostics related to content-hash
2. `product hash verify` exits with code 0
3. `product adr amend ADR-XXX --reason "test"` on any accepted ADR succeeds and subsequent `product graph check` still passes
