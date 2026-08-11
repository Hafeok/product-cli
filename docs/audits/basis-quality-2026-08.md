# Basis-quality audit — pre-format-5 decisions (2026-08)

**What is under test.** `DDD-method-06` — *mandatory claim-basis manufactures
weak claims* — filed `projected` when typed basis (`dec/ddd/typed-basis`,
decision format 5) was ruled, with this falsifier:

> An audit of basis quality after typed basis (format 5) ships: if the
> pre-format-5 decisions' claim bases audit as genuine — each basedOn claim
> would have been filed on its own merits, none exists only to satisfy rule 3
> — the manufactured-claims mechanism did not operate here and the claim dies.

This document is that audit. **Section 1 was written and committed before any
rationale text was read or any classification made.** Nothing in it was
adjusted after results were seen; if the criteria look badly chosen in
hindsight, that is a finding about the pre-registration, not a licence to
re-cut it.

---

## 1. Method, as pre-registered

*(Committed before the data was gathered. Fixed.)*

### 1.1 Population

Every decision file under `.ddd/decisions/` whose `format:` field is **less
than 5** — i.e. filed before decision format 5, the format that made basis
typed. In such a file every `based_on` entry is an untyped `claim: <id>`
reference, because rule 3 admitted nothing else.

Format-5 decisions are **excluded**: they were filed after the affordance
existed, so they cannot exhibit the mechanism under test.

The unit of the headline reading is the **decision**. The unit of the table is
the **citation edge** (decision → claim).

### 1.2 Per cited claim, record — mechanically

| Signal | How it is computed |
|---|---|
| **Co-filing** | The commit that introduced the claim file (`git log --diff-filter=A -1`) compared with the commit that introduced the citing decision file. Same commit → `co-filed`. Different → `pre-existing`, with both hashes recorded. |
| **Citation count** | Number of **distinct decisions** (any format, whole store) whose `based_on` names the claim. `1` = single-purpose. |
| **Falsifier** | `present` / `absent` / `unfalsifiable`. See 1.3. |
| **Status** | The claim's `status:` field verbatim. |
| **Evidence** | Whether an `evidence:` field is present. |
| **Basis classification** | What the basis actually was, read from the **decision's own `rationale` / `title` text** — never from the claim, never from the `type` field. One quoted phrase per row, the phrase that decided it. See 1.4. |

### 1.3 Falsifier grading — criteria fixed in advance

A falsifier is **`present-but-unfalsifiable`** if no observation of this repo,
its toolchain, or its use could ever produce the stated condition. Three
disqualifiers, one word recorded per claim:

- **`restatement`** — it names no observable event, only the negation of the
  statement ("if X turns out not to hold").
- **`vapour`** — it is conditioned on an artifact or milestone that does not
  exist and has no scheduled existence, so the observation can never be made.
- **`taste`** — it requires a judgement of quality or intent with no stated
  threshold, so no observation settles it.

Everything else with a `falsifier:` field is **`present`**. A falsifier that is
merely *unlikely to fire*, or that names a future-but-scheduled milestone, is
`present` — hard to falsify is not unfalsifiable.

### 1.4 Basis vocabulary — definitions fixed in advance

Read from the decision's rationale text. The question is *what the rationale
actually argues from*, not what it cites.

- **claim** — the rationale argues from a **contestable proposition about the
  world** that could be shown false by observation, and the decision follows
  from it. Form: "X does/does not Y", "X is sufficient for Y".
- **constraint** — the rationale argues from something the project **cannot
  change**: a language, tool, or format property; a host-system fact. Form:
  "Rust has no X", "the format requires Y".
- **mandate** — the rationale argues from an **authority requiring it**: a
  principal's ruling, a spec, an upstream or house rule. Form: "ruled at
  review", "the PRD requires", "policy is".
- **preference** — the rationale argues from **taste, ergonomics, or style**,
  with no falsifiable proposition. Form: "cleaner", "reads better", "we prefer".
- **experiment** — the rationale is explicitly **provisional, adopted to find
  out**. Form: "try it and see", "for now".
- **risk-acceptance** — the rationale **acknowledges a known cost or gap and
  prices it**. Form: "priced, not paid", "accepted cost", "we take the hit".
- **indeterminate** — the text supports two or more of the above and **no
  phrase discriminates them**. Recorded as a result, not resolved by guessing.

A row where the classification was genuinely close is additionally marked
**borderline** and listed in §5 with its quote, for the principal's ruling. It
still receives its best-reading classification in the table; borderline is an
annotation, not a third value.

### 1.5 Weak-claim composite — fixed in advance

A cited claim is **weak** iff it meets **two or more** of:

- **W1** — co-filed with the citing decision in the same commit;
- **W2** — cited by exactly one decision;
- **W3** — falsifier `absent` or `unfalsifiable`;
- **W4** — status `projected` **and** no evidence attached.

Two-of-four, not one: any single signal has an innocent reading (a claim filed
alongside its first consumer is normal practice; a claim cited once may simply
be young). Two together is the pattern the claim predicts.

### 1.6 The reading

A citation edge is **manufactured** iff **both**:

- the citing decision's basis classification is **not `claim`** (it is mandate,
  preference, experiment, risk-acceptance, or constraint), **and**
- the cited claim is **weak** by §1.5.

A **decision** counts as a **manufactured-basis decision** iff **every** claim
it cites is manufactured. The `every` is deliberate and tight: if a decision
cites one genuine claim alongside one weak one, rule 3 was already satisfied by
the genuine one and nothing needed inventing. Only a decision with no genuine
claim to stand on was *forced* to manufacture.

**The audit's result is that count, over the population.**

### 1.7 Verdict bands — fixed in advance

Chosen before the count was known:

| Count of manufactured-basis decisions | Verdict | Proposed status |
|---|---|---|
| **0** | The falsifier fired exactly as written. | `retired` |
| **1–3** (≤ ~7%) | Mechanism operated, but marginally. Report it as *held weakly* and say plainly that the typed-basis ruling rests mostly on its ergonomic argument. | `reported`, weakly |
| **≥ 4** (≥ ~10%) | Prediction held. | `reported` |

**Underpowered** is claimed only if **both**: fewer than 10 decisions in the
population classify as non-`claim` basis, **and** more than 30% of
classifications come out `indeterminate` — i.e. the rationale texts cannot
discriminate the types at all. Population size alone is not grounds; 41 is not
a small N for this question.

### 1.8 Constraints on this audit

Nothing in the store is changed. No re-typing of bases, no claim edits, no
retirements, no status moves, no cleanup of anything noticed. Every finding in
§6 is a **proposal for the principal's ruling**, filed unactioned.

---

## 2. Population

> *Note on §1:* the pre-registered text above is left byte-identical to what was
> committed in `9488ba6`, including two cross-references that the final section
> numbering moved — §1.4's "listed in §5" is now §6, and §1.8's "finding in §6"
> is now §8. Correcting them in place would edit a committed pre-registration;
> they are pointed out here instead.

**41 decisions**, every one under `.ddd/decisions/` at `format: 2`. The store
holds 44 decision files; the three at `format: 5` — `dec/ddd/typed-basis`,
`dec/ddd/prd-split`, `dec/ddd/m8-enforcement-closure` — are excluded per §1.1.
No decision sits at format 1, 3 or 4.

All 41 cite at least one claim; none cites anything else, because rule 3
admitted nothing else. Together they make **52 citation edges** over **41
distinct claims** (of the 77 claims in the store).

Mechanical totals over the 52 edges:

| Signal | Count |
|---|---|
| Co-filed with the citing decision (same commit) | 31 / 52 |
| Cited claim is single-purpose (1 citing decision) | 31 / 52 |
| Cited claim `projected` with no evidence | 17 / 52 |
| Cited claim has no `falsifier` field at all | **0** / 52 |
| Cited claim's falsifier graded unfalsifiable (§1.3) | 8 / 52 (4 distinct claims) |

---

## 3. The per-claim table

`W` lists which of the §1.5 signals fired. **Weak** = two or more. **Manuf.**
= the §1.6 conjunction: non-`claim` basis **and** weak claim.

| Decision (pre-format-5) | Cited claim | Co-filed | Cites | Falsifier | Status / ev. | W | Weak | Basis read from rationale | Manuf. |
|---|---|---|---|---|---|---|---|---|---|
| `bicep/linter-at-error` | DDD-bicep-ver-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `bicep/linter-at-error` | DDD-bicep-sec-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `bicep/linter-at-error` | DDD-bicep-sec-02 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `bicep/psrule-in-pr-stage` | DDD-bicep-net-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `bicep/psrule-in-pr-stage` | DDD-bicep-diag-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/async-policy-rules-at-error` | DDD-cs-pol-03 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/async-policy-rules-at-error` | DDD-cs-pol-04 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/authorization-fallback-deny` | DDD-cs-authz-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/disposal-rules-at-error` | DDD-cs-disp-02 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/disposal-rules-at-error` | DDD-cs-disp-05 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/general-catch-at-error` | DDD-cs-fail-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `cs/policy-rules-at-error` | DDD-cs-pol-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `cs/startup-assertions-required` | DDD-cs-map-01 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/startup-assertions-required` | DDD-cs-conc-02 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/unsafe-reinterpretation-banned` | DDD-cs-tc-04 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `cs/unsafe-reinterpretation-banned` | DDD-cs-tc-05 | no | 1 | present | reported + ev | W2 | no | claim |  |
| `ddd/adapter-policy-tables` | DDD-adapter-01 | **yes** | 2 | present | established + ev | W1 | no | claim |  |
| `ddd/amend-explicit-evidence-frozen` | DDD-friction-02 | **yes** | 2 | present | projected / no ev | W1 W4 | **weak** | claim |  |
| `ddd/amend-explicit-evidence-frozen` | DDD-arch-04 | **yes** | 1 | unfalsifiable (vapour) | projected / no ev | W1 W2 W3 W4 | **weak** | claim |  |
| `ddd/correspondence-rows-are-stratified` | DDD-arch-06 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `ddd/curation-over-mining` | DDD-method-02 | **yes** | 2 | present | reported + ev | W1 | no | claim |  |
| `ddd/enforce-matching-tightens-to-symbol` | DDD-arch-05 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `ddd/enum-member-gap-priced` | DDD-adapter-03 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `ddd/fixtures-not-sdk` | DDD-detect-02 | **yes** | 2 | present | reported + ev | W1 | no | claim |  |
| `ddd/git-is-the-amend-trail` | DDD-method-04 | **yes** | 1 | present | projected / no ev | W1 W2 W4 | **weak** | constraint | **YES** |
| `ddd/interceptor-not-extension` | DDD-arch-03 | **yes** | 1 | present | projected / no ev | W1 W2 W4 | **weak** | indeterminate |  |
| `ddd/internal-not-surface` | DDD-adapter-02 | **yes** | 1 | present | projected / no ev | W1 W2 W4 | **weak** | mandate | **YES** |
| `ddd/lsp-as-seam` | DDD-arch-02 | **yes** | 2 | present | projected / no ev | W1 W4 | **weak** | claim |  |
| `ddd/m6-proceeds-no-flip` | DDD-adapter-01 | no | 2 | present | established + ev | — | no | mandate |  |
| `ddd/pins-at-m2` | DDD-method-02 | no | 2 | present | reported + ev | — | no | claim |  |
| `ddd/predicates-carry-no-status` | DDD-method-01 | **yes** | 1 | unfalsifiable (vapour) | reported + ev | W1 W2 W3 | **weak** | mandate | **YES** |
| `ddd/rejection-facts-prefilled` | DDD-friction-01 | **yes** | 2 | present | projected / no ev | W1 W4 | **weak** | claim |  |
| `ddd/report-coverage-explicit` | DDD-method-03 | **yes** | 2 | unfalsifiable (taste) | projected / no ev | W1 W3 W4 | **weak** | claim |  |
| `ddd/rust-class-enforced-here` | DDD-friction-01 | no | 2 | present | projected / no ev | W4 | no | experiment |  |
| `ddd/rust-class-enforced-here` | DDD-friction-02 | no | 2 | present | projected / no ev | W4 | no | experiment |  |
| `ddd/rust-host-is-real` | DDD-detect-02 | no | 2 | present | reported + ev | — | no | claim |  |
| `ddd/rust-host-is-real` | DDD-arch-02 | no | 2 | present | projected / no ev | W4 | no | claim |  |
| `ddd/sarif-unification` | DDD-detect-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `ddd/seed-lands-in-claims-not-shared` | DDD-method-05 | **yes** | 1 | unfalsifiable (vapour) | projected / no ev | W1 W2 W3 W4 | **weak** | claim |  |
| `ddd/v1-scope` | DDD-scope-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `ddd/what-boundaries-priced-not-paid` | DDD-what-04 | **yes** | 1 | present | projected / no ev | W1 W2 W4 | **weak** | risk-acceptance | **YES** |
| `ddd/what-policy-table` | DDD-what-01 | **yes** | 1 | present | projected / no ev | W1 W2 W4 | **weak** | claim |  |
| `ddd/what-policy-table` | DDD-what-03 | **yes** | 2 | present | reported + ev | W1 | no | claim |  |
| `ddd/what-published-qualifier` | DDD-what-03 | **yes** | 2 | present | reported + ev | W1 | no | claim |  |
| `ddd/why-resolves-three-ways` | DDD-method-03 | **yes** | 2 | unfalsifiable (taste) | projected / no ev | W1 W3 W4 | **weak** | claim |  |
| `ddd/workspace-member-delivery` | DDD-arch-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | claim |  |
| `rust/no-unwrap` | DDD-gates-01 | **yes** | 1 | present | reported + ev | W1 W2 | **weak** | constraint | **YES** |
| `web/htmlcss-enforce` | DDD-web-01 | **yes** | 3 | present | reported + ev | W1 | no | claim |  |
| `web/markup-validity` | DDD-web-01 | no | 3 | present | reported + ev | — | no | claim |  |
| `web/plain-pair-scope` | DDD-web-01 | **yes** | 3 | present | reported + ev | W1 | no | claim |  |
| `web/render-status-palette` | DDD-web-02 | **yes** | 2 | present | projected / no ev | W1 W4 | **weak** | claim |  |
| `web/token-discipline` | DDD-web-02 | **yes** | 2 | present | projected / no ev | W1 W4 | **weak** | claim |  |

### 3.1 Falsifier gradings that are not `present`

Four of the 41 cited claims were graded unfalsifiable under §1.3. Every other
cited claim carries a falsifier naming an observable event, so it is graded
`present` — including ones that are merely hard to fire (`DDD-adapter-02`,
`DDD-what-01`, `DDD-what-04`), per the §1.3 sentence that hard-to-falsify is
not unfalsifiable.

| Claim | Word | Why |
|---|---|---|
| `DDD-arch-04` | **vapour** | Falsifier asks for "a study over the rows in which declarant-supplied structural fields proved as reliable as the LSP-derived ones". `dec/ddd/amend-explicit-evidence-frozen` — the decision this claim grounds — abolished declarant-supplied structural fields, so the rows can never contain the contrast the study needs. Self-sealing. |
| `DDD-method-01` | **vapour** | Falsifier asks for "a predicate whose embedded status would have stayed accurate". `dec/ddd/predicates-carry-no-status` makes a status-bearing predicate a schema rejection (`deny_unknown_fields`), so the artifact the observation requires cannot exist in the store. Self-sealing. |
| `DDD-method-03` | **taste** | Falsifier asks for "a repo where surfacing the uncheckable set changed no reader's action". Reader action, no threshold, and no instrument named anywhere in the store that would settle it. |
| `DDD-method-05` | **vapour** | Falsifier asks for "a body of entries filed into `shared/` at authoring time whose observed transfer rate matched…". `dec/ddd/seed-lands-in-claims-not-shared` forbids filing into `shared/` at authoring time, so the body of entries cannot accrue here. Self-sealing. |

Worth recording as a pattern in its own right: **three of the four are
self-sealing in the same way** — the decision the claim grounds removes the
arrangement under which the claim's falsifier could ever be observed. That is
not the mechanism `DDD-method-06` predicts, and it is not counted toward the
reading. It is filed as finding **F-8**.

`DDD-method-02` was graded **present**, though close: its decisive term
("entries remain explainable to a named principal") is a judgement, but the
store names the instrument that produced its evidence — the PRD §2 warning-swamp
analysis — and the same instrument would settle the falsifier. Recorded as a
borderline grading in §5.

---

## 4. Basis classification, read from each decision's own rationale

One row per decision, with the phrase that decided it. Classified per §1.4,
from the rationale text alone.

| Decision | Basis | The phrase that decided it |
|---|---|---|
| `ddd/adapter-policy-tables` | claim | "the tables are themselves claims, falsifiable against where boundary defects occur" |
| `ddd/amend-explicit-evidence-frozen` | claim | "because the correspondence dataset is only evidence while those fields stay machine-authored" |
| `bicep/linter-at-error` | claim | "Adopted with its limit stated, because DDD-bicep-sec-02 measured it" |
| `bicep/psrule-in-pr-stage` | claim | "DDD-bicep-net-01 and DDD-bicep-diag-01 measured both halves" |
| `ddd/correspondence-rows-are-stratified` | claim | "DDD-arch-06 says that question is unanswerable here, and a reader who does not know the rows are stratified will answer it anyway and be wrong" |
| `cs/async-policy-rules-at-error` | claim | "A repo that reads a green CA2016 as 'cancellation is handled' has misread it, and DDD-cs-pol-04 is the entry that says so" |
| `cs/authorization-fallback-deny` | claim | "DDD-cs-authz-01 measured the default: an endpoint with no attribute answers 200 OK" |
| `cs/disposal-rules-at-error` | claim | "Adopted with an unusual caveat, because DDD-cs-disp-05 measured it" |
| `cs/general-catch-at-error` | claim | "DDD-cs-fail-01 establishes that CA1031 reports the general-catch shape from the SDK analyzers alone" |
| `cs/policy-rules-at-error` | claim | "DDD-cs-pol-01 establishes that CA1305, CA1307, CA1309 and CA1310 close the comparison and culture cases" |
| `cs/startup-assertions-required` | claim | "Both close nothing unless the assertion actually runs, which makes them the most likely entries in the catalog to be recorded as closed while being inert" |
| `cs/unsafe-reinterpretation-banned` | claim | "A residual with no rejection witness cannot be allocated to review, because review has nothing to find" |
| `ddd/curation-over-mining` | claim | "nothing is mined into the catalog, so every entry has a principal who can explain it" |
| `ddd/enforce-matching-tightens-to-symbol` | claim | "the file arm of match_declarations is too broad to sustain what enforce mode claims" |
| `ddd/enum-member-gap-priced` | claim | "DDD-adapter-03 establishes that the §9.1 table is wrong by omission rather than by a wrong row" |
| `ddd/fixtures-not-sdk` | claim | "two #[ignore]d integration tests regenerate the SARIF from real builds with the documented invocations for local verification" |
| **`ddd/git-is-the-amend-trail`** | **constraint** | "`.ddd/` is committed, so git already carries who amended what and when" |
| **`ddd/interceptor-not-extension`** | **indeterminate** | "the check is part of the arrangement the agent edits through, not the agent's residual discretion" — states the design and its properties; nothing in the text says what settled the choice between feasibility (a claim), fork-avoidance (a constraint) and architectural cleanliness (a preference) |
| **`ddd/internal-not-surface`** | **mandate** | "Settles PRD §14 question 2 with the stated default" |
| `ddd/lsp-as-seam` | claim | "This bounds the blast radius of prerelease language-server churn to version pins and keeps the core language-free" |
| **`ddd/m6-proceeds-no-flip`** | **mandate** | "The condition therefore could not be evaluated against filed evidence; it was resolved by the stated default" |
| `ddd/pins-at-m2` | claim | "Escaped-decision reporting is load-bearing for the tool's thesis — the warning swamp is the canonical escape" |
| **`ddd/predicates-carry-no-status`** | **mandate** | "The split is enforced by schema (deny_unknown_fields rejects a status key on a predicate entry, naming ontology rule 1), not by convention or review" |
| `ddd/rejection-facts-prefilled` | claim | "Pre-filling the judgment invites rubber-stamping; omitting the facts wastes the agent's calls re-deriving them" |
| `ddd/report-coverage-explicit` | claim | "Every section of `ddd report escapes` separates checked-and-clean from not-checkable and names the uncheckable set" |
| **`ddd/rust-class-enforced-here`** | **experiment** | "Running the tail of this session governed is the first live reading of the friction falsifier on a real repo" |
| `ddd/rust-host-is-real` | claim | "a rust-analyzer change that moves symbol kinds, impl-block naming, or the readiness protocol fails the suite instead of passing a mock that was written to agree with the adapter" |
| `ddd/sarif-unification` | claim | "both toolchains emit SARIF through supported switches, verified at M2 and recorded in the README" |
| `ddd/seed-lands-in-claims-not-shared` | claim | "Intent is cheap to declare and, once declared, indistinguishable in the store from a transferability that was actually observed" |
| `ddd/v1-scope` | claim | "Scope follows where Context& demand concentrates and where adapter cost is lowest" |
| **`ddd/what-boundaries-priced-not-paid`** | **risk-acceptance** | "The finding stays standing and visible rather than being suppressed or satisfied cheaply" |
| `ddd/what-policy-table` | claim | "The table names what forms a boundary … and everything else … is internal elaboration that never demands a declaration" |
| `ddd/what-published-qualifier` | claim | "The unqualified rows called every event and command boundary-forming, which the measurement killed (DDD-what-02): 39 elements selected, 0 of them actually crossing anything" |
| `ddd/why-resolves-three-ways` | claim | "an ungoverned detected rule is a real escape — but reads as a governance failure rather than a lookup failure" |
| `ddd/workspace-member-delivery` | claim | "Basis rechecked against the current repo state at M1 and confirmed by the M1 implementation itself" |
| **`rust/no-unwrap`** | **constraint** | "Every unwrap is an undeclared panic path" |
| `web/htmlcss-enforce` | claim | "the observed failure mode is agents drifting past ungoverned HTML↔CSS seams, and `warn` is exactly the exhortation that failure mode ignores" |
| `web/markup-validity` | claim | "always a copy-paste artifact, and one that silently masks a missing *second* class the author meant to add, which is how orphan references are born" |
| `web/plain-pair-scope` | claim | "Context&'s visual deliverables are single-file HTML+CSS, so the plain pair is where the demand is" |
| `web/render-status-palette` | claim | "the palette is one choice — status colours mean verdicts together or not at all" |
| `web/token-discipline` | claim | "a raw restatement is not a lesser form of the token — it is the drift the token exists to prevent" |

### 4.1 Classification totals

| Basis actually argued from | Decisions | Share of 41 |
|---|---|---|
| claim | 33 | 80.5% |
| mandate | 3 | 7.3% |
| constraint | 2 | 4.9% |
| experiment | 1 | 2.4% |
| risk-acceptance | 1 | 2.4% |
| preference | **0** | 0% |
| indeterminate | 1 | 2.4% |

`preference` came out empty. Worth saying plainly, because it is the type the
typed-basis vocabulary's critics would expect to dominate a backfill: not one
pre-format-5 rationale argues from taste alone. The indeterminate count is
1/41 = 2.4%.


---

## 5. The reading

Applying §1.6 to the table:

> **5 of the 41 pre-format-5 decisions are manufactured-basis decisions** —
> every claim they cite is both weak by §1.5 and cited by a decision whose
> real basis, read from its own rationale, was not a claim.
>
> **5 / 41 = 12.2%.**

The five:

| Decision | Real basis | Sole cited claim | Why the claim is weak |
|---|---|---|---|
| `ddd/git-is-the-amend-trail` | constraint | `DDD-method-04` | co-filed · single-purpose · projected, no evidence |
| `ddd/internal-not-surface` | mandate | `DDD-adapter-02` | co-filed · single-purpose · projected, no evidence |
| `ddd/what-boundaries-priced-not-paid` | risk-acceptance | `DDD-what-04` | co-filed · single-purpose · projected, no evidence |
| `ddd/predicates-carry-no-status` | mandate | `DDD-method-01` | co-filed · single-purpose · falsifier self-sealing |
| `rust/no-unwrap` | constraint | `DDD-gates-01` | co-filed · single-purpose |

Per §1.7 that lands in the **≥ 4** band: **the prediction held.**

### 5.1 How strongly — the honest decomposition

The five are not equally strong, and the count should not be read as five
instances of one thing.

**Three are unambiguous.** `DDD-method-04`, `DDD-adapter-02` and `DDD-what-04`
each fire W1 + W2 + W4: filed in the same commit as the one decision that has
ever cited them, `projected`, no evidence attached. Each restates its citing
decision's rationale back at it — `DDD-method-04` restates "git carries
history", `DDD-adapter-02` restates the app-repo posture, `DDD-what-04`
restates "these graphs have no second party". That is the shape
`DDD-method-06` describes: a claim written so a decision could cite one.

**One is medium.** `DDD-method-01` carries a `reported` status and real
evidence (the framework corpus's predicate/claim split), so it is not
invented — but it is single-purpose, co-filed, and its falsifier is
self-sealing (§3.1). The decision it grounds argues from ontology rule 1, not
from observed drift.

**One is marginal, and may be a false positive of the measure.**
`DDD-gates-01` is `reported`, carries evidence that was actually read ("without
a single waiver request"), and its falsifier is concrete and firable. It is
weak only on W1 + W2 — co-filed and single-purpose — which is the composite's
weakest firing pattern. A reasonable ruling is that it would have been filed on
its own merits, in which case `rust/no-unwrap` drops out.

**This does not change the verdict.** The kill condition in §1.7 was **0**.
Even ruling both `DDD-gates-01` and `DDD-method-01` genuine, the floor is
**3 / 41**, which is band "held weakly" — still `reported`, never `retired`.
The claim survives under every reading of its own borderline calls.

### 5.2 The secondary reading, which is stronger than the primary

The mechanism `DDD-method-06` names has two steps: rule 3 forces a claim slot,
and the slot gets filled with something manufactured. The audit measures the
second step. **The first step is confirmed outright:**

> **7 of 41 (17.1%)** pre-format-5 decisions argue from a basis that is not a
> claim — 3 mandate, 2 constraint, 1 experiment, 1 risk-acceptance — and **all
> 7 cite a claim anyway**, because format 2 offered no other slot. An eighth
> (`ddd/interceptor-not-extension`) is indeterminate and also cites a claim.

That is the precondition of the mechanism, present in a fifth of the
population, with no counter-example: not one pre-format-5 decision left
`based_on` empty or found another way to say what it rested on. Whether the
claim in the slot was *manufactured* is the 5-of-7 question above; whether the
slot was *misfilled* is 7-of-7 among non-claim bases.

### 5.3 What the measure got wrong, reported not repaired

The weak-claim composite (§1.5) turned out to have **little discriminating
power in this store**: it flags **24 of 52 edges** and **22 of the 41 distinct
cited claims** as weak. Co-filing (W1, 31/52) and single-purpose citation (W2,
31/52) are simply the house style here — claims are filed in the same commit as
the work that motivated them, and most are cited once — so two-of-four fires on
claims nobody would call manufactured, including the whole evidence-backed
C#/Bicep seed.

The result is therefore carried almost entirely by the **basis classification**,
not by the weakness composite: of the 7 non-claim-basis decisions, the composite
excluded only 2 (`ddd/m6-proceeds-no-flip`, `ddd/rust-class-enforced-here`).

This is recorded, not fixed. Re-cutting W1/W2 after seeing that they fire
everywhere is exactly the adjustment §1 forbids, and it would move the number.
The correct response is to say the composite was poorly chosen for this store
and let the principal weigh the result knowing that.

### 5.4 The underpower test, run as pre-registered

§1.7 required **both** arms. First arm: fewer than 10 decisions classify as
non-`claim` — **met** (7, or 8 counting indeterminate). Second arm: more than
30% indeterminate — **not met** (1 decision, 2.4%). The conjunction fails, so
**underpowered is not claimed.**

That the first arm fired is worth the principal's attention anyway: the
discriminating subpopulation is 7 decisions, and a single reclassification moves
the count by ~2.4 points. The rationale texts are unusually explicit — they
routinely name their own limits — which is why the indeterminate rate came out
so low; a store with terser rationales would not classify this cleanly.

---

## 6. Borderline set — for the principal's ruling

Recorded per constraint 4: not consulted, not guessed away. Each carries its
best-reading classification in §4; these are the ones where the text
genuinely pulled two ways. **The three that would move the reading are marked
★.**

| Decision | Recorded as | Pulled toward | The phrase |
|---|---|---|---|
| ★ `rust/no-unwrap` | constraint | claim | "Every unwrap is an undeclared panic path" — true by definition of `unwrap` (a language fact), but `DDD-gates-01`'s cost proposition ("at negligible authoring cost in this codebase") is contestable and does not appear in the rationale at all |
| ★ `ddd/predicates-carry-no-status` | mandate | claim | "Closure lives only in claims, with status and falsifiers" — restates the ontology; the drift argument that `DDD-method-01` makes is nowhere in the rationale |
| ★ `ddd/interceptor-not-extension` | **indeterminate** | claim / preference / constraint | "Neither language server is extended or forked" — fork-avoidance (constraint), client-side feasibility (claim, = `DDD-arch-03`) and architectural cleanliness (preference) are all present and none is marked as decisive. Its claim `DDD-arch-03` is weak (W1+W2+W4), so ruling this `mandate`/`constraint`/`preference` would make it a sixth manufactured-basis decision; ruling it `claim` leaves the count at 5 |
| `ddd/curation-over-mining` | claim | mandate | "The graph is the source of truth and the code must conform to it" — stipulated posture; the `DDD-method-02` proposition survives only in the trailing "so every entry has a principal who can explain it" |
| `ddd/internal-not-surface` | mandate | preference | "Settles PRD §14 question 2 with the stated default" — adopting a stated default could be read as deference to the PRD (mandate) or as taste (preference). Either way it is not a claim, so the count is unaffected |
| `ddd/lsp-as-seam` | claim | preference | "keeps the core language-free (one adapter + policy table per new language, by design)" — architectural taste, but "bounds the blast radius … to version pins" is contestable and could be shown false |
| `ddd/enum-member-gap-priced` | claim | risk-acceptance | "Not paid in M5. … this milestone is curation." — the gap is claim-grounded (`DDD-adapter-03`, measured), the deferral is scope. Reclassifying would not add a manufactured decision: `DDD-adapter-03` is `reported` with evidence |
| `ddd/enforce-matching-tightens-to-symbol` | claim | risk-acceptance | "Priced, not paid, in M5." — same shape; `DDD-arch-05` is `reported` with an observation that already fired |
| `ddd/correspondence-rows-are-stratified` | claim | risk-acceptance | "Consequence accepted: the rows support within-stratum questions … and no question whose denominator is all edits" |
| `cs/startup-assertions-required` | claim | mandate | "the decision is not 'these are closable' — the claims already say that" — says outright that the claims are not the decision's content; the basis is still the inertness proposition in the same paragraph |
| `ddd/report-coverage-explicit` | claim | constraint | "counting unpinned edges as escapes would break CI for every format-1 repo and punish a supported format" — the not-a-gate half rests on format compatibility, the state-your-coverage half on `DDD-method-03` |
| `ddd/pins-at-m2` | claim | preference | "so the format bump lands with M2 rather than waiting for a later milestone" — the sequencing is a priority call; the necessity ("without the pin, `report escapes` cannot say whether the ground moved") is mechanism, not observation |
| `ddd/fixtures-not-sdk` | claim | preference | "so workspace CI stays hermetic and fast" — "fast" is preference; "hermetic" is a property `DDD-detect-02` measured |
| `ddd/amend-explicit-evidence-frozen` | claim | preference | "so create and amend cannot be confused in either direction and no silent upsert exists" — API taste; the decisive clause is the evidence-freezing one |
| `web/markup-validity` | claim | — | Not a type dispute: the proposition the rationale argues from ("always a copy-paste artifact … which is how orphan references are born") is **not what `DDD-web-01` states**. The basis is a claim; the cited claim is a different one. Filed as F-9 |
| `DDD-method-02` (falsifier grading) | present | unfalsifiable (taste) | "A mined catalog whose entries remain explainable to a named principal over time" — a judgement, but the store names the instrument (PRD §2 warning-swamp analysis) that produced its evidence and would settle it |

**Net effect if every ★ is ruled the other way:** `ddd/interceptor-not-extension`
becomes manufactured (+1) and `rust/no-unwrap` and `ddd/predicates-carry-no-status`
stay manufactured — count 6/41. If ruled toward `claim` instead: 3/41. **The
band is `reported` across the whole range.**

---

## 7. Status proposal for `DDD-method-06` — for the principal's ruling, and no other authority

**Proposed: `projected` → `reported`.**

The argument, from the reading and nothing else:

1. **The falsifier as written did not fire.** It required that the pre-format-5
   claim bases "audit as genuine — each `basedOn` claim would have been filed on
   its own merits, none exists only to satisfy rule 3." Three claims —
   `DDD-method-04`, `DDD-adapter-02`, `DDD-what-04` — are single-purpose,
   co-filed with their one consumer, `projected`, unevidenced, and restate
   their citing decision's own rationale. Each grounds a decision whose
   rationale argues from a constraint, a mandate, and a risk-acceptance
   respectively. "None exists only to satisfy rule 3" is false.

2. **The mechanism's precondition is confirmed at 7/41.** A fifth of the
   population argues from a non-claim basis and cites a claim regardless, with
   no counter-example. That is rule 3 doing exactly what the claim says it
   does, independent of how the weak-claim composite is graded.

3. **The strength is moderate, not strong.** 5/41 = 12.2%, just over the
   pre-registered ≥4 threshold, and only 3 of the 5 are unambiguous. The
   corruption is real and bounded — it did not swamp the store. `reported`
   ("exercised evidence") is the right status; `established` ("checker-engaged")
   is not claimed and nothing here supports it.

4. **The verdict is robust to every borderline call in §6.** Across the full
   range of rulings the count moves between 3 and 6. Killing the claim required
   0. No reading of this population retires it.

**Why not `retired`:** the kill condition did not come close. **Why not
underpowered:** the pre-registered conjunction failed (§5.4) — the population
discriminated, with a 2.4% indeterminate rate.

**What the principal should discount:** §5.3. The weak-claim composite was
badly chosen for this store and carries almost none of the result; the basis
classification carries all of it. A ruling that trusts the number should be a
ruling that trusts the §4 reading of 41 rationale texts, which is a judgement
call made against fixed definitions — not a mechanical measurement.

### 7.1 If the ruling is `reported`, the evidence text to attach

Drafted, **not applied** — no file in `.ddd/` was touched by this audit.

```yaml
status: reported
evidence: >
  Basis-quality audit of the 41 pre-format-5 decisions, 2026-08-11
  (docs/audits/basis-quality-2026-08.md), method pre-registered before the
  data was read. 7 of 41 decisions (17.1%) argue from a basis that is not a
  claim — 3 mandate, 2 constraint, 1 experiment, 1 risk-acceptance — and all
  7 cite a claim anyway, because format 2 offered no other slot. In 5 of
  those 7 the cited claim is itself weak: co-filed with its one consumer,
  single-purpose, and mostly projected with no evidence. Three are
  unambiguous — DDD-method-04, DDD-adapter-02, DDD-what-04 — each restating
  its citing decision's own rationale back at it. Two (DDD-method-01,
  DDD-gates-01) rest on the weakest firing pattern of the audit's composite
  and may be false positives; the count survives at 3/41 if both are ruled
  genuine. Recorded limit: the audit's weak-claim composite flagged 22 of 41
  cited claims, so it discriminated poorly and the result rests on the basis
  classification of the 41 rationale texts.
```

---

## 8. Findings — filed unactioned, for per-entry ruling *after* the status question

Nothing below was acted on. Per constraint 1 and the sequencing in the brief,
these are held until the status question in §7 is settled, so the audit's
result is not contaminated by cleanup.

### 8.1 Weak claims worth considering for retirement or repair

| # | Claim | Why | Note |
|---|---|---|---|
| F-1 | `DDD-arch-04` | All four weakness signals. Falsifier self-sealing: the study it names cannot be run because the decision it grounds abolished the data | Retire, or rewrite the falsifier against a store that still has both arms |
| F-2 | `DDD-method-05` | All four. Falsifier self-sealing in the same way | Same |
| F-3 | `DDD-method-01` | Falsifier self-sealing; single-purpose; co-filed. Carries genuine evidence, so repair may beat retirement | Rewrite the falsifier |
| F-4 | `DDD-method-03` | Falsifier ungraded-able ("changed no reader's action", no instrument); projected, unevidenced, both citations co-filed | Name an instrument or retire |
| F-5 | `DDD-method-04` | Restates a git property as a proposition; projected, unevidenced, single-purpose, co-filed | Candidate for retirement as `constraint` re-typing (see F-6) |
| F-6 | `DDD-adapter-02`, `DDD-what-04` | Projected, unevidenced, single-purpose, co-filed. Both restate their citing decision's rationale | Both have firable falsifiers if the world supplies the case; retirement is not obviously right |

### 8.2 Decisions whose basis type is visibly wrong

| # | Decision | Filed as | Reads as | Note |
|---|---|---|---|---|
| F-7a | `ddd/git-is-the-amend-trail` | claim | **constraint** | |
| F-7b | `rust/no-unwrap` | claim | **constraint** | see ★ in §6 |
| F-7c | `ddd/internal-not-surface` | claim | **mandate** | |
| F-7d | `ddd/predicates-carry-no-status` | claim | **mandate** | see ★ in §6 |
| F-7e | `ddd/m6-proceeds-no-flip` | claim | **mandate** | **Also a content mismatch:** it cites `DDD-adapter-01`, which is about policy-table localisation and says nothing about the M6/M7 flip condition. The rationale states outright that the condition "could not be evaluated against filed evidence". The cited claim is genuine but unrelated |
| F-7f | `ddd/rust-class-enforced-here` | claim | **experiment** | Honestly self-described in its own rationale; the claims it cites are the ones it exists to test |
| F-7g | `ddd/what-boundaries-priced-not-paid` | claim | **risk-acceptance** | |
| F-7h | `ddd/interceptor-not-extension` | claim | **indeterminate** | Needs the principal to say what settled it |

### 8.3 Other

| # | Finding |
|---|---|
| F-8 | **Self-sealing falsifiers are a pattern, not three coincidences.** In `DDD-arch-04`, `DDD-method-01` and `DDD-method-05`, the decision the claim grounds removes the arrangement under which the claim's falsifier could ever be observed. This is a distinct failure mode from manufactured claims and is not what `DDD-method-06` predicts. It may deserve a claim of its own — but filing one is a ruling, not an audit finding, and none was filed |
| F-9 | **`web/markup-validity` cites a claim that does not state its rationale.** The argument is that a duplicated class is always a copy-paste artifact that masks a missing second class; `DDD-web-01` says nothing about duplicate classes. The basis is a claim — an unfiled one |
| F-10 | **31 of 52 citation edges point at a single-purpose claim; 22 of 41 cited claims are weak by §1.5.** Most are the evidence-backed C#/Bicep catalog, where single-purpose is correct by design — a catalog entry records a measurement, not a decision prop. This is offered as a caution against reading §1.5's counts as a defect list, not as a cleanup queue |

---

## 9. One line for M8

**Yes, and it lands on both halves.** The id migration will carry 41
pre-format-5 decisions into `dec:` ids, and 7 of them (17%) have a real basis
that is not a claim — so a migration that copies `based_on: {claim: …}` edges
verbatim imports 5 manufactured claim-bases into the ledger graph as though
they were genuine, while re-typing them on the way is a principal's ruling and
not a migration step; and since retroactive acceptance means signing a version
hash, those 5 decisions would be signed with a stated basis this audit says is
wrong, which argues for settling §8.2 **before** the migration runs rather than
after.
