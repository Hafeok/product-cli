---
id: FT-042
title: Request Log Hash-Chain and Replay
phase: 5
status: complete
depends-on:
- FT-041
- FT-018
- FT-020
- FT-034
- FT-036
adrs:
- ADR-039
- ADR-032
- ADR-038
- ADR-015
- ADR-036
- ADR-009
- ADR-013
tests:
- TC-505
- TC-506
- TC-507
- TC-508
- TC-509
- TC-510
- TC-511
- TC-512
- TC-513
- TC-514
- TC-515
- TC-516
- TC-517
- TC-518
- TC-519
- TC-520
- TC-521
- TC-522
- TC-523
- TC-524
- TC-525
- TC-526
- TC-527
- TC-528
- TC-529
domains:
- api
- data-model
- error-handling
- observability
- security
domains-acknowledged:
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
---

The Product request log is the committed, hash-chained, tamper-evident audit trail of every graph mutation. It is the append-only record behind `product request apply`, `product request undo`, `product verify`, `product migrate`, and `product migrate schema`, and it is replayable — the same log deterministically reconstructs the graph at any point in its history.

The full specification is in [`docs/product-request-log-spec.md`](/docs/product-request-log-spec.md); the pinned decisions are in [ADR-039](/docs/adrs/ADR-039-hash-chained-request-log-for-tamper-evident-audit.md).

---

## Depends on

- **FT-041** — Product Request — Unified Write Interface. The log already exists but at a gitignored path without hash chaining. This feature promotes it to a committed chained log and extends it to cover verify/migrate/undo/schema-upgrade.
- **FT-018** — validation and graph health; the E-class codes and `graph check` integration.
- **FT-020** — migration path; `migrate` entries are the log's genesis.
- **FT-036** — tag-based implementation tracking (ADR-036 completion tags); cross-referenced by `verify --against-tags`.
- **FT-034** — content hash immutability; the sibling mechanism this feature extends to the log level.

---

## Scope of this feature

### In

1. **Log path migration.** Move from `.product/request-log.jsonl` (FT-041) to `requests.jsonl` at the repository root. One-shot migration on first run of the new binary: if the old file exists and the new one does not, copy the entries forward (re-computing the hash chain) and append a `migrate` entry documenting the move.
2. **Hash-chained entries.** Every log entry carries `prev-hash` and `entry-hash`. Genesis uses `prev-hash: "0000000000000000"`. Each `entry-hash` is `sha256(canonical_json(entry with entry-hash: ""))` — canonical JSON is keys-sorted-alphabetically-at-every-level, no trailing whitespace, UTF-8 without BOM.
3. **Seven entry types.** `create`, `change`, `create-and-change`, `undo`, `migrate`, `schema-upgrade`, `verify`. All share the envelope (`id`, `applied-at`, `applied-by`, `commit`, `type`, `reason`, `prev-hash`, `entry-hash`) and differ in type-specific payload fields.
4. **`product verify` writes a log entry.** Every successful `product verify FT-XXX` appends one `verify` entry with `tcs-run`, `passing`, `failing`, and the `tag-created` tag name. This is verify's first participation in the audit log.
5. **`product request undo REQ-ID`.** Appends an `undo` entry with an `inverse-request` payload that reverses each mutation of the target entry. `create` entries are inverted by marking the created artifacts `abandoned` and stripping links (files are not deleted). The chain is never rewritten.
6. **`product request log` family.** Read commands — `log` (tabular list with filters), `log --show REQ-ID` (full entry detail), `log --type TYPE`, `log --feature FT-XXX`. Pure read, no writes.
7. **`product request log verify`.** Verifies every entry's hash and the chain integrity. Pure read. Exit 0 clean, exit 1 on any E-class finding. `--against-tags` additionally cross-references `product/*/complete` and `product/ADR-*/accepted` git tags.
8. **`product request replay`.** Rebuilds graph state from the log into a temporary directory. `--to REQ-ID` for state-at-point, `--from REQ-ID` for partial replay from a cursor, `--full` for genesis-to-head. `--output DIR` selects the output directory (default: `/tmp/product-replay-{timestamp}`). Never overwrites the working tree.
9. **`graph check` integration.** `product graph check` runs log verification as part of its standard sweep when `[log] verify-on-check = true` (default). Chain break and hash mismatch exit 1 at the same severity as broken links.
10. **`product.toml` additions.** New `[paths] requests = "requests.jsonl"` and `[log] verify-on-check = true`, `hash-algorithm = "sha256"`.
11. **Entry ID format.** `req-{YYYYMMDD}-{NNN}` where `NNN` is the 1-indexed sequence within the UTC date, zero-padded to three digits. Sequence starts at `001` each day. Readable, sortable, compact.
12. **`applied-by` from git.** Each entry records `git:{user.name} <{user.email}>` at apply time. Apply refuses with a clear error if git identity is not configured — Product does not invent a parallel identity system.

### Out

- **Per-entry signatures (Ed25519 or otherwise).** Hash chain covers the threat model (accidental or casual tampering). Signatures can be layered on later as a `signature:` field without redesigning the chain.
- **Hash algorithm pluggability.** The config field exists for forward compatibility but the implementation accepts only `sha256` in v1. A second algorithm would require a schema-upgrade entry marking the transition.
- **Server-side log storage or remote log shipping.** The log is a committed file; the filesystem is its store. Remote shipping is out of scope.
- **Automatic log compaction or rotation.** The log is expected to grow at the pace of human graph mutations — thousands of entries per active repo, not billions. Rotation can be a later feature.
- **Deletion of artifacts via the undo path.** Undo sets status to `abandoned` and strips links; files remain on disk. Hard deletion remains a manual operation outside the request model.
- **Reconstructing state *into* the working tree via replay.** Replay always writes to a separate directory. Restoring the working tree from replay is a manual `rsync`-equivalent step.

---

## Entry schema

Shared envelope on every entry:

| Field | Description |
|---|---|
| `id` | `req-{YYYYMMDD}-{NNN}` — unique within the log |
| `applied-at` | ISO 8601 UTC timestamp |
| `applied-by` | `git:{name} <{email}>` from `git config` at apply time |
| `commit` | Short SHA of HEAD at apply time |
| `type` | One of the seven entry types |
| `reason` | Human-readable reason (from request YAML, or auto-generated for verify/migrate) |
| `prev-hash` | `entry-hash` of preceding entry; `"0000000000000000"` on genesis |
| `entry-hash` | `sha256(canonical_json(entry with entry-hash: ""))` |

Type-specific fields:

| Type | Extra fields |
|---|---|
| `create`, `change`, `create-and-change` | `request` (full parsed request), `result` (`created` + `changed` arrays) |
| `undo` | `undoes` (target REQ-ID), `inverse-request` (synthesised change request) |
| `migrate` | `sources` (array of input paths), `result.created` |
| `schema-upgrade` | `from-version`, `to-version`, `changes` (description of what migrated) |
| `verify` | `feature` (FT-XXX), `result.tcs-run`, `result.passing`, `result.failing`, `result.tag-created` |

See the spec for canonical examples.

---

## Canonical JSON

Deterministic serialisation is the cornerstone of hash stability:

- Keys sorted alphabetically at every nesting level
- No trailing whitespace, no pretty-printing — one entry is one line
- UTF-8, no BOM
- Numbers without trailing zeros (`42` not `42.0`); booleans lowercase; null lowercase
- Strings use standard JSON escaping; non-ASCII characters pass through unescaped

The `entry-hash` is computed by:
1. Build the entry object with `entry-hash: ""`
2. Serialise with canonical JSON
3. Compute `sha256` of the resulting byte string
4. Write the hash back into the `entry-hash` field
5. Serialise the entry again (now with the real hash) for disk storage

Same input entry, same bytes, same hash — always.

---

## Verification

`product request log verify` is a pure read operation:

```
product request log verify

  Verifying requests.jsonl (47 entries)...

  ✓ Entry hashes valid (47/47)
  ✓ Hash chain intact (47/47)
  ✓ Tag cross-reference clean

  Log is tamper-free.
```

On E017 (entry hash mismatch), E018 (chain break), or W021 (tag without log entry), the command prints a structured error naming the line, the stored hash, and the computed hash.

### Validation codes

| Code | Tier | Description |
|---|---|---|
| E017 | Integrity | `requests.jsonl` entry hash mismatch — entry at line N has been tampered with |
| E018 | Integrity | `requests.jsonl` chain break — `prev-hash` at line N does not match `entry-hash` of line N-1 |
| W021 | Integrity | Git completion tag has no corresponding `verify` entry — possible log truncation |

Allocated by ADR-039 after scanning the current error catalogue. E014/E015 are taken by ADR-032, E016 by ADR-034; E017/E018 are the next free integrity-tier error codes. W020 is reserved by the verify-and-llm-boundary spec; W021 is the next free warning code. No collision.

---

## Replay

```
product request replay --full --output /tmp/replay

  Replaying 47 entries...
  → req-20260414-000  migrate           Initial migration — 47 artifacts created
  → req-20260414-001  create            5 artifacts created
  ...
  → req-20260417-003  verify            FT-041 marked complete

  Replay complete. State written to /tmp/replay
  Run: product graph check --repo /tmp/replay
```

Replay applies each entry's `request` (or `inverse-request`, for undo) to a fresh repository skeleton in the output directory, in order. The resulting graph is the graph-as-of that entry. Running `product graph check --repo /tmp/replay` should produce zero findings (the replay is itself a valid apply sequence) and the resulting `docs/` tree should be byte-equivalent to the files on disk at the matching point.

`--to REQ-ID` stops replay at the named entry (inclusive). `--from REQ-ID` replays from that entry onwards (useful for partial rebuilds in tests). `--full` is the integrity proof.

---

## Undo

```yaml
# req-20260417-004
id: req-20260417-004
type: undo
reason: "Reverting rate limiting — design changed"
undoes: req-20260414-001
inverse-request:
  type: change
  reason: "Undo of req-20260414-001"
  changes:
    - target: FT-009
      mutations: [{ op: set, field: status, value: abandoned }]
    - target: ADR-031
      mutations: [{ op: set, field: status, value: abandoned }]
```

The inverse-request is synthesised at undo time by walking the target entry's `result.created` (mark each `abandoned`, strip its links from peers) and `result.changed` (reverse each mutation: `set` is inverted to `set` with the prior value, `append` inverts to `remove`, `remove` inverts to `append`, `delete` inverts to `set` with the prior value). The prior value for a `set` is the value present in the targeted artifact **immediately before** the original entry was applied — this requires either re-parsing the git blob at `commit` or walking the log backwards from the target entry.

Undo of an undo is legal and produces another undo entry whose `inverse-request` re-applies the original mutations.

---

## Acceptance criteria summary

A user can:

1. Run `product request apply request.yaml` and observe one new line appended to `requests.jsonl` at the repository root with a populated `entry-hash` and a `prev-hash` matching the previous entry's hash.
2. On first run of the new binary in a repo that has `.product/request-log.jsonl`, observe that `requests.jsonl` is created with the old entries re-hashed into a valid chain and a final `migrate` entry noting the move.
3. Run `product request log verify` on a clean log and observe exit 0 with per-entry and chain verification counts.
4. Manually edit one byte in one entry of `requests.jsonl`, run `product request log verify`, and observe E017 at the modified line plus E018 at every following line.
5. Delete one entry from `requests.jsonl`, run `product request log verify`, and observe E018 at the entry after the deletion.
6. Run `product request log verify --against-tags` and observe W021 per git tag that has no matching log entry.
7. Run `product graph check` on a tampered log and observe exit 1 with the E017 / E018 findings.
8. Run `product verify FT-XXX` on a feature whose TCs all pass and observe a `verify` entry appended to the log containing the TC results and the tag name.
9. Run `product request undo REQ-ID` on a past entry and observe an `undo` entry appended to the log (not a deletion of the target entry) and the targeted artifacts returned to their pre-REQ-ID state.
10. Run `product request replay --full --output /tmp/replay` and observe the command produces a graph in `/tmp/replay` that passes `product graph check --repo /tmp/replay` and whose `docs/` tree matches the current working copy.
11. Run `product request log` and observe a tabular view of every entry with id, type, reason.
12. Run `product request log --type verify` / `--feature FT-XXX` / `--show REQ-ID` and observe the filtered / detailed views.
13. Observe that `product request apply` refuses with a clear error in a repository where `git config user.name` or `git config user.email` is not set.
14. Observe that two runs of canonical JSON over the same entry produce byte-identical output (TC-P015 property).
15. Observe that changing any field in an entry and recomputing the hash produces a different hash (TC-P016 property).
16. Observe that deletion of any entry produces a chain-break finding at the following entry (TC-P017 property).
17. Observe that replay of the full log, followed by a diff of the replay graph against the on-disk graph, shows zero differences (TC-P018 property — the integrity proof).

---

## Implementation notes

- **New module: `src/request_log.rs`.** Canonical JSON, hash computation, chain linkage, entry append, entry parsing, verify, replay. Keep this module pure-ish — file I/O lives in `fileops` per ADR-015.
- **Canonical JSON is a small bespoke serialiser.** `serde_json` does not guarantee deterministic key ordering across versions; a 200-line hand-written serialiser over `serde_json::Value` is cheaper than depending on a canonicalisation crate and keeps the contract in-tree.
- **Git integration for `applied-by` and `commit`.** Use `git2` if already in the dependency tree, otherwise shell out to `git config` and `git rev-parse HEAD`. Fail apply if either is unavailable.
- **Replay reuses the apply pipeline.** Each entry's `request` is applied to a fresh directory using the same code path as `product request apply`, only with the target root redirected. This is the single biggest correctness invariant — one code path, two drivers (live apply and replay).
- **Chain re-hash during the `.product/request-log.jsonl` migration.** Old entries lack `prev-hash` / `entry-hash`. Migration walks them in order, sets `prev-hash` from the previous entry, computes `entry-hash`, and writes the result. The final appended `migrate` entry chains off the last re-hashed entry.
- **Validation codes are pinned at ADR-039 time: E017 (entry hash mismatch), E018 (chain break), W021 (tag without log entry).** They do not collide with ADR-032's E014/E015 or ADR-034's E016. No reconciliation step is required at implementation time.
