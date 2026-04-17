# Product Request Log Specification

> Standalone reference for the Product request log — the immutable,
> hash-chained audit trail of all knowledge graph mutations.
> Amendment to ADR-032 in `product-adrs.md`.

---

## Overview

Every `product request apply` appends an entry to `requests.jsonl`. The file is:

- **Append-only** — entries are never edited or deleted
- **Hash-chained** — each entry includes the hash of the previous entry
- **Committed** — lives in the repository alongside `metrics.jsonl` and `gaps.json`
- **Complete** — migration, undo, and schema upgrade operations all produce entries

The log is not the source of truth for day-to-day queries — the artifact files on disk
are. The log is the audit trail and the replay mechanism. It answers: what mutations
produced the current state, when, by whom, and in what order?

---

## Entry Format

```json
{
  "id": "req-20260414-001",
  "applied-at": "2026-04-14T09:14:22Z",
  "applied-by": "git:Emil <emil@example.com>",
  "commit": "abc123def",
  "type": "create",
  "reason": "Add rate limiting to the resource API",
  "request": {
    "type": "create",
    "reason": "Add rate limiting to the resource API",
    "artifacts": [
      { "type": "feature", "ref": "ft-rate-limiting", "title": "Rate Limiting", "phase": 2 },
      { "type": "adr",     "ref": "adr-token-bucket", "title": "Token bucket algorithm" }
    ]
  },
  "result": {
    "created": [
      { "ref": "ft-rate-limiting", "id": "FT-009", "file": "docs/features/FT-009-rate-limiting.md" },
      { "ref": "adr-token-bucket", "id": "ADR-031", "file": "docs/adrs/ADR-031-token-bucket.md" }
    ],
    "changed": []
  },
  "prev-hash": "sha256:f7c2a91b3e4d85f6c0a2b7d9e3f1c4a8b5d2e7f9c1a3b6d8e0f2a4c7b9d1e3f5",
  "entry-hash": "sha256:a3f9b2c1d4e7f0a2b5c8d1e4f7a0b3c6d9e2f5a8b1c4d7e0f3a6b9c2d5e8f1a4"
}
```

### Fields

| Field | Description |
|---|---|
| `id` | Unique entry ID — `req-{date}-{seq}` where seq increments within the day |
| `applied-at` | ISO 8601 timestamp — when `product request apply` ran |
| `applied-by` | Git author string from `git config user.name/email` at apply time |
| `commit` | Short SHA of HEAD at apply time — not the commit that adds the log entry, but the repo state when applied |
| `type` | Entry type — see Entry Types below |
| `reason` | The `reason:` field from the request YAML |
| `request` | Full parsed request object (YAML parsed to JSON) |
| `result` | Assigned IDs and changed targets from apply |
| `prev-hash` | `entry-hash` of the preceding entry. First entry uses `"0000000000000000"` |
| `entry-hash` | sha256 of this entry serialised with `entry-hash` set to `""` |

---

## Hash Chain

Each entry's `entry-hash` is computed as:

```
sha256(canonical_json(entry with entry-hash: ""))
```

Where `canonical_json` is deterministic JSON serialisation:
- Keys sorted alphabetically at every level
- No trailing whitespace
- UTF-8 encoding
- No BOM

The chain: each entry's `prev-hash` equals the `entry-hash` of the entry immediately
before it in the file. The first entry uses `prev-hash: "0000000000000000"` as the
genesis sentinel.

### Why a hash chain and not individual signatures?

Individual entry hashes catch modification of a single entry. A hash chain additionally
catches **deletion** and **insertion**:

- **Delete entry N** → entry N+1's `prev-hash` no longer matches entry N-1's `entry-hash`. Detected.
- **Insert fabricated entry** → the inserted entry's `prev-hash` must match its predecessor, requiring recomputation of all subsequent hashes. Without rewriting the entire log (which git history would record), insertion is detectable.
- **Truncate from the end** → not detectable by hash alone. Caught by cross-referencing with git completion tags: if `product/FT-009/complete` exists but no log entry created FT-009, the log has been truncated.

---

## Entry Types

| Type | Created by | Description |
|---|---|---|
| `create` | `product request apply` (type: create) | New artifacts created |
| `change` | `product request apply` (type: change) | Existing artifacts mutated |
| `create-and-change` | `product request apply` (type: create-and-change) | Both |
| `undo` | `product request undo REQ-ID` | Reversal of a previous entry |
| `migrate` | `product migrate` | One-time import from monolithic docs |
| `schema-upgrade` | `product migrate schema` | Schema version bump |
| `verify` | `product verify FT-XXX` | TC run results and git tag creation |

### `verify` entry

`product verify` is Product's one write-side operation outside the request model.
It records TC results and tag creation in the log:

```json
{
  "id": "req-20260414-006",
  "type": "verify",
  "reason": "product verify FT-009",
  "feature": "FT-009",
  "result": {
    "tcs-run": ["TC-050", "TC-051", "TC-052"],
    "passing": ["TC-050", "TC-051", "TC-052"],
    "failing": [],
    "tag-created": "product/FT-009/complete"
  },
  "prev-hash": "sha256:...",
  "entry-hash": "sha256:..."
}
```

### `undo` entry

Undo never deletes past entries — it appends a reversal:

```json
{
  "id": "req-20260414-007",
  "type": "undo",
  "reason": "Reverting rate limiting — design changed",
  "undoes": "req-20260414-001",
  "inverse-request": {
    "type": "change",
    "reason": "Undo of req-20260414-001",
    "changes": [
      {
        "target": "FT-009",
        "mutations": [{ "op": "delete", "field": "status" }]
      }
    ]
  },
  "prev-hash": "sha256:...",
  "entry-hash": "sha256:..."
}
```

The `inverse-request` is a change request that reverses all mutations from the target
entry. For create entries, the inverse is a `delete` — marking artifacts as abandoned
and removing their links (files are not deleted from disk, status is set to `abandoned`).

### `migrate` entry

```json
{
  "id": "req-20260414-000",
  "type": "migrate",
  "reason": "Initial migration from picloud-prd.md and picloud-adrs.md",
  "sources": ["picloud-prd.md", "picloud-adrs.md"],
  "result": {
    "created": [
      { "id": "FT-001", "file": "docs/features/FT-001-cluster-foundation.md" },
      { "id": "ADR-001", "file": "docs/adrs/ADR-001-rust-language.md" }
    ]
  },
  "prev-hash": "0000000000000000",
  "entry-hash": "sha256:..."
}
```

Migration is always the first entry. If migration ran in multiple passes, each pass
gets its own entry with the chain intact.

---

## Verification

### `product request log verify`

```
product request log verify

  Verifying requests.jsonl (47 entries)...

  ✓ Entry hashes valid (47/47)
  ✓ Hash chain intact (47/47)
  ✓ Tag cross-reference clean — all completion tags have corresponding verify entries

  Log is tamper-free.
```

On failure:

```
product request log verify

  Verifying requests.jsonl (47 entries)...

  ✓ Entry hashes valid (46/47)
  ✗ Entry hash mismatch at line 23 (req-20260414-019)
    stored:   sha256:a3f9b2c1...
    computed: sha256:d7e2f4a8...

  error[E015]: requests.jsonl entry at line 23 has been tampered with
    The stored hash does not match the computed hash.
    The entry reason was: "Add rate limiting"
    All entries after line 23 cannot be trusted.
```

```
  ✗ Chain break at line 24 (req-20260414-020)
    prev-hash in entry: sha256:a3f9b2c1...
    actual hash of entry 23: sha256:d7e2f4a8...

  error[E016]: requests.jsonl chain break at line 24
    This entry's prev-hash does not match the hash of the preceding entry.
    An entry may have been inserted, deleted, or modified before this point.
```

### `product graph check` integration

`product graph check` runs log verification as part of its standard checks. E015 and
E016 are exit code 1 — the same severity as broken links and dependency cycles. A
tampered log is a structural integrity violation.

### `product request log verify --against-tags`

Cross-references the log with git tags to detect truncation:

```
product request log verify --against-tags

  Checking completion tags against log...

  product/FT-009/complete  ✓  req-20260414-006 (verify entry)
  product/FT-001/complete  ✓  req-20260414-002 (verify entry)
  product/ADR-031/accepted ✗  no log entry found

  warning[W021]: git tag product/ADR-031/accepted has no corresponding log entry
    The log may have been truncated or the tag was created outside Product.
```

---

## Replay

### `product request log`

```
product request log

  #  ID                    Type              Reason
  ─────────────────────────────────────────────────────────────────
  1  req-20260414-000  migrate           Initial migration from picloud-prd.md
  2  req-20260414-001  create            Add rate limiting to the resource API
  3  req-20260414-002  verify            product verify FT-001
  4  req-20260414-003  change            Add security domain after preflight W010
  5  req-20260414-004  create-and-change Add exit criteria TC to FT-003
  ...

product request log --show req-20260414-001   # full entry detail
product request log --type create             # filter by type
product request log --feature FT-009          # entries touching a feature
```

### `product request replay`

Replays the log to reconstruct graph state at any point:

```
product request replay --to req-20260414-003   # state after entry 3
product request replay --from req-20260414-002 # entries 2 onwards
product request replay --full                  # full replay from genesis
```

Replay writes to a temporary directory by default — it does not overwrite the current
working tree. The output is a complete repository at the specified point in history.

```
product request replay --to req-20260414-003 --output /tmp/replay-20260414

  Replaying 3 entries...
  → req-20260414-000  migrate           47 artifacts created
  → req-20260414-001  create            5 artifacts created
  → req-20260414-002  verify            FT-001 marked complete
  → req-20260414-003  change            FT-009 security domain added

  Replay complete. State written to /tmp/replay-20260414
  Run: product graph check --repo /tmp/replay-20260414
```

`--full` replay followed by `product graph check` on the result is the integrity proof:
if the graph derived from the log matches the current graph on disk, the files and the
log are consistent.

---

## `product.toml` Configuration

```toml
[paths]
requests = "requests.jsonl"   # append-only request log (committed)

[log]
verify-on-check = true        # run log verification during product graph check
hash-algorithm = "sha256"     # sha256 only for now
```

---

## New Validation Codes

| Code | Tier | Description |
|---|---|---|
| E015 | Integrity | `requests.jsonl` entry hash mismatch — entry at line N has been tampered with |
| E016 | Integrity | `requests.jsonl` chain break — `prev-hash` at line N does not match hash of preceding entry |
| W021 | Integrity | Git completion tag has no corresponding verify entry in the log — possible truncation |

---

## Session Tests

These sessions cover the log and hash chain behaviour and belong in `tests/sessions/`:

```
ST-090  log-entry-appended-on-apply
ST-091  log-entry-hash-valid-after-apply
ST-092  log-chain-intact-after-multiple-applies
ST-093  log-verify-passes-on-clean-log
ST-094  log-verify-detects-entry-modification     # tamper entry N, assert E015
ST-095  log-verify-detects-chain-break            # tamper prev-hash, assert E016
ST-096  log-verify-detects-entry-deletion         # delete entry N, assert E016 at N+1
ST-097  log-replay-reconstructs-state             # replay --full, diff against current
ST-098  log-replay-to-checkpoint                  # replay --to N, assert correct state
ST-099  log-undo-appends-inverse                  # undo, assert undo entry in log
ST-100  log-undo-does-not-delete-entries          # undo, assert original entry present
ST-101  log-migrate-entry-first                   # migration creates genesis entry
ST-102  log-verify-entry-on-product-verify        # product verify writes verify entry
ST-103  log-cross-ref-tags-detects-truncation     # delete verify entry, assert W021
```

---

## Property Tests

Property-level invariants for the hash chain (belong in `tests/property/`):

| TC | Property | Formal expression |
|---|---|---|
| TC-P015 | Entry hash is deterministic | `∀e:Entry: hash(e) = hash(e)` |
| TC-P016 | Any field change invalidates hash | `∀e:Entry, f:Field, v:Value: hash(mutate(e,f,v)) ≠ hash(e)` |
| TC-P017 | Chain breaks on any deletion | `∀log:Log, n:Index: verify(delete(log,n)) = Err(E016)` |
| TC-P018 | Replay of log produces same graph | `∀log:Log: graph(replay(log)) = graph(files(log))` |

TC-P018 is the most important — it is the proof that the log and the files are
equivalent representations of the same state.

---

## Invariants

- `requests.jsonl` is **append-only**. Product never overwrites or deletes entries.
  Undo produces a new entry. Replay writes to a separate directory.
- The genesis entry always has `prev-hash: "0000000000000000"`.
- Every `product request apply` produces exactly one log entry, regardless of how many
  artifacts are created or changed.
- Every `product verify` that succeeds produces exactly one `verify` log entry.
- `product request log verify` is a pure read operation — it never modifies the log,
  even when it finds errors.
- The `entry-hash` is computed over the full entry with `entry-hash` set to empty string,
  then the computed hash is written into the entry. The canonical JSON serialisation is
  deterministic — same entry, same hash, always.
