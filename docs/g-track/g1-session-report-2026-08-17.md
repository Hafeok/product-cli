# G-1 session report — reconcile, verify, initialise (2026-08-17)

**Session:** G-1, per the session prompt of 2026-08-17. Interactive; Emil rules at every gate.
**Branch:** `claude/g1-reconcile-verify-initialise-g0uno2` (all three repositories).
**This file grows gate by gate.** Step 1 (canon reconciliation) is below; Steps 2–4 append after
their gates open.

## Sources pinned (the staleness discipline, applied to this session's own inputs)

Every artefact this report projects is named here with its ref. A row without a ref would fail the
projection-as-source diagnostic the session prompt applies to its own outputs.

| Source | Ref / identity | Role |
|---|---|---|
| `actor-indexed-determination` (upstream canon) | `110bf10ff8fc0de0d71440310c869df78e34d8ef` — v5.5.0 (merge `e8663b8`, tagged 2026-08-16) plus two post-tag merges (PR #11 release CI, PR #12 session reconciliation); no canon content moved after the tag | reconciliation target |
| `decision-driven-design` (downstream canon) | `4848b9e15ea49bc923b2d23933e2c05a21202ba0` — merge of PR #21; `graph/upstream.yaml` pinned at v5.5.0 (DDD-dec-18) | reconciliation target |
| `product-cli` | `d506ac94310bb24e3c6a1b786034046ac0d024b0` (branch point) | the G-track home |
| PRD `prd-ground-as-ontology.md` | Emil's upload, 589 lines, written 2026-08-14 against holding-note revisions 13–14 | the document reconciled |
| Ground-axes holding note | Emil's upload, revision 18 (2026-08-15), 2,093 lines | **context only — canon outranks it everywhere they touch** |
| `meta/corpus-test-results-2026-08-14.md` | downstream, at `4848b9e` | evidence, fetched not uploaded |
| `meta/vocabulary-delivery-session-2026-08-15.md` | downstream, at `4848b9e` | evidence, fetched not uploaded |
| `meta/session-reconciliation-2026-08-16.md` | upstream, at `110bf10` | evidence, fetched not uploaded |

Queue position: satisfied. The vocabulary-and-delivery scope merged as v5.5.0 (upstream PR #10,
downstream PR #21); this session reconciles against that canon, not against the holding note.

---

# Step 1 — canon reconciliation

## 1.1 What v5.5.0 filed, and the deltas against what the PRD assumed

The PRD was pinned to holding-note revisions 13–14. The vocabulary-and-delivery session filed from
holding-note revision 8 plus the applicability note of 2026-08-12, with every filing scoped to what
the corpus test of 2026-08-14 evidenced. The deltas that matter to the PRD:

| Canon object | What it says | Delta against the PRD's assumption |
|---|---|---|
| `DDD-ground-01` (upstream, normative, projected) | A governing decision must declare a **resolvable applicability predicate**, or explicit universal applicability; non-evaluation never silently becomes non-applicability; implemented axes are marked mechanically-evaluable or judgement-evaluable | The PRD assumed Q1's named-axis gate. Canon amended it: the gate is predicate-general, and factored axes are one implementation, not the ontology of every region. The corpus put beyond-region predicate cases at ~27–36% |
| `DDD-ground-01` evidence (the matched pair) | The axis-type mark is a **maturity state**, not a fixed type — an axis moved nameable→resolvable in ~2.5 weeks | This answers open ruling 12 in canon's favour, which the PRD's §9.1 treated as open. Erratum noted in canon: the corpus document's "five weeks" was wrong; the repo dates govern |
| `DDD-ground-02` (upstream, conceptual, projected) | Source coverage (covered · declared-empty · undeclared · unknown), resolution (resolved · deliberately-open · unknown), and assurance (adequate · inadequate · unknown) are **orthogonal**; Unknown is never a pass | The PRD's §6.2 used Q3's four-state typing as UI. Canon rules the orthogonal typing governing; the four states survive only as the corpus's recorded projection, with "inert" replaced by **declared-empty** (ruled in, zero corpus draws recorded as its evidence status; the filing of a declared-empty is a claim-layer act) |
| `DDD-ground-03` (upstream, conceptual, projected) | Timing carries a fourth value, **"—(open)"**, for decisions whose resolution is deliberately-open | New since the PRD; touches the status line and any timing display |
| `DDD-ground-04` (upstream, normative, projected) | A retro-filed decision carries **two fields** — when the gap was uncovered, and that it was retro-filed | Ruled in, as the PRD hoped. Replaces holding-note §13.4 as the authority for §5.3's retro-filed decision set. Both sub-rulings recorded: retro-filing is the ledger-side discharge mechanism for the escape generators, and the retro-filing act is a claim-layer act |
| `DDD-delivery-01…03` (upstream) + `DDD-delivery-04` (downstream) + `core/13-delivery.md` + `term:delivery` / `term:undelivered` / `term:presumed-discharge` | Filing is not encoding; undelivered governance is **escape that presents as governance** (a generator, answering Q18's open question); unretrieved decision + unretrieved check are correlated failures; maturation's paid-once property holds only where the channel delivers per act-site | The PRD's delivery references (Q15/Q16/Q18/Q19) re-pin to canon terms and claims. The delivered-vs-emitted comparison the PRD names as its primary evidence output now has its stake in canon (`DDD-delivery-02`) |
| Term-collision repair | `00-primitives`' closing aside renamed "A note on delivery" → "A note on **presentation**" — the registry owns "delivery" | The PRD's uses of "delivery" were audited: every instance is in canon's sense (how authored governance reaches an act). No edit required. Spelling: canonical texts take "judgment"; claim and projection prose keeps British "judgement" — the PRD conforms |
| `graph/axis-registry.yaml` (downstream) | axis-registry/v1; **artefact-not-canon**; 22 axes (19 resolvable, 1 resolvable-partial, 2 nameable); promotion path: a validator reads it plus a ratification act | The first concrete axis registry exists — the framework program's own instance, not the G-track registry. It is the format precedent G0's proposal graphs inherit: quality marks and extractor sketches per axis |

One finding the PRD did not anticipate, in the other direction:

**The provenance typing is not in canon.** The PRD's §4.1 calls the four-value set (controlled /
observed / inferred / institutional) "canon's provenance typing", and §10 listed it among ratified
canon the PRD stands on without qualification. A search of both repositories at the pinned commits
finds no such typing — not in the core documents, not in `core/graph/terms.yaml`, not in any claim.
The holding note itself presupposes it rather than defining it (Q12 "the existing provenance typing
already supplies the scaling"; Q27 says canon "has carried this slot empty since the foundation
document"). The mechanism in the PRD stands; its basis is unratified. Marked in the PRD as
UNVERIFIED — Emil review, with two resolution paths: file the typing upstream before G0 consumes
it, or let the G-track SHACL shapes own the value set as track vocabulary with no canon claim
behind it. This is a mis-pin, not a design collision.

## 1.2 The §10 walk, row by row

Statuses read against upstream `110bf10` and downstream `4848b9e`. The PRD's fallback column
applies only on rejection; nothing was rejected, so no fallback fires.

| §10 construct | Status | Canon citation / absence | PRD sections touched |
|---|---|---|---|
| Position / region retrieval (Q2, Q19) | **ratified amended** (gate half); not filed (retrieval mechanism) | `DDD-ground-01` — predicate-or-explicit-universal, axis marking; region states axes are one implementation. No claim files the evaluator or subsumption retrieval itself | §7.2 (edits 2, 7), §7.3 |
| Extraction verification (Q20) | **not filed** | No claim; `DDD-ground-01`'s marking clause anchors the closure commitment; open ruling 13 open | none — dependency stands as flagged |
| Reading three-tuple (Q11 amended) | **not filed** | Assurance appears only as `DDD-ground-02`'s orthogonal property of ground; the per-reading tuple is note-only | §4.1 (provenance note, edit 11) |
| Trust decision backing (Q27) | **not filed** | Open ruling 27 open | none |
| Emitted proxy (ruling 22) | **not filed** as obligation; stake now canon | Open ruling 22 open; `DDD-delivery-02` + `term:presumed-discharge` file the comparison's stake | §4.2 (edit 8) |
| Registry = existing ontology (ruling 25, A1) | **not filed** | Open ruling 25 explicitly open — "the fork … the retrieval PRD waits behind" | none — A1 stands as assumption |
| Triangulation-with-independence (Q27 amended) | **not filed** | No filing | none |
| Decay-of-relevance under a pin (Q21) | **not filed** | Open rulings 14, 15 open | none |

## 1.3 Named holding-note items outside §10

| Item | PRD section | Status | Canon citation / absence | Edit |
|---|---|---|---|---|
| Q1 gate at proposal time | §4.3 | **ratified amended** | `DDD-ground-01` — named-axis became resolvable-predicate-or-explicit-universal | edit 2 |
| Q3 four-state typing | §6.2, §7.6 | **ratified amended** | `DDD-ground-02` — orthogonal typing governs; four states demoted to recorded projection; declared-empty ruled in; `DDD-ground-03` adds "—(open)" | edits 3, 4 |
| Q6 declaration discipline | §6.1 declare row | **not filed** | Precedence discipline is note-only; `DDD-ground-04`'s before/after evidential-status reasoning is the adjacent canon | none |
| Q11 three-tuple | §4.1 | **not filed** | as §10 row above | edit 11 (provenance only) |
| Q21/Q12 remedies | §6.1 declare row, §5.5 | **not filed**; the note's Q12/Q18 tension is resolved | `DDD-delivery-02` rules undelivered a generator of escape; ground-not-as-expected stays outside escape — both rulings now compatible, delivery is what distinguishes them | none |
| Q23 contract | §4.2 | **not filed**; one field anchored | `unevaluated_axes` is `DDD-ground-01`'s non-evaluation clause mechanised; the exposure profile stays note-only | edit 7 |
| Q26 ontology reading | §3, substrate | **not filed** | The design substrate remains unratified; axis-registry/v1 is the first concrete instance (artefact) | edit 9 |
| Q27 trust decisions | §4.4 | **not filed** | as §10 row above | none |
| Q30 authority/projection, "ground registry" | §3, §5.3 | **not filed** as canon term | Open ruling 32 is a freight-list item; Emil's rulings embedded in the PRD header are settled and unaffected | none |

## 1.4 The re-pin table — edits applied to the PRD copy

The edited PRD is `docs/g-track/prd-ground-as-ontology.md` on this branch. Every edit carries the
marker *(re-pinned at G-1)* in place. COLLISION count: **0**. One basis error (provenance typing)
is marked UNVERIFIED — Emil review rather than COLLISION, because it is a mis-pin, not a design
conflict between canon and the PRD.

| # | PRD section | Construct | Canon status | Edit |
|---|---|---|---|---|
| 1 | Provenance header | pinning | — | re-pin note added: commits named, holding note demoted to context |
| 2 | §4.3 proposed-decisions row | Q1 gate | ratified amended | gate restated as `DDD-ground-01`'s predicate-or-explicit-universal with axis marking |
| 3 | §6.2 status line | Q3 typing | ratified amended | orthogonal typing governing; four-state kept as canon's recorded projection; inert→declared-empty, uncovered-undeclared→undeclared; "—(open)" added (`DDD-ground-03`) |
| 4 | §7.6 halt row 2 | Q3 typing | ratified amended | trigger restated as source coverage = `undeclared`; escalation options renamed to canon values |
| 5 | §5.3 identity decision | retro-filing §13.4 | ratified as `DDD-ground-04` | authority re-pinned from the note to the claim; two fields and both Gate 4 sub-rulings cited |
| 6 | §9.1 evidence criterion | ruling 12 | answered in canon | criterion reframed from type-vs-maturity to transition rate; matched-pair evidence cited |
| 7 | §4.2 `unevaluated_axes`; §7.2 step 3 | Q23 field; Q2 regions | anchored / partially filed | `DDD-ground-01` citations added; predicate-generality of regions noted |
| 8 | §4.2 comparison; §7.2 close | delivery vocabulary | filed (draft) | stake cited to `DDD-delivery-02` / `term:presumed-discharge`; "holding note's Q19 sense" → `term:delivery`, `core/13-delivery.md` |
| 9 | §5.2 axis-registries row | axis registry format | artefact exists | axis-registry/v1 named as format precedent; quality-mark vocabulary adopted; scope distinction stated (framework's own instance, not this registry) |
| 10 | §10 table and closing line | all eight rows | as walked above | canon-status column added; provenance typing removed from the stands-without-qualification list; v5.5.0 filings added as named canon the PRD now stands on |
| 11 | §4.1 provenance row | provenance typing | **not in canon** | row re-marked as holding-note vocabulary; UNVERIFIED — Emil review note added with the two resolution paths |

## 1.5 What was checked and not edited

- The PRD's uses of "delivery" — all in canon's term sense; the term-collision repair requires
  nothing here.
- Emil's rulings embedded in the PRD header — settled; untouched.
- The PRD's remaining "verify at G-1" annotations (ratatui, GitHub Models, LogMap-class tooling,
  reasoner mechanics) — Step 2's work, not Step 1's.
- The fallback column of §10 — never applied; no construct was rejected.

**GATE 1 — closed** (Emil, 2026-08-17). Commit `077790d` stands; both structural amendments
accepted as canon's improvement on the draft. Rulings recorded:

1. **Provenance typing: the track owns it, pending the wave.** The value set does not file
   upstream from this session; the owning session is the queued Q25/Q27/Q30 filing wave (Q27's
   trust-decision mechanism is the backing institutional provenance needs). The G-track SHACL
   shapes own the value set as track vocabulary; filed as track decision `g-dec-01`
   (`docs/g-track/decisions/g-dec-01-reading-vocabulary.yaml`) with a `revisit_if` pinned to the
   wave. The finding is forwarded to the wave as evidence — its Q27 filing now has a named
   consumer waiting on it. The same disposition covers the Reading tuple's other unfiled halves
   (Q11 three-tuple, assurance-on-reading); one decision file carries all of it. The PRD's §4.1
   is marked down from "canon's provenance typing" to "track vocabulary, candidate for the Q27
   wave" — the mis-pin corrected in the artifact, not just the report.
2. **Status line: projection for the human, orthogonal for the machine.** §6.2 keeps the
   four-state projection (a status line is a projection for a human arrangement mid-act; compact
   and absorbable beats complete); halt logic and the act log run on the orthogonal values; a
   future detail view displays the orthogonal triple raw. Recorded in the PRD at §6.2.

---

# Step 2 — verification worklist (V1–V9)

*(Grows row by row; holding at GATE 2 when complete.)*
