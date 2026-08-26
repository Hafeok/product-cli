# C0 — upstream basis: cite, do not restate

**Status:** `[PROPOSED]` — session output, not ratified.
**Dated:** 2026-08-26. **Gate:** 2b §3.4.
**Read at:** `Hafeok/decision-driven-design` @ `efb4668` (shallow clone; see `bootstrap.md` for the
limit that places on these citations).
**Nothing is filed upstream. Routing is a separate act.**

Two independent statements of one claim is drift, and the drift validator does not run across
repositories, so nothing would catch it. Every C-track falsifier was therefore checked against
upstream canon.

---

## 1. FC-5 — cite, do not restate

**Upstream basis: `term:presumed-discharge`.**

| | |
|---|---|
| Established | The principle repository, `core/13-delivery.md` — per `DDD-dec-18`, which records the adoption of "the delivery area (`core/13-delivery.md` establishing `term:delivery`, `term:undelivered`, `term:presumed-discharge`)" |
| Adopted here | `DDD-dec-18` |
| Applied | `DDD-dec-20` notes; `DDD-dec-23`; `DDD-dec-27` (as a `basedOn` edge); `DDD-dec-29`; `projections/tracks/README.md` |
| Canonical phrasing | "the artefact recording the pass identical to the artefact recording the skip" (`DDD-dec-18`, `DDD-dec-20`) |

**`[PROPOSED]`: FC-5 is withdrawn as an independent falsifier statement and re-filed as an
application of `term:presumed-discharge` to the claim register.** The C-track's contribution is the
*instance* — a query path counting a presumed discharge toward delivered coverage — not the claim,
which is upstream and older.

### 1.1 An observation about the pin, offered with its uncertainty

`DDD-dec-20`'s note cites the term as "(`term:presumed-discharge`, pinned)". At `efb4668`,
`graph/upstream.yaml` lists 30-odd `term:` pins and **`term:presumed-discharge` is not among them**.

**This is reported, not asserted as a finding.** The term was *adopted* via `DDD-dec-18` rather than
merely referenced, and this session does not know whether adopted terms are pinned by a different
route, nor whether a shallow clone at one commit shows the whole pin set. `DDD-dec-27` does carry it
as a `basedOn` edge, so it is load-bearing somewhere.

If it is genuinely unpinned it is the case `DDD-dec-04` and the GATE 6 basis-impact sweep exist to
catch — "a basis cited without a pin is a basis that can move silently" — which would make it an
instance of the repository's own rule, found from downstream. **Not filed. Routing is a separate act,
and this session is outside that repository's scope by `DDD-dec-20`'s own clause.**

## 2. FC-3 — no duplication, but substantial prior art to copy from

**No upstream falsifier states FC-3.** But `validate-core-order.py` implements the same mechanism
over `graph/upstream.yaml`, and it is further along than the C-track's design:

| Class | What it catches | C-track relevance |
|---|---|---|
| **E12** | A pinned id no longer exists upstream at the pinned ref | Source withdrawn |
| **W5** | **Basis loss** — upstream *status* moved since the pin was set | The staleness propagation FC-3 is about |
| **E13** | A document embedding a pinned id must match upstream byte-for-byte | Embodiment drift, in the §4.1 sense |

The pin record carries `status_at_pin` **and** `content_hash` — independently the same shape the
boundary proposed at §1.1 and §1.6 (opaque version token plus optional content pin), arrived at from
the schema without knowledge of this file. Worth recording as convergence, not as derivation.

**One design note to import verbatim**, from `validate-core-order.py`:

> Status is NOT hashed — W5 already instruments it, and folding it in here would make one class fire
> twice for the same movement.

That is C-O-11's problem already solved once: when staleness splits into a core failure and a corpus
failure, the two classes must instrument disjoint things or one movement reports twice. The C-track
should adopt the rule, not rediscover it.

## 3. §2 mechanism 2 — cite `DDD-dec-04`

`DDD-dec-04`: *"SDP direction enforced: the principle repo carries no reference to its dependents.
Cross-repo edges are pinned at version and status with basis-loss detection (W5), per `DDD-agent-01`
applied to repositories."*

The C-track PRD §1 already carries this as "Ruling (sibling decision): SDP direction enforced". The
`[PROPOSED]` correction is only that the citation should be explicit: **mechanism 2 is an instance of
`DDD-dec-04` applied to crates rather than repositories**, not a new rule. That matters for the C1
acceptance condition — the `xtask` check being built is the crate-level analogue of a repository-level
mechanism that already exists and already runs, and it should be recognisably the same rule.

## 4. FC-1, FC-2, FC-4 — no duplication found

All 26 falsifiers in `core/claims/` were read. They are cost-model, maturation, organisational-
discipline, simulation and learning-track claims — a different layer entirely. **None duplicates FC-1
(factoring), FC-2 (refusal completeness) or FC-4 (mutation localisation).**

`DDD-tool-01` is the nearest neighbour and is not a duplicate: its falsifier is about whether staleness
still requires manual audit after a canon-revision cycle — a claim about the tool's maintenance
economics, not about a register's integrity.

**Coverage limit, stated:** this checked `core/claims/`, `core/decisions/`, `spec/`, `projections/`
and `graph/` at one commit in one repository. The principle repository
(`actor-indexed-determination`) was **not** cloned and **not** searched. Since `term:presumed-discharge`
turned out to originate there, the principle repository is where a further duplicate would most
likely be, and this check does not cover it. Filed as **C-O-15**.

## 5. The convergence, noted and not built on

The principal's instruction: note it, do not build on it.

`term:presumed-discharge` appears in upstream canon **eight days before** the C-track PRD named FC-5,
in a different domain (session-arrival records and canon pinning), derived from five observed arrival
failures rather than from the claim schema.

That is stronger evidence for the claim than the `ledger-core` overlap was. Common authorship weakens
agreement — same author, same framework, same year — whereas an independent domain reaching the same
distinction from different evidence strengthens it.

**Not built on.** It is one convergence, within one programme, and the author is common even where the
domain is not. It is recorded as an observation about the claim's robustness, and no C-track element
rests on it.
