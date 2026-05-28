---
id: FT-034
title: Content Hash Immutability
phase: 1
status: complete
depends-on: []
adrs:
- ADR-002
- ADR-013
- ADR-015
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
domains:
- data-model
- security
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
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

---

## Functional Specification

### Inputs

- `product adr status ADR-XXX accepted` — triggers hash computation and writes `content-hash` to the ADR file at the moment of acceptance.
- `product adr amend ADR-XXX --reason "..."` — mandatory non-empty reason string; triggers hash recomputation and appends an amendment record.
- `product adr rehash [ADR-XXX | --all]` — seals accepted ADRs that predate this feature (no previous hash).
- `product hash seal [TC-XXX | --all-unsealed]` — computes and writes `content-hash` to TC files with non-empty bodies.
- `product hash verify [ARTIFACT-ID]` — verifies one or all content-hashes without running the full graph check.
- `product graph check` — triggers integrity verification of all ADR and TC content-hashes on every run.
- The protected fields for hash computation:
  - **ADR**: body text (everything after the closing `---`) + `title`, normalised to LF endings with leading/trailing whitespace trimmed.
  - **TC**: body text + `title`, `type`, `validates.adrs`.

### Outputs

- **`product adr status ... accepted`** — writes updated front-matter with `content-hash: sha256:...` to the ADR file atomically.
- **`product adr amend`** — writes updated `content-hash` and appends an entry to the `amendments` array in front-matter; the amendment records `date`, `reason`, and `previous-hash`.
- **`product hash seal`** — writes `content-hash: sha256:...` to the TC file; prints `sealed TC-XXX -> sha256:...` per sealed file.
- **`product hash verify`** — prints pass/fail per artifact; exits 0 if all match, exits 1 if any mismatch.
- **`product graph check`** — emits E014 or E015 (exit 1) on hash mismatch; emits W016 (exit 2) for accepted ADRs without a content-hash.
- Error messages for E014 and E015 name the file, show expected vs. actual hash, and provide a remediation hint.

### State

Content-hashes are stored as `content-hash: sha256:...` in YAML front-matter alongside the artifact they protect. Amendment records accumulate in the `amendments` array in ADR front-matter. These fields persist across invocations — they are the durable integrity record. No separate database or index is maintained.

### Behaviour

1. **Hash algorithm** — SHA-256, hex-encoded, stored with the `sha256:` prefix. Uses the `sha2` crate already in the dependency tree.
2. **Protected fields** — only body text and specified front-matter fields are hashed. Mutable fields (`status`, `features`, `domains`, `scope`, `source-files`, `last-run`, `runner`, `runner-args`, etc.) are excluded so that normal lifecycle operations do not invalidate the hash.
3. **ADR acceptance sealing** — `product adr status ADR-XXX accepted` computes the hash from the current file content and writes it. Draft ADRs (status `proposed`) carry no hash and can be freely edited.
4. **TC sealing** — `product hash seal TC-XXX` is a manual step. TCs with empty bodies are skipped. `--all-unsealed` seals all TCs with a body and no existing hash.
5. **Graph check verification** — `product graph check` recomputes the hash for every accepted ADR with a `content-hash` field and every TC with a `content-hash` field, comparing against the stored value. Any mismatch is a hard error (exit 1).
6. **W016 for unsealed accepted ADRs** — if an accepted ADR has no `content-hash`, `graph check` emits W016 (exit 2). This provides a migration path for repos adopting the feature incrementally.
7. **Amendment path** — `product adr amend ADR-XXX --reason "..."` recomputes the hash from the current (modified) content, records the old hash in an amendment entry, and writes the new hash. If the hash has not changed (no actual modification), the command exits with "nothing to amend".
8. **MCP protection** — MCP write tools that could modify an accepted ADR's body are blocked. `product_adr_status` (which only touches `status`) is allowed and writes the hash on acceptance.
9. **`product hash verify`** — a focused subset of graph check that only verifies content-hashes, useful in CI pipelines that want a fast integrity check.

### Invariants

- Once an ADR reaches `status: accepted`, its body text and `title` must not change without going through `product adr amend`. Any other modification triggers E014 on the next `product graph check`.
- Once a TC has a `content-hash` field, its body, `type`, and `validates.adrs` must not change. Any modification triggers E015.
- `product adr amend` requires a non-empty `--reason`; an empty reason is rejected with an error.
- The `amendments` array in ADR front-matter is append-only; existing amendment records are never removed or modified.
- The hash prefix is always `sha256:`. No other hash algorithm is used.

### Error handling

- **E014** — ADR body or title changed after acceptance (content-hash mismatch). Exit code 1. Message names the file, shows expected vs. actual hash, hints to revert or run `product adr amend`.
- **E015** — sealed TC body or protected fields changed. Exit code 1. Message names the file, hints to revert or create a new TC.
- **W016** — accepted ADR has no content-hash. Exit code 2. Hint: run `product adr rehash`.
- `product adr amend` without `--reason` exits with `ProductError::ConfigError` naming the missing flag.
- `product adr rehash` on an ADR that already has a hash exits with a message "already sealed" and does not overwrite.

### Boundaries

- Feature and dependency `id` and `title` fields are immutable by convention, not enforced by content-hash. The hash mechanism targets ADRs and TCs where unauthorized mutation has the highest impact on agent-driven implementation.
- TC runner configuration fields (`runner`, `runner-args`, `runner-timeout`, `requires`) are explicitly excluded from the hash — they are infrastructure details that can legitimately change without invalidating the specification.
- `product adr amend` is the only legitimate path to update an accepted ADR's protected content. There is no equivalent amendment command for TCs; changed TC specifications require a new TC artifact.
- `product hash verify` is a read-only command; it never writes files or modifies hashes.

## Out of scope

- Git-level content integrity — `product graph check` verifies artifact-level hashes, independent of git history. The two mechanisms are complementary, not duplicative.
- Hash enforcement for feature and dependency artifacts — those use ID/title immutability by convention.
- Automatic amendment recording for any field change — only `product adr amend` creates amendment records; other field changes (status, links, domains) are excluded from the hash by design.
- Cryptographic signing or public-key attestation of amendments — the audit trail records who ran the command based on git authorship, not via PKI.
- Hash verification for artifacts in external repositories or over the network.
