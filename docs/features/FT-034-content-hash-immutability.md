---
id: FT-034
title: Content Hash Immutability
phase: 1
status: planned
depends-on: []
adrs:
- ADR-032
tests:
- TC-420
- TC-421
- TC-422
- TC-423
- TC-424
- TC-425
- TC-426
- TC-427
- TC-428
- TC-429
- TC-430
domains: []
domains-acknowledged: {}
---

## Description

Enforce immutability of accepted ADR bodies and sealed TC specifications through SHA-256 content hashing. When an ADR is accepted, its body text and title are hashed and stored in front-matter. `product graph check` verifies these hashes on every run, emitting E014 (ADR tamper) or E015 (TC tamper) on mismatch. A `product adr amend` command provides the legitimate amendment path with mandatory reason and full audit trail.

### Capabilities

- **Hash computation**: SHA-256 over normalized body text + protected front-matter fields, written at acceptance (ADRs) or explicit seal (TCs)
- **Integrity checking**: `product graph check` and `product hash verify` detect unauthorized mutations
- **Amendment path**: `product adr amend --reason "..."` records legitimate corrections with audit trail
- **Migration**: `product adr rehash` seals existing accepted ADRs; `product hash seal` seals TCs
- **MCP protection**: Write tools enforce the same rules — no tool can modify an accepted ADR body

### New Commands

| Command | Purpose |
|---|---|
| `product adr amend ADR-XXX --reason "..."` | Record amendment, recompute hash |
| `product hash seal TC-XXX` | Compute and write content-hash for a TC |
| `product hash seal --all-unsealed` | Seal all TCs without a content-hash |
| `product hash verify [ID]` | Verify one or all content-hashes |
| `product adr rehash ADR-XXX` | Seal an accepted ADR that predates this feature |
| `product adr rehash --all` | Seal all accepted ADRs without content-hash |

### New Error Codes

| Code | Tier | Condition |
|---|---|---|
| E014 | Integrity | ADR body or title changed after acceptance |
| E015 | Integrity | Sealed TC body or protected fields changed |
| W016 | Warning | Accepted ADR has no content-hash |
