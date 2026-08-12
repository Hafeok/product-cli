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

## Format 3 / Spec v1.4 — the `contract:` discharge scheme (2026-08-12, ddd M8)

A **`format` bump without a `CANONICAL_FORM` bump**, by the same
reasoning as format 2: `format: 3` adds exactly one discharge scheme,
`contract:<boundary>` — the repository-diff contract check as a
discharge kind (M8 ruling 5: the CI contract-check discharge kind is
added through this amendment procedure, at the current version, with
this note). A `contract:` pointer names a declared boundary (a
`seam/...` declaration id or a `file#symbol` contract location) whose
changes are validated in CI by the shared classifier: every
contract-surface change in a revision range must be discharged by a
declaration signing that exact transition.

Rules that arrive with it:

- A writer declares `format: 3` only on a change-set that actually
  carries a `contract:` pointer — a store that never uses the scheme
  remains pure format 1/2. A lower-format file carrying one is a schema
  fault (the `merged_from` rule, applied to a scheme).
- Hashing is unaffected: a discharge pointer was always hashed by its
  string form, so no digest moves, no acceptance is invalidated, and the
  hash prefix stays `ledger.decision-version.v1`.
- The scheme's *resolution* (does the named boundary exist; is the CI
  check actually wired) is not the file gate's business — same posture
  as every other scheme at L0.

**Renumbering note:** the L6 signing revision, which had renumbered from
`format: 2` to `format: 3` when L3 consumed its slot, renumbers a second
time to **spec v1.5 / `format: 4`**. Nothing else about the L6 plan
changes; it remains ruled and unimplemented.

**Migration note:** nothing to migrate. Existing stores stay valid; the
first consumers of the scheme are the ddd M8 migration's seam-declaration
entries.

## Format 2 / Spec v1.3 — `merged_from`; `G005` (2026-08-11, L3)

A **`format` bump without a `CANONICAL_FORM` bump**, and the reasoning is
part of the record: `format: 2` adds exactly one optional version field,
`merged_from` — the other tip a merge arbitration closed. The field *is*
hashed when present, but an absent key is omitted from the canonical
object entirely (spec §4.2 step 3), so every version written before the
field existed canonicalises to byte-identical content: no digest moves,
no acceptance is invalidated, and the hash prefix stays
`ledger.decision-version.v1`.

Rules that arrive with it:

- A writer declares `format: 2` only on a change-set that actually carries
  `merged_from` — a store that never merged remains pure format 1. A
  format-1 file carrying the field is a schema fault.
- A version's tip-hood is judged over `parent` **and** `merged_from`: a
  reconciled version closes the tip it names, which is how a `G004` fork
  heals inside the DAG rather than by editing history.
- The graph stage gains `G005` (one decision superseded by two live
  claimants — the write-time fork refusal met across branches) and `G002`
  now also polices a dangling `merged_from`.
- No acceptance survives reconciliation. A reconciled version is a new
  version awaiting a fresh signature; prior acceptances keep signing the
  historical versions they named. Same law as `revise`.

**Migration note:** nothing to migrate. Existing stores are format 1 and
stay valid; they gain `G005` checking, which can newly fail a store that
already carried a silent competing supersession — that is the point.

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
| `parents` on a change-set | merge (L3) | may be written; still unresolved — L3 shipped on the *version* DAG (`parent`/`merged_from`), not the change-set DAG |
| `scope: class:<ref>` on an acceptance | precommitment (L1) | parses; the acceptance still signs one version hash |
| `.decisions/index/` | the RDF materialized view (L2) | not created; ignored by git |

### Known future migrations

Recorded now so the shape of the change is not a surprise.

- **OD-6 — expiry default.** If `expires_at` becomes mandatory, that is a
  further `format` bump with a migration path for entries that carry none
  (this entry originally said `format: 2`, a number since consumed by L3's
  `merged_from`). Hashing is unaffected: `expires_at` is not hashed.
- **OD-3 — signatures. Scheduled 2026-08-11 as milestone L6 / spec v1.5 /
  `format: 4`; unimplemented.** This entry originally read "populating
  `signature` is a `format: 2`" — that number was consumed by L3's
  `merged_from`, and the renumbered `format: 3` slot in turn by M8's
  `contract:` scheme, so the signing bump renumbers to `format: 4`;
  nothing else about the plan changes. What the revision will occupy
  (PRD §4.5):
  `signature` goes live as a detached, certificate-based signature (git's
  `gpg.format` trio — `openpgp` | `ssh` | `x509`) over a canonical
  acceptance payload — decision id, version hash, actor, signing timestamp —
  required only above the tolerance floor (T2 signed, T1 claimed identity as
  today); the trust root (allowed signers) enters the store as governed
  entries with an explicitly-named genesis entry; the file gate gains `L011`
  (required signature absent or invalid as of its signing timestamp) and
  `L012` (acceptances exist under a since-revoked key — a review trigger,
  never retroactive invalidation), its closed count of **ten becoming
  twelve** by the same amendment mechanism as `L010`. Hashing is unaffected:
  the signature is *over* the hash, not inside it — no digest moves,
  `CANONICAL_FORM` stays `v1`.
- **§9.4 — upstreams manifest.** A new file schema, not a change to these
  two. Closing the `based_on` vocabulary at that point **is** a hashed-meaning
  change and would require a `CANONICAL_FORM` bump, so the closure should
  arrive as validation over an unchanged canonical form instead.
