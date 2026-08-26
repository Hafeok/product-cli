# C0 — field provenance audit

**Status:** `[PROPOSED]` — session output, not ratified.
**Dated:** 2026-08-26. **Gate:** 2b §3.1, required before Gate 3 opens.
**Correction owned by the principal, recorded against the PRD:** PRD §2 mechanism 1 ("factor from the
schema, not by extraction … the claim model in r2 §7.1 came out of the review note's general
decision-to-code argument, before either corpus was examined") is **false for at least one field**.
This audit establishes for how many.

---

## The audit's own limitation, stated before its results

r2 §7.1 is **not reachable from this session**. It is in neither `Hafeok/product-cli` nor
`Hafeok/decision-driven-design`. Every row below therefore records **attributed** provenance — what
the C-track PRD says about a field's origin — and not **verified** provenance. No row can be checked
against r2 as filed.

The audit is consequently sound in one direction only:

- Where the PRD **marks a field as post-corpus**, that is reliable — the PRD would not invent a
  corpus origin for a field that had none.
- Where the PRD **attributes a field to r2**, that is *unverified*. A field silently added at Annex A
  and then listed without a marker is invisible to this audit, and §4's count of one such field
  proves the failure mode is live rather than hypothetical.

**So the true count of post-corpus elements is a lower bound.** Verification requires r2.

Provenance codes:

| Code | Meaning | Mechanism 1 protection |
|---|---|---|
| **R** | Attributed by the PRD to r2 §7.1. Unverified. | Claimed |
| **A4** | Post-corpus, identified. Entered at Annex A A-4, after the corpus probe. | **Lost** |
| **A6** | Post-corpus, identified. Entered at Annex A A-6. | **Lost** |
| **A?** | Post-corpus by the PRD's own count, **entry point unidentified**. | **Lost** |
| **P** | Authored in the C-track PRD itself; not in r2, not corpus-derived. | **Lost** |
| **X** | Imported from a shipped implementation. | **Lost**, and by a route mechanism 1 names as forbidden |
| **S** | Session-derived at Gate 2. Not in any schema. | **Lost** |

"Substrate-exposed" records whether the element's *disposition* was formed after this session had read
`ledger-core` and `ddd-core`. Gate 0 read both in depth before Gate 2 wrote §1. Writing order was
schema-then-map as ratified; **reading order was not, and could not have been** — Gate 0 required it.

---

## 1. Claim-model fields (PRD §4.1)

| # | Element | Code | Entry point / evidence | Substrate-exposed |
|---|---|---|---|---|
| 1 | Decision identity | **R** | PRD §4.1 table | no |
| 1a | — its *opacity* qualifier ("opaque to the core; the corpus supplies the identifier scheme") | **A?** | Unattributed. A qualifier one learns is needed by meeting a corpus whose identifier scheme resists typing. **Candidate for the unidentified second A-4 addition** (§4). | no |
| 2 | Authoritative source | **R** | PRD §4.1 table | no |
| 3 | Source version | **R** | PRD §4.1 table | no |
| 4 | **Embodiment** | **A4** | PRD §4.1: "the two Annex A A-4 additions". The only row the PRD **bolds**. | no |
| 5 | **Authority grade** | **A4** | PRD §4.1: "generalised from `eli:legal_value`" — the field's own definition names a corpus vocabulary. | no |
| 6 | Applicability, as a field | **R** | PRD §4.1 table | no |
| 6a | — its five slots (actor, act, position, time, other) | **R**, contested | Attributed to r2, but norm-shaped; C-O-7. | no |
| 7 | Interpretation | **R** | PRD §4.1 table | no |
| 8 | Residue, incl. empty-vs-absent | **R** | PRD §4.1 table | no |
| 9 | Implementation binding | **R** | PRD §4.1 table | no |
| 10 | Verification binding | **R** | PRD §4.1 table | no |
| 11 | Principal, as a field | **R** | PRD §4.1 table | no |
| 11a | — the **"per boundary, not per claim"** quantifier | **A6** | Presupposes the three boundaries, which the PRD attributes to Annex A A-6. The field may be r2's; the quantifier cannot be. | no |
| 12 | Supersession | **R** | PRD §4.1 table | no |

## 2. Boundaries (PRD §4.3)

| # | Element | Code | Entry point / evidence | Substrate-exposed |
|---|---|---|---|---|
| 13 | The three transcription boundaries | **A6** | PRD §4.3: "Three, per Annex A A-6" | no |
| 14 | Per-boundary separate ratification | **A6** | PRD §4.3 | no |
| 15 | Each boundary carries its own principal | **A6** | PRD §4.3 | no |
| 16 | Boundary 0 (ingest) excluded as corpus-specific | **A6** | PRD §3, §4.3 | no |
| 17 | Competence condition routed (C-O-3) | **A6** | PRD §4.3 | no |

## 3. Mechanisms and queries (PRD §4.2, §4.4–4.7, §2)

| # | Element | Code | Entry point / evidence | Substrate-exposed |
|---|---|---|---|---|
| 18 | Refusal — incomplete governance record | **P** | PRD §4.2 | no |
| 19 | **Refusal — model identity as accepting principal** | **X** | PRD §4.2: "**the L006 pattern**". L006 is a `ledger-core` verify class. The PRD imports a refusal from a shipped implementation and names it as the source. | n/a — the PRD did the importing |
| 20 | Refusal — one signature, two boundaries | **A6** | Unrepresentable without the boundaries | no |
| 21 | Refusal — rewrite refused, supersession only | **P** | PRD §4.2 | no |
| 22 | Refusal — absent residue | **R** | Follows field 8 | no |
| 23 | Three discharge states (delivered/presumed/escaped) | **P** | PRD §4.4; unattributed | no |
| 24 | "Presumed is not evidence" | **P** | PRD §4.4. See `c0-upstream-basis-2026-08-26.md` §1 — upstream canon has this. | no |
| 25 | Four coverage/gap states | **P** | PRD §4.5; unattributed. Now amended — §1.4 of the boundary. | no |
| 26 | Audit separation (process validity vs semantic evidence) | **P** | PRD §4.6 | no |
| 27 | Event log — "the acts are the events" | **P** | PRD §4.7, self-marked `[PROPOSED]` | no |
| 28 | Mechanism 1 — factor from the schema | **P** | PRD §2. **Self-refuting per this audit.** | no |
| 29 | Mechanism 2 — no dependent may be a dependency | **P** | PRD §2. Asserted as existing machinery; it does not exist (principal's Gate 2 correction). Instance of upstream `DDD-dec-04`. | no |
| 30 | Mechanism 3 — two synthetic corpora | **P** | PRD §2, self-marked `[PROPOSED]` | no |
| 31 | Mutation harness (C5) | **P** | PRD §3, §5, §6 | no |

## 4. Session-derived at Gate 2

Every row here is **S**: in no schema, and therefore never under mechanism 1's protection at all.

| # | Element | Substrate-exposed | Note |
|---|---|---|---|
| 32 | The four-disposition vocabulary (in-core / in-core-opaque / in-track / routed) | **yes** | "in-core, opaque" was justified in the boundary by citing `ledger-core`'s `BasisRef`/`RevisitRef`. Flagged there as precedented rather than invented; under audit it is **substrate-derived**. |
| 33 | Applicability as corpus-declared dimensions (C-O-7) | no | Held by the principal pending §3.3 |
| 34 | Boundary chain as corpus-declared and variable (C-O-8) | no | Held; restated in `c0-refusal-over-chain-2026-08-26.md` |
| 35 | The `inapplicable` fourth state | no | Derived from corpus B. Ratified. |
| 36 | Version precedence routed; FC-3 split (C-O-11) | no | Ratified |
| 37 | Embodiment disposition narrowed (field + refusal in-core; vocabulary in-track; predicate routed) | **yes** | The open-unresolved-vocabulary pattern is `ledger-core`'s |
| 38 | Implementation binding carries an optional content pin | **yes** | Shape taken from `ddd-core`'s `SeamBinding` |
| 39 | FC-1 check 1 — dependency closure | no | |
| 40 | FC-1 check 2 — vocabulary tripwire | no | |
| 41 | FC-1 check 3 — exercise join | **yes** | Shape taken from `product-cli`'s §14.4 completeness join |
| 42 | FC-1 step 5 — "exercised by one must not fail" | no | Ratified as canon |
| 43 | Corpora — Bandmaster, Steward | no | Ratified |
| 44 | C-O-9 control corpus | no | Promoted to a C0 completion condition |

---

## 5. Results

### 5.1 Elements that have lost mechanism 1's protection

**Not re-argued here. Merits are a separate act; this records provenance only.**

Counted as distinct elements, and **excluding** the 13 session-derived rows (§4), which were never
under mechanism 1:

| Route | Count | Elements |
|---|---|---|
| **A4** — post-corpus, identified | 2 | 4, 5 |
| **A6** — post-corpus, identified | 6 | 11a, 13, 14, 15, 16, 17, 20 *(seven rows; 20 follows from 13–15 and is counted once with them)* |
| **A?** — post-corpus, entry unidentified | 1 | 1a |
| **P** — PRD-authored | 13 | 18, 21, 23, 24, 25, 26, 27, 28, 29, 30, 31 |
| **X** — imported from a shipped implementation | 1 | 19 |

**Of the 12 claim-model fields the PRD presents as "fields per r2 §7.1", two are A-4 additions the PRD
itself marks, one is a quantifier that cannot predate A-6, and one qualifier is unattributed.**
Everything in PRD §4.2–§4.7 and §2 — the entire mechanism layer, including all three anti-shaping
mechanisms and the falsifier apparatus — is **P**, PRD-authored, and was never in r2 either.

Restated at the altitude that matters: **mechanism 1 protects the claim-model field list and
essentially nothing else.** The PRD reads as though "factored from the schema" covers the design; it
covers roughly a dozen field names, eight of which survive scrutiny unqualified.

### 5.2 The unidentified second A-4 addition — an open item, not a finding

PRD §4.1 states "the two Annex A A-4 additions" and **bolds exactly one row**. The second is
unmarked and unidentifiable from any document reachable here.

Row 1a — the decision-identity opacity qualifier — is this audit's **candidate**, on the reasoning
that opacity is a property one discovers is necessary by meeting an identifier scheme that resists
typing, which is a corpus encounter. **It is a candidate and not a determination.** Resolving it
requires Annex A A-4. Filed as **C-O-12**.

Until resolved, one A-4-originating element is somewhere in §1 wearing an **R**, and this audit
cannot say which. That is the audit's own falsifier: if A-4 shows the second addition to be a field
this table marks **R**, the table was wrong about that row, and the "**R** is unverified" caveat at
the top was load-bearing rather than decorative.

### 5.3 A second breach of mechanism 1, in a direction it does not name

Row 19. PRD §4.2 sources one of its five refusals to **"the L006 pattern"** — a verify class in
`ledger-core`, a shipped implementation.

Mechanism 1 forbids "extraction from a shipped track as a factoring method". `ledger-core` is not a
track, so the letter is intact. The principal closed that gap in-session by ruling that the C-track
must not factor by extraction from `ledger-core` either — **but the PRD had already done so before
this session began, and says as much in the text.**

The refusal is correct and the import is a good one. That is not the point. The point is that
mechanism 1's protection was already breached in two directions — once toward a corpus (rows 4, 5),
once toward the substrate (row 19) — before the session that mechanism 1 was written to constrain
had started.

### 5.4 The session's own exposure

Rows 32, 37, 38 and 41 formed after Gate 0 had read `ledger-core` and `ddd-core` in depth. The Gate 2
claim that "§1 was written before §2 and did not revise it" is true as stated and is **not** a claim
to have factored unexposed. No such claim can be made: Gate 0 mandates reading the workspace, so by
Gate 2 the substrate is known. Mechanism 1 cannot be discharged by a session whose Gate 0 requires
the exposure mechanism 1 is meant to prevent.

**This is a defect in the gate order, not in the session's conduct, and it is filed as C-O-13.** A
future factoring session that wants mechanism 1's protection must do the schema derivation **before**
its live-state gate, or delegate the two to different readers.
