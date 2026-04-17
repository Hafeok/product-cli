---
id: ADR-039
title: Hash-Chained Request Log for Tamper-Evident Audit
status: accepted
features:
- FT-042
supersedes: []
superseded-by: []
domains:
- data-model
- observability
- security
scope: domain
content-hash: sha256:0516156eea7bf9b19914870a18b949957db9b5bcaf58a275581b29d987bf810b
amendments:
- date: 2026-04-17T14:12:30Z
  reason: Sync body Status line with accepted front-matter
  previous-hash: sha256:d3718d2ffccb92fa200742c513e7b6a208ab7e0ea3f3e405c41789b15d944da5
source-files:
- product.toml
- src/main.rs
- src/request.rs
- src/request_log.rs
---

**Status:** Accepted

**Context:** FT-041 established the Product request as the unified write interface. Every successful `product request apply` appends one line to a request log. As shipped, the log had three gaps:

1. **Gitignored path** — FT-041 wrote to `.product/request-log.jsonl`, which `.product/` patterns typically exclude from version control. The log was local to a clone and disappeared on fresh checkouts. An audit record that is not committed is not an audit record.
2. **No tamper evidence** — entries were plain JSON lines. Silent edits, deletions, or insertions in the log file were undetectable. Nothing distinguishes an honest log from one an attacker (or a well-meaning editor) has quietly rewritten.
3. **No replay** — the log recorded what happened but could not reconstruct the graph at a past point. It was an audit sink, not an audit source.

Separately, `product verify` is Product's only write-side operation outside the request model. It emits TC results and creates `product/FT-XXX/complete` git tags (ADR-036), but nothing binds those tags to a log entry. A completion tag with no corresponding log record is invisible to the audit pipeline.

The system needs a request log that is committed to the repository, cryptographically chained so any modification is detected, and replayable so the log and the files can be proven equivalent.

**Decision:** Promote the request log to a committed, hash-chained, verifiable, and replayable audit trail at `requests.jsonl` alongside `metrics.jsonl` and `gaps.json`. The full specification is in [`docs/product-request-log-spec.md`](../product-request-log-spec.md); this ADR pins the decisions that shape the spec.

---

### Decisions pinned by this ADR

**1. The log is committed, at `requests.jsonl` at the repository root.** FT-041's `.product/request-log.jsonl` path is replaced. Migration is a one-shot: on first run of the new binary, if `.product/request-log.jsonl` exists and `requests.jsonl` does not, copy the existing entries forward (re-hashing the chain) and record a `migrate` entry that notes the move. The `[paths]` section of `product.toml` gains a `requests = "requests.jsonl"` key.

**2. Entries are hash-chained with SHA-256.** Each entry carries `prev-hash` (the `entry-hash` of the preceding entry) and `entry-hash` (sha256 over the entry's own canonical JSON with `entry-hash` set to `""`). The genesis entry uses `prev-hash: "0000000000000000"`. A single altered entry invalidates its own hash (caught by per-entry verification) and every subsequent entry's `prev-hash` (caught by chain verification). Deletion and insertion are detectable without rewriting the entire log.

**3. Canonical JSON is deterministic.** Keys sorted alphabetically at every nesting level, no trailing whitespace, UTF-8 with no BOM. Same entry always serialises to the same bytes and the same hash. Without a canonical form, hashes are a round-tripping guessing game.

**4. Seven entry types, one schema.** `create`, `change`, `create-and-change` (produced by `product request apply`); `undo` (produced by `product request undo`); `migrate` (produced by `product migrate`); `schema-upgrade` (produced by `product migrate schema`); `verify` (produced by `product verify`). All seven share the same chained envelope — `id`, `applied-at`, `applied-by`, `commit`, `type`, `reason`, `prev-hash`, `entry-hash` — and differ only in the type-specific payload fields. A new entry type can be added without breaking existing verifiers, as long as the envelope fields remain.

**5. Undo never deletes, only appends.** `product request undo REQ-ID` appends a new entry of type `undo` that carries an `inverse-request` payload — a synthesised `change` request that reverses each mutation of the target entry. For `create` entries, the inverse sets status to `abandoned` and strips links; files on disk are not deleted. Undoing past undo entries is legal and produces another undo. The chain is never rewritten.

**6. `product verify` writes a log entry.** Before this ADR, verify was invisible to the log. Now every successful `product verify FT-XXX` appends one `verify` entry containing `tcs-run`, `passing`, `failing`, and the `tag-created` tag name. This closes the loop between ADR-036 completion tags and the audit trail: every tag has a log entry, every verify log entry corresponds to a tag.

**7. Entry IDs are date-sequence, not UUIDs.** `req-{YYYYMMDD}-{NNN}` where `NNN` is the 1-indexed sequence within that UTC date, zero-padded to three digits. Human-readable, sortable, compact. Collisions within a day are prevented by the sequence counter; collisions across clones are prevented by the fact that each clone has its own log — merge conflicts on `requests.jsonl` are resolved by the user, and the chain is re-verified after merge.

**8. The `applied-by` field uses `git config user.name/email` at apply time.** Not a Product identity. If the operator has no git identity configured, `product request apply` refuses with a clear error. This avoids inventing a parallel identity system and reuses the git contract the repo already operates under.

**9. Verification is a pure read.** `product request log verify` never writes to the log, even when it finds tampering. The tampered log is surfaced as structured output; remediation is up to the operator. This keeps verify safe to run in CI, in read-only replicas, and against stored archives.

**10. Verification is wired into `graph check`.** `product graph check` runs log verification as part of its standard sweep when `[log] verify-on-check = true` (default). Chain break and hash mismatch findings exit 1 at the same severity as broken links and dependency cycles. A tampered log is a structural integrity violation, not a warning.

**11. Replay writes to a temporary directory, never the working tree.** `product request replay --to REQ-ID --output /tmp/replay-X` reconstructs the graph at the specified point into a separate directory. The working tree is never overwritten. `product graph check --repo /tmp/replay-X` on the result is the integrity proof: if the graph derived from replaying the log matches the current graph on disk, the log and the files are consistent. Overwriting the working tree would make replay dangerous; a separate directory makes it safe.

**12. Truncation from the end requires git-tag cross-reference.** Hash chaining detects modification, insertion, and deletion from the middle. Truncation from the tail is invisible to the chain alone — the truncated log is still internally consistent. `product request log verify --against-tags` closes this gap: every `product/*/complete` or `product/ADR-*/accepted` git tag must correspond to a `verify` or `change` entry in the log. A tag without a matching entry is W021 — possible truncation, possibly a tag created outside Product.

**13. Validation codes: E017, E018, W021.** E017 is per-entry hash mismatch (line N's stored `entry-hash` does not match the computed hash). E018 is chain break (entry N+1's `prev-hash` does not match entry N's `entry-hash`). W021 is tag-without-log-entry. These codes were picked by reading the current error catalogue: E014/E015 are taken by ADR-032, E016 is taken by ADR-034; E017/E018 are the next free integrity-tier codes. W017–W020 are taken by existing features; W021 is the next free warning code.

**14. Canonicalisation algorithm is specified explicitly in a test.** TC-P015 asserts that canonical JSON of a given entry equals a byte-for-byte fixed expected string. This pins the canonical serialisation independently of whichever JSON library is used, preventing silent behaviour changes when dependencies update.

**15. Replay produces the same graph as the files on disk.** TC-P018 — the most important property. Its failure means either the log is inconsistent with the files (a bug in apply) or the replayer has drifted from the applier (a bug in replay). This property is the single check that keeps the log worth trusting: if replay diverges from reality, the log is decorative at best and misleading at worst.

---

### Validation code allocation

| Code | Tier | Description |
|---|---|---|
| E017 | Integrity | `requests.jsonl` entry hash mismatch — entry at line N has been tampered with |
| E018 | Integrity | `requests.jsonl` chain break — `prev-hash` at line N does not match the `entry-hash` of line N-1 |
| W021 | Integrity | Git completion tag has no corresponding `verify` entry — possible log truncation or a tag created outside Product |

These codes do not collide with ADR-032 (E014/E015) or ADR-034 (E016). They follow the ADR-009 exit-code convention: E017 and E018 are exit 1; W021 is exit 2.

---

### `product.toml` schema additions

```toml
[paths]
requests = "requests.jsonl"    # new

[log]                          # new section
verify-on-check = true         # run log verification during product graph check
hash-algorithm = "sha256"      # sha256 only for now
```

Both are additive. Existing configurations continue to work with defaults.

---

### Relationship to ADR-032

ADR-032 enforces immutability of individual artifact content (ADR body, TC protected fields) via per-artifact content hashes. This ADR extends the immutability principle to the request log itself via entry-chained hashes. The two mechanisms are complementary:

- ADR-032's hash protects one file's content at a point in time
- ADR-039's chain protects the sequence of mutations across the whole graph

Together they give a repository that is hostile to silent tampering at both the artifact level and the audit-trail level. This ADR does not amend or supersede ADR-032; it extends the same principle to a new surface.

---

### Test coverage

The feature's TCs (FT-042) cover every decision pinned here:

| Decision | Covered by |
|---|---|
| Log is committed at `requests.jsonl` | TC: log path migration preserves chain |
| Entries hash-chained with SHA-256 | TC: log entry hash valid after apply; log chain intact after multiple applies |
| Canonical JSON deterministic | TC (property): entry hash is deterministic |
| Seven entry types, one schema | TC: verify entry on product verify; undo entry on request undo; migrate entry first |
| Undo never deletes | TC: undo does not delete entries; undo appends inverse |
| `product verify` writes a log entry | TC: verify entry on product verify |
| Entry IDs are date-sequence | TC: entry IDs increment within UTC day |
| `applied-by` from git config | TC: apply refuses without git identity |
| Verification is a pure read | TC (invariant): log verify never writes |
| Verification wired into graph check | TC: graph check exits 1 on tampered log |
| Replay writes to temp dir | TC: replay never overwrites working tree |
| Truncation detected via tags | TC: log cross-ref tags detects truncation |
| Replay ≡ files on disk | TC (property): replay produces same graph |

---

**Rationale:**

- **Hash chain, not per-entry signatures.** Signatures require key management — a weak identity surface for an audit log that already ties to git identity. A hash chain is keyless, trivially implementable, and catches deletion/insertion in addition to modification. It does not prove *who* wrote an entry — but the `applied-by` + `commit` fields plus the git commit that added the entry already answer that question from a different direction.
- **SHA-256 over any other hash.** Already a dependency via the `sha2` crate (ADR-019, ADR-032). No new crate, same family of primitives across the audit surface. Collision-resistant well past any realistic attacker budget for this threat model.
- **Canonical JSON instead of protobuf or CBOR.** JSON is already the log's storage format; a canonical JSON variant keeps the wire format readable and the canonicalisation step cheap. Binary formats would require either a second representation or a format flip, both of which lose the "grep the log" ergonomic property.
- **Committing `requests.jsonl`.** An audit log that lives outside version control is an audit log one `rm -rf .product/` away from vanishing. Committing it means the log shares the repository's durability, backup, and history properties automatically. It also means the log lives inside git's own tamper-evidence story (signed commits, force-push protection) as a second line of defence.
- **Replay to a separate directory.** Overwriting the working tree on replay would be a destructive default — a fat-fingered `--to` argument would trash uncommitted work. Replay is a read-and-reconstruct operation; its natural output is a fresh directory. The cost is one extra flag on `graph check`; the benefit is that replay is safe to run casually.
- **`verify` as a log entry.** Before this ADR, completion tags were the only record of verify runs. Tags are a low-dimensional record — name + SHA, no reason, no TC results. Log entries carry the full run detail and make `product verify` a first-class participant in the audit story.
- **Tag cross-reference for truncation.** A hash chain cannot detect tail truncation — by construction, a valid prefix of a valid chain is itself a valid chain. Tags are the external anchor: git controls tag creation independently of the log, and a tag without a log entry is structurally impossible under honest operation. The cross-reference test catches the one failure mode the chain alone cannot.
- **Allocated codes at ADR time, not implementation time.** An earlier draft of this ADR delegated the code numbers to the implementation because the spec used E015/E016 as placeholders without checking the catalogue. That was the wrong call: allocating codes in the ADR makes the spec, the tests, and the error-message strings all refer to the same identifiers without a future rewrite step. E017/E018/W021 were picked by scanning every E- and W-code reference in `src/` and `docs/`, so they are collision-free at the time of acceptance.

**Rejected alternatives:**

- **Per-entry Ed25519 signatures with a repo-embedded public key.** Strongest tamper evidence but requires key management (generation, rotation, revocation, loss). For a threat model that is "detect accidental or casual tampering", signatures are overkill. If a future threat model requires them, they can be layered on top of the hash chain (`signature:` field) without redesigning the chain.
- **Merkle tree over all entries with a root committed separately.** Logarithmic proofs per entry but requires tree maintenance and a second artifact for the root. Linear chain is sufficient for this log's size (thousands of entries in an active repo, not billions). Merkle is the right answer when random-access verification is a goal; here, full-log verification is the primary operation.
- **Store the log in git-notes instead of a committed file.** git-notes are designed for this shape of data (append-only annotation separated from the main object tree), but the tooling is obscure, the UX is poor, and notes are easily lost on clone (`--mirror` required). A plain committed file is legible, diffable, and survives standard clone operations.
- **Sign the log file with a PGP detached signature per commit.** Leans on git's existing signing story but signs the whole file, not individual entries — so verifying one entry requires the whole file plus the signature. The chain already gives per-entry verifiability at zero operational cost.
- **Skip the canonical-JSON step and trust the serialiser.** Rejected because JSON library output varies across versions and languages (key ordering, whitespace, number formatting). A hash computed with one library may not match the same entry re-serialised by another. Canonicalisation is the only stable contract.
- **Overwrite the working tree on replay by default with a `--dry-run` flag.** Inverted safety posture. Rejected because replay-to-temp + `graph check --repo` is strictly more informative (it lets the user diff) and the cost of the extra flag is trivial.
- **Make verify log entries optional.** Rejected because optionality reintroduces the gap this ADR closes. If verify sometimes logs and sometimes doesn't, the log's completeness claim becomes a footnote.
- **Hash algorithm pluggability (sha256 / sha3 / blake3).** The config field exists (`hash-algorithm = "sha256"`) for forward compatibility, but the implementation accepts only `sha256` in v1. Pluggability now would require a hash-agility story for chain verification across algorithm changes, which is complexity without a driving requirement. When a second algorithm becomes necessary, a schema-upgrade entry will mark the transition point.
