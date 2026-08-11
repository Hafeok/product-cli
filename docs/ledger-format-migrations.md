# Decision Ledger Format Migrations

Every schema change to the `.decisions/` format is a version bump with a
migration note here, and `ledger verify` checks each entry against the
version it declares — existing entries never break silently. This file is the
migration record. The format itself is specified in `ledger-format-v1.md`.

Two versions move independently; both are recorded here.

| Version | Governs | A bump means |
|---|---|---|
| `format: N` on each file | how a file is read | older files keep working; validation is per declared version |
| `CANONICAL_FORM` in the hash prefix | how a version hash is computed | **every existing acceptance is invalidated** |

A `CANONICAL_FORM` bump is a governed act, not a fix. It is required whenever
a hashed field changes meaning, gains or loses membership in the hashed set,
or is normalised differently. It is *not* required for a `format` bump that
only adds an unhashed field.

---

## Spec v1.2 — latest from the parent DAG; `G004` (2026-08-11)

An **amendment to the specification document**, not a `format` bump and
not a `CANONICAL_FORM` bump: no file schema changes, no hashed field
changes meaning, every existing acceptance stays valid.

The shipped L1 computed a decision's latest version by ULID order of
change-sets — the single-writer leak the L1+L2 report named: ULIDs order
by one clock, and two writers' clocks prove nothing about parenthood. As
of v1.2, §5.2 defines "latest" as **the unique version whose hash no
other version of the same decision names as `parent`** — the parent DAG
(which `G002` already polices) is the authority, and file order is not
consulted. Content-identical filings of one hash are one version.

A chain with more than one tip has **no** latest, and no ordering
heuristic may pick one. The reference implementation's graph stage gains
`G004` (forked version chain) for exactly that state, and its authoring
verbs refuse to extend or sign a forked decision. The remedy is an
arbitration recorded through `ledger merge --resolve` (L3), never a
silent resolution.

**Migration note:** a single-writer store is unaffected — a linear chain's
tip is the same version ULID order found, so no hash moves and no
acceptance is disturbed. A store already carrying interleaved clocks may
change which version `status`/`coverage`/the gate judge as latest; the DAG
reading is the correct one and the ULID reading was the defect. A store
carrying an undetected fork newly fails `G004` — that is the point of the
amendment.

## Spec v1.1 — the tenth class, `L010` (2026-08-10)

An **amendment to the specification document**, not a `format` bump and not
a `CANONICAL_FORM` bump: no file schema changes, no hashed field changes
meaning, every existing acceptance stays valid.

v1.0's §6 stated the gap plainly: the model-identity rule was scoped to
`accepted-by`, and the classes were closed at nine — so a *judgment
allocated to a model actor* passed the gate. Ruled by the principal
(2026-08-10): the gap closes as `L010` — a `judgment`'s `actor` refused by
the §3.2 identity rules fails the gate. The class runs under both gates and
judges latest versions only, like every allocation rule.

**Migration note:** the amendment is additive and stricter. A store that was
conformant under v1.0 may newly fail `L010` — that is the point, not a
regression. The remedy is a new version reallocating the judgment to an
accountable human actor (or to another store); there is nothing to rewrite,
because history is append-only. The "exactly nine" contract is now "exactly
ten" everywhere it is asserted, including the closed-enum count test.

## L2 — the graph stage arrives (2026-08-10, no format change)

Not a `format` bump, not a `CANONICAL_FORM` bump, not part of the file
gate's ten classes. `.decisions/index/` (reserved since format 1) is now
written by `ledger reindex` as byte-deterministic Turtle, and `verify`
gains a distinct graph stage (`G001`–`G003`, cross-entry referential
integrity) with unchanged exit semantics. Specified in
`ledger-format-v1.md` §8, explicitly outside the import surface an
outside implementation must reproduce.

## Format 1 / `ledger.decision-version.v1` (L0)

The baseline. Two file schemas — the set file and the change-set log file —
plus the canonical form and its pinned conformance vector, all specified in
`ledger-format-v1.md`.

Nothing to migrate from.

### Fields reserved but inert at this version

These exist in the schema so their milestone is additive rather than a
migration. Writing them is legal where noted; nothing reads them yet.

| Field | Reserved for | State at format 1 |
|-------|--------------|-------------------|
| `signature` on an acceptance | OD-3's cryptographic upgrade | must be **empty**; a non-empty value is a schema fault |
| `supersedes` on a version | supersession (no command before L1) | may be written; hashed; unresolved |
| `based_on` on a version | §9.4 basis pointers, resolved at L4 | may be written; hashed; vocabulary open |
| `parents` on a change-set | merge (L3) | may be written; unresolved |
| `scope: class:<ref>` on an acceptance | precommitment (L1) | parses; the acceptance still signs one version hash |
| `.decisions/index/` | the RDF materialized view (L2) | not created; ignored by git |

### Known future migrations

Recorded now so the shape of the change is not a surprise.

- **OD-6 — expiry default.** If `expires_at` becomes mandatory, that is a
  `format: 2` with a migration path for entries that carry none. Hashing is
  unaffected: `expires_at` is not hashed.
- **OD-3 — signatures.** Populating `signature` is a `format: 2`. Hashing is
  unaffected: the signature is *over* the hash, not inside it.
- **§9.4 — upstreams manifest.** A new file schema, not a change to these
  two. Closing the `based_on` vocabulary at that point **is** a hashed-meaning
  change and would require a `CANONICAL_FORM` bump, so the closure should
  arrive as validation over an unchanged canonical form instead.
