# The second axis — derivation confidence and domain relevance: session record

**Repository:** `Hafeok/product-cli`, branch `claude/two-axis-derivation-relevance`.
**Implements:** the top-ranked finding of `g0-session-report-2026-08-19.md` §7.1, with §7.4 arriving
at the same change from the review side.
**Corpus:** `corpus-backend`, read-only, at `3b0a56b3` — the **full solution** this time, not G0's
restorable three-project subset.
**Gates:** 1 design · 2 implement · 3 template and PRD · 4 close.

**Naming discipline** holds: corpus repositories are `corpus-backend` / `corpus-android`, domain
identifiers abstracted structurally. Extracted content lives in the generated instance, never here.

---

## 0. Environment — the condition that constrained G0 is cleared

G0's mapping-rule verdicts are qualified in §5.2 as *provisional pending a full-solution run*,
because the corpus restored only on the three projects that need no private feed. This session runs
with feed access. Confirmed before any design was settled:

| Check | Result |
|---|---|
| Corpus ref `3b0a56b3` resolvable | yes, after a fetch — the ref was not in the local clone |
| Full solution restores | **yes** — 16 of 16 projects, `dotnet restore` exit 0, no `NU1101`/auth failure |
| Private-feed packages resolved | yes — the private-prefix packages are in the local package cache |
| Corpus written to | **no** — the tree is a `git archive` export into a scratch directory; no worktree, no branch, no commit in the corpus repo |

Scale, measured, as the denominator for every count below:

| | source files (`.cs`, excluding `bin`/`obj`) |
|---|---|
| G0's subset (the three restorable projects) | 57 |
| Full solution `src/` | **951** |

≈16.7×, which is the "roughly fifteen times" §7.4 predicted. The G0 verdicts therefore stop being
provisional at Gate 2, and any row that *behaves* differently at full scale is a finding.

---

## 1. Gate 1 — the design

Proposed, not implemented. The template-change and backfill answers are Emil's rulings.

### 1.1 What the finding actually demands

§5.2 carries one `Assurance` column. G0 measured that it is really two properties wearing one name,
and that they are independent rather than correlated: **31 assertions at the top grade, every
mechanical check passing, all 31 wrong to ratify** (Group B — private serialisation state on
converter types), plus 18 more at mixed grades (Group A — generated schema-change code).

The forcing argument is recoverability. Group A is findable after the fact through `reg:evidence`
paths. **Group B is not findable at all** — nothing in a ratified assertion records that its property
came from a member the server reported as a *field*, so the count had to be taken from the
extractor's own JSON. A reviewer cannot locate the class of error they would need to re-examine.
That is ground that cannot be audited.

Read carefully, the demand splits in two, and the split governs the whole design:

- **The two marks** stop the table claiming a property it never computed.
- **The derivation record** is what makes a class of error *queryable*. It is the half that
  discharges the forcing argument.

Section 1.4 states plainly which half does the work at G0. It is not the half the finding's headline
suggests.

### 1.2 The two axes, and their value sets

**Axis 1 — derivation confidence.** How reliably the instrument read the code. Ordinal.

| Mark | Meaning |
|---|---|
| `high` | Declared. A probed LSP operation returned the thing itself; the language's semantics say what it is |
| `mid` | Structural or inferred. The server answered, and a mapping rule took a step the language does not itself take |
| `low` | Heuristic. A lexical rule over identifier text; no server semantics behind it |
| `not-derivable` | No instrument on this surface at any grade |

**The value set is unchanged from today's `Assurance` column, deliberately.** The existing column
was already measuring derivation and nothing else — that is precisely G0's finding — so re-grading
axis 1 is a *renaming of the column, not a re-marking of its rows*. Every one of the 561 ratified
assertions keeps its mark, and its bytes. §1.6 turns that into the backfill answer.

**Axis 2 — domain relevance.** Whether the result is the kind of thing the registry is for.

| Mark | Meaning |
|---|---|
| `high` | The assertion states something about the domain |
| `low` | Read correctly; describes a mechanism of the software, not of the domain |
| `unknown` | **Not computed.** The extractor has no rule that decides it from what it read |

Three values, not four: no middle mark is defined, because no rule produces one and an unused
middle is exactly the over-specification §7.8 warns about (*"the weakest thing that satisfies a
row's stated output is usually a sign the output was mis-specified"*).

**The two axes are not the same kind of scale, and the design does not pretend otherwise.**
Confidence is ordinal — `high` really is above `mid`. Relevance is a *verdict* with an undetermined
state; `unknown` is not between `high` and `low`, it is the absence of a verdict. Forcing it into an
ordinal would reintroduce the conflation this change exists to remove.

**Axis 2 carries its basis, because a mark without one is unreadable.** `reg:relevanceBasis`, one of:

| Basis | Meaning |
|---|---|
| `computed` | A named rule decided it from facts the extractor read |
| `defaulted` | The row's own construction fixes it, independent of any codebase |
| `not-computed` | The extractor has no rule for it on this surface |
| `not-applicable` | The row proposes nothing, so no assertion carries a mark |

### 1.3 The seven rows, re-graded

| Row | Confidence | Relevance | Basis | One-line justification |
|---|---|---|---|---|
| `classes` | `high` | `unknown` | `not-computed` | The language says it is a type. Nothing in the declaration subset says it is a *domain* type — Group A's migration classes and Group B's converters are both perfectly ordinary declared types |
| `subclass` | `high` | `unknown` | `not-computed` | `typeHierarchy` answers on production declarations. Whether an inheritance edge is a domain taxonomy or a framework one is not in the answer |
| `properties` | `high` | `unknown` | `not-computed` | **The measured case.** 31 assertions here, top confidence, every check passing, all wrong to ratify. This row is the finding |
| `foreign-keys` | `not-derivable` | `unknown` | `not-applicable` | No instrument; the row proposes nothing, so no assertion carries a mark. The pair is stated so the row reads as *no instrument*, not as *no relevance* |
| `modules` | `mid` | `unknown` | `not-computed` | A project name is a fact about code arrangement. Whether it names a bounded context is a domain reading nobody has taken |
| `usage-relations` | `mid` | `unknown` | `not-computed` | The rule reads reference positions. Relevance rides on the two endpoints, and both are themselves `unknown` |
| `synonyms` | `low` | `unknown` | `not-computed` | A spelling of an identifier. §7.8 already suspects the row's stated output is mis-specified; a relevance verdict on it would be a second guess on top of a first |

### 1.4 Relevance is `unknown` on every row, and that is the honest answer

This is the part to rule on, so it is stated without softening.

**At G0 the extractor computes domain relevance for no row.** Every relevance mark it writes is
`unknown`, with basis `not-computed`. The remedies §7.2 named — generated-code attributes, a
generated-file header, visibility — are all outside the declaration-level six, and widening that
subset is a separate filed decision that this session does not take.

Three consequences, all of them intended:

1. **§5.2 stops claiming relevance it never computed.** The current table's implication — declared
   rows are where extraction closes, review earns its keep on the inferred ones — is now measured
   false, and the two-axis table says so in a cell rather than in a footnote.
2. **The pair alone does not separate Group A from Group B.** With relevance uniformly `unknown`,
   the pair distinguishes them only through confidence, and Group A spans all three confidences.
   What separates them is the **derivation record** (§1.5) — which is what the forcing argument
   asked for in the first place. The batching output therefore keys on the pair *and then* on the
   derivation signature (§1.7).
3. **The axis is, today, a declared gap rather than a working discriminator.** Naming it that way is
   the point. An axis that defaulted to a plausible `high` would read as computed, which is
   `term:presumed-discharge` reappearing inside our own instrument for the second time in this
   track.

#### The one rule that would move it, and why it is not taken here

There is exactly one relevance rule available without a new LSP operation: **a path-convention rule**
— mark `low` where every evidence path of an assertion lies under a generated-code folder
convention. It uses only facts the extractor already holds (`Declaration::file`), so it does not
widen the LSP subset, and it would move Group A to `high`/`low`, which is the prompt's own worked
example.

**Recommendation: do not take it in this session.** It is a convention judgement — a claim that a
folder name means "generated" — and it belongs in the filed subset-widening decision with its
closure consequence stated, exactly as §7.2 requires of the other three remedies. Taking it here
would smuggle a filtering rule into a change whose stated scope is *making the groups visible*.

**Emil's ruling wanted:** accept uniform `unknown` (recommended), or rule the path-convention rule
in — in which case the `classes`/`modules`/`synonyms`/`usage-relations` rows gain `computed` relevance
for generated code and the pair separates the two groups on its own.

### 1.5 The derivation record — the half that discharges the forcing argument

What must be recorded so both rejected groups become queryable, in the template's vocabulary, riding
on the assertion node.

**Shape: flat, typed predicates on the assertion node.** Not a separate node, not a blank node:

- g-dec-04 fixes assertion identity as `sha256(subject, predicate, object)`, and a Reading that rides
  on the assertion node shares that identity. Riding on the node is therefore free — it cannot
  perturb identity, and §2.4's fixture proves it rather than assuming it.
- The writer emits **no blank node at all** by construction (G0 §8), and the projection drops any
  quad carrying one. A blank-node record would be silently lost.
- A separately minted derivation node would put a second typed subject in the file. The instance's
  CI file rule counts assertions and decisions, so it would pass — but the rule exists to keep a
  file's meaning one thing, and generalising a carve-out is what 0.2.0 spent its last revision
  undoing.

| Predicate | Object | Present when | Why it is here |
|---|---|---|---|
| `reg:mappingRow` | literal row id | always | already emitted at 0.2.0; now formally part of the record |
| `reg:derivedByRule` | literal rule name | always | **the rule**, not just the row — a row may carry several (`declared-member`, `member-range`, `usage-construct`, `identifier-word-split`) |
| `reg:standsOn` | `reg:op-<operation>`, repeatable | always | **the LSP operation(s)** the assertion's facts came from, as probed. An assertion that stands on an unprobed operation cannot exist, and the record says which |
| `reg:memberKind` | `reg:property` \| `reg:field` \| `reg:enum-member` | member-derived assertions | **Group B becomes one query.** This is the fact whose absence made Group B unfindable |
| `reg:containerKind` | `reg:class` \| `reg:interface` \| `reg:struct` \| `reg:enum` \| `reg:record` | assertions about a declared type or its member | the source-kind fact the reader actually saw |
| `reg:sourcePath` | literal, repo-relative | declaration-derived assertions | **Group A becomes one query.** `reg:evidence` already carries paths, but mixed with type names and rule names; this is the typed, queryable form |

**The design principle, stated so it is reviewable:** *record the facts, and let the reviewer's
query — not the extractor's rule — do the classification.* Group A is recoverable by a predicate over
`reg:sourcePath`; Group B by `reg:memberKind reg:field`. Neither requires the extractor to judge
anything, neither filters, and neither widens the subset.

**Template version: 0.2.0 → 0.3.0.** New vocabulary individuals, new optional predicates, new shapes.

**Backward compatible with the 561 ratified assertions: yes, and by construction rather than by
luck.** The new shape must not be a bare `sh:minCount 1` on the new predicates, or every ratified
file fails CI; but it must also not be plainly optional, or a future run could omit them silently —
the presumed-discharge shape again. So the constraint is **conditional on the generation marker**:

> An assertion carrying `reg:derivationConfidence` MUST also carry `reg:domainRelevance`,
> `reg:relevanceBasis`, `reg:derivedByRule`, and at least one `reg:standsOn`.

Pre-0.3.0 assertions carry no `reg:derivationConfidence`, so they are not targeted and stay
conformant, byte-unchanged. Post-0.3.0 assertions are fully constrained. **No file in the instance
changes, and none becomes non-conformant** — so no instance migration is needed and none is
proposed.

### 1.6 Backfill — what the 561 can and cannot gain

Supersede-never-rewrite applies; the re-confirmation problem is g-dec-04's and is not solved here.

| | Disposition |
|---|---|
| **Axis 1 (confidence)** | **Gained retroactively, without a byte changing.** The template declares `reg:assuranceGrade owl:equivalentProperty reg:derivationConfidence` in its vocabulary. The projection build runs OWL-RL, so every ratified assertion answers a confidence query. The value set is unchanged, so the answers are the marks they already carry |
| **Axis 2 (relevance)** | **Already correct, by absence.** Relevance is `unknown` for everything the extractor produces at G0, and a missing `reg:domainRelevance` is indistinguishable from `unknown`. The template states that reading for the pre-0.3.0 generation in one line. No rewrite, and no dishonesty |
| **Derivation record** | **Cannot be gained on the assertion node.** The facts were never stored. Recomputing them means re-running the extractor, and writing them into ratified files is a rewrite. Superseding is not available either: identity is the triple, so a "fresh" assertion carrying derivation is *the same assertion* — supersession has nothing to point at. This is g-dec-04's accepted cost arriving from a second direction |

**The honest disposition, and the narrowest option, are different things — both are stated.**

- *Forward-only* (recommended default): the pre-0.3.0 cohort carries no derivation, permanently. It
  is identifiable — the absence of `reg:derivationConfidence` names it exactly — but a reviewer
  auditing Group B **inside the ratified set still cannot find its members**. That limit survives
  this change and must be recorded in the PRD rather than papered over.
- *The narrowest option that would lift it*: a **run sidecar** — one file per extraction run,
  outside `graphs/`, recording the derivation record for every assertion the run produced, keyed by
  assertion id. It rewrites nothing, adds no assertion, and creates no new node in a graph file. The
  instance's file rule applies to `graphs/**/*.ttl` only, so a `runs/` file is mechanically
  available where g-dec-04's rejected option was not. Cost: a second place derivation lives, and a
  file whose relationship to ratification is "evidence about", not "ratified content".

**Emil's ruling wanted:** forward-only (recommended), or the run sidecar.

### 1.7 Review batching — the volume answer and the correctness answer

G0's 561 files were already at the edge of per-file review; the full solution is ~16.7× the corpus.
Wholesale merge is manufactured ground by §5.6, so the answer is batching.

**The reviewer sees a batch, not a file.** A batch is the set of assertions sharing a
**derivation signature**:

```
(derivation confidence, domain relevance, row, rule, member kind, container kind)
```

and carries: the pair, the row and rule, the operations the batch stands on, the count, a bounded
sample rendered as the reviewer reads a triple, and the full id list so the batch is actionable
rather than merely descriptive.

Ordering is total and deterministic — confidence descending, then relevance, then row, rule,
member kind, container kind — so two runs over the same facts emit the same batches in the same
order, and a batch listing diffs cleanly across refs.

**The batching output is on the plan's JSON, not behind `#[serde(skip)]`.** G0 §7.6 is explicit that
the extractor's account of its own limits is the reviewer's batching key and belongs in the output
contract. That mistake is not repeated.

### 1.8 What this change does not do

It does not filter Group A. It does not filter Group B. It does not widen the declaration subset.
It makes both groups **visible and batchable**, and it stops the table claiming a property nobody
computed. Filtering needs the widened subset, which is a separate filed decision.

### 1.9 Gates at this hold

`cargo t` · `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used` · `cargo xtask check` ·
`ddd validate` · the contract-surface gate. Results recorded at the hold.

---

## 2. Gate 2 — implemented

Emil's Gate 1 rulings, as taken: **uniform `unknown`** (relevance is declared-empty, not undeclared);
**the run sidecar** as the per-source evidence layer, with retroactive population as a Gate 2
deliverable; and the entailment/shape defect fixed by targeting the generation marker rather than
predicate presence.

### 2.1 What was built

| Piece | Where | What it does |
|---|---|---|
| The two axes | `product-core/src/ground/axes.rs` | `DerivationConfidence` (the old value set, renamed) · `DomainRelevance` (`high`/`low`/`unknown`, no `Ord`, because `unknown` is not a middle) · `RelevanceBasis` |
| The re-graded table | `ground/rows.rs` | seven rows, each carrying the pair with its basis |
| The derivation record | `ground/derivation.rs` | rule · probed operations · member kind · container kind · source paths |
| The evidence layer | `ground/sidecar.rs` | `runs/<corpus>-at-<ref>.ttl`, keyed by assertion id, with the orphan check |
| Review batching | `ground/batch.rs` | batches by `(confidence, relevance, row, rule, member kind, container kind)`, totally ordered |
| The projection join | `ground/projection.rs` | loads `graphs/` ∪ `runs/` ∪ `vocabulary/`, applies the equivalence entailment, validates or queries the result |
| Template 0.3.0 | `docs/g-track/registry-template/` | `vocabulary/reg.ttl` · `shapes/derivation.ttl` · CI joins the layers and checks orphans |

**`reg:mappingRow` is not repeated in the sidecar.** The canonical file already carries it, and
duplicating a fact across layers is how two layers drift.

### 2.2 The defect Emil found, and how it is now impossible to reintroduce

`reg:assuranceGrade owl:equivalentProperty reg:derivationConfidence` means the projection
*manufactures* `reg:derivationConfidence` on every pre-0.3.0 assertion. A shape keyed on that
predicate's presence would have passed in the authority repo — where validation runs pre-entailment
— and failed the moment anyone validated the projection.

The shapes target **`reg:GradedAssertion`**, a class the writer stamps and nothing entails. The
fixture `entailment_does_not_drag_the_pre_split_cohort_into_the_new_shape` asserts three things in
order, so it cannot pass for the wrong reason:

1. the equivalence *does* fire — one pre-0.3.0 assertion gains `reg:derivationConfidence`;
2. nothing gains `reg:GradedAssertion`;
3. `validate_projection` finds nothing, with a non-zero constraint count.

A second fixture validates the **projection**, not the repo, for the 0.3.0 case: an assertion whose
run evidence is absent fails loudly on the derivation half rather than passing quietly.

### 2.3 Fixtures

| Prompt's requirement | Fixture | Result |
|---|---|---|
| Both grades on every assertion | `propose::every_proposal_carries_both_marks`, `plan::no_assertion_carries_one_mark_without_the_other`, `rows::every_row_carries_both_axes` | pass |
| Unknown relevance representable, distinct from high | `propose::unknown_relevance_is_distinct_from_high`, `rows::no_row_claims_a_relevance_it_did_not_compute` | pass |
| Derivation queryable — both groups recoverable from the graph | `projection::the_field_derived_group_is_a_query_over_the_projection` (Group B) · `…the_generated_code_group_is_a_query_over_recorded_paths` (Group A) · `…the_two_groups_are_separable_from_one_another` | pass |
| G0's two groups re-derived as distinguishable batches | `batch::field_derived_assertions_batch_apart_from_property_derived_ones` — asserts the *kinds* separate, never a count | pass |
| Batching output stable | `batch::batching_is_deterministic_under_input_order`, `plan::the_plan_carries_a_stable_batching_of_every_assertion` | pass |
| **Idempotence preserved** | `mint::the_two_axes_and_the_derivation_stay_out_of_the_identity`, `plan::a_re_run_proposes_nothing_that_is_already_ratified` | pass |
| The join is checked | `sidecar::an_entry_with_no_assertion_behind_it_is_an_orphan`, `…orphans_are_counted_per_run_over_an_instance_tree` | pass |

**The identity row, verified rather than assumed.** Two assertions differing in confidence,
relevance, basis, rule, operations, member kind, container kind and source path — differing in
*everything the split added* — mint one id, because `g-dec-04` hashes the triple and nothing else.
The instance's history is unaffected, and no ratified assertion becomes a fresh proposal.

### 2.4 The full-solution re-run

`corpus-backend` at `3b0a56b3`, **all 16 projects restored**, 951 source files read through
`roslyn-language-server 5.11.0-1.26380.4` — the same server build G0 used. Written into a working
copy of the G0 instance, so the 561 ratified assertions are present and the run had to behave as a
re-run rather than a first run.

```
967 declarations · 174 hierarchy edges · 6 305 reference sites
9 662 assertions read · 561 already ratified · 9 101 proposed
18 review batches · 0 orphaned evidence entries
shapes: conformant — 42 constraints over the join (graphs ∪ runs ∪ vocabulary)
```

#### Per-row: does the full solution confirm what the subset showed?

Counts differ because the corpus is ≈16.7× larger; that is expected and is not the question. The
question is **behaviour**.

| Row | G0 (subset) | Full solution | Behaviour |
|---|---|---|---|
| `classes` | fired — 58 | fired — 942 | **confirmed** |
| `subclass` | fired — 17 | fired — 172 | **confirmed** |
| `properties` | fired — 215 | fired — 4 007 | **confirmed** |
| `foreign-keys` | not-derivable, absence reported | not-derivable, absence reported | **confirmed** — still no navigation property, still no FK attribute, at seven times the projects |
| `modules` | fired — 86 | fired — 1 514 | **confirmed**, with a table correction — see finding 2 |
| `usage-relations` | fired — 130 | fired — 2 103 | **confirmed** |
| `synonyms` | fired — 55 | fired — 924 | **confirmed** |

Six of seven rows fired; no row found nothing; **no row was gated off**. The §5.2 mapping verdicts
stand as written, and their *provisional pending a full-solution run* qualifier is discharged.

Two structural readings also confirmed at scale:

- **`subclass-transitive` still fires zero**, over 172 hierarchy edges rather than 17. Every declared
  derivation in this corpus is one level, type → marker interface. §7.3's "correct and idle" is not
  a small-sample artefact.
- **The restorable-subgraph limit disappears.** G0 counted supertypes named by an edge but declared
  outside the loaded subset; at full solution that count is **0**. Nothing is named that is not
  declared — which is what loading the whole solution was supposed to buy.

#### Three findings the subset could not have produced

**1. `workspace/symbol` availability is not stable across identical runs.** Two runs of the same
binary against the same corpus at the same ref, minutes apart:

| | `workspace/symbol` |
|---|---|
| Run 1 (first full-solution read) | advertised `true`, **did not answer within 15 s** — recorded unavailable, `divergence: over-advertised`; five of six operations available |
| Run 2 (the idempotence re-run) | advertised `true`, **answered — 7 symbols**; six of six available |

So the run-1 result is a **timeout under load, not a capability the server lacks**, and the honest
statement is sharper than "over-advertises at scale": **the probe's own verdict is run-dependent.**
An operation can be recorded unavailable on one run and available on the next, and a mapping row
gated on it would then yield different results on two runs that differ in nothing a reviewer can
see. G0's discipline — probe the operation, never the advertisement — holds and is vindicated; what
it gains here is that a probe verdict is a *reading*, with the same run-scoped honesty every other
reading in this track carries.

Not overstated: this is one operation, two runs, on one corpus. It is enough to say the verdict
varies; it is not enough to characterise the distribution. What it does settle is that *the subset
could not have shown this at all* — 57 files never loaded the server enough to time out.

**2. §5.2's `modules` row does not stand on `workspaceSymbol`, whatever the table says.** Run 1 is
the natural experiment: `workspace/symbol` was unavailable and the row still fired **1 514**
assertions, because the implementation reads module containment from `documentSymbol`. The
measurement holds regardless of *why* the operation failed. On the subset both answered, so the
divergence between table and implementation could not surface. **The table's LSP-operations cell for
this row is wrong and should read `documentSymbol`.** Corrected in this session's PRD edit.

The two findings compound: a row gated on an operation whose probe verdict varies would have
produced 1 514 assertions on one run and a gated-off row on the next. This row was not gated on it —
by accident of the implementation, not by the table's design.

**3. §7.5's collision prediction came true.** G0 measured **0** cross-module simple-name collisions
and noted that name-based range resolution was safe *on that corpus*, with `definition` held as the
exact-resolution route "if collisions appear at full-solution scale". At full solution there are
**21** — a mix of same-named DTO types across projects, per-project `Program` and `Version` types,
same-named extension-method holders, and paired handler/interface names.

So the `properties` row's range resolution **is now unsafe on this corpus**: a member typed with a
colliding simple name resolves against whichever declaration the name matches, and there are 21
names where that is a coin toss. `definition` remains wired and **unspent** — it is the only
operation no row consumes. This is a real defect, it is out of this session's scope, and it should
be filed: *range resolution matches on simple name; at full-solution scale the corpus contains 21
names where that conflates two types; the fix is the already-wired `definition` route.*

#### The fixpoint, against `g-dec-02`'s revisit condition

```
4 rounds · 3 rule parses · 12 executions · 509 ms
seed 35 032 triples · 2 103 proposed · 3 561 entailed
  usage-direct          2 103  over 1 productive round
  usage-transitive      3 561  over 3 productive rounds  [entailed, not proposed]
  subclass-transitive       0  over 0 productive rounds  [entailed, not proposed]
```

Parse-once holds (3 parses, 12 executions). **The ratio G0 said to watch has inverted**: 130 → 109
on the subset (0.84), 2 103 → 3 561 at full solution (1.69). The transitive closure now exceeds the
direct set, which is the quadratic behaviour `g-dec-02`'s `revisit_if` names. Wall-time is 509 ms and
nowhere near a projection-build budget, so **the condition is not met and the decision does not
flip** — but the direction is now measured rather than assumed, and the next scale step is where it
would be tested.

### 2.5 The two groups, recovered from the graph

The forcing argument, discharged against the real ratified cohort rather than a fixture. Both
queries run over recorded facts; neither asks the extractor to judge anything.

| | Query | Full solution | **Inside the 561 ratified** | G0 reported |
|---|---|---|---|---|
| **Group B** | `reg:memberKind = reg:field` | 195 | **31** | 31 |
| **Group A** | any `reg:sourcePath` under a generated-migration folder | 18 | **18** | 18 |

**Both counts reproduce G0's exactly**, and both were reached by query where G0 had to take them
from the extractor's own JSON — Group B by a hand count that was wrong twice before the notes were
made queryable. Group A's 18 fall across four rules (`declared-type` 4, `module-containment` 6,
`identifier-word-split` 4, `usage-construct` 4), which is the "spread across four rows" G0 described,
now enumerable.

**Every one of the 561 ratified assertions has derivation: 0 without.** Emil's ruling that the facts
are recomputable is confirmed on the real cohort — one deterministic re-run at the pinned ref, and
the pre-0.3.0 generation became auditable without a byte of ratified content changing.

One honest note on Group A's predicate. A first pass required *every* source path to sit under a
migrations folder and recovered 14 of the 18; the four it missed are `usage-construct` edges whose
second endpoint is a non-migration type. Relaxing the predicate to *any* path recovers all 18. That
is the design working as intended rather than a bug — **the extractor records the paths and the
reviewer writes the predicate** — but it is worth recording that the predicate is a real choice with
a real effect on what a batch contains.

### 2.6 Idempotence, end to end at full-solution scale

The row the prompt said to watch, run as a whole second extraction rather than only as a fixture.
The same binary, the same corpus at the same ref, against the instance the first run had just
populated:

```
0 assertion(s) proposed across 1 file(s)
9 662 assertion(s) already ratified in the canonical graph — not re-proposed
sidecar entries: every derivation record resolves to an assertion in the graph
nothing proposed — every one of the 9 662 triple(s) this run read is already ratified
```

**No tracked file in the instance changed** (`git status --porcelain` shows 0 modified). The single
planned file was the run sidecar, byte-identical to the one already on disk, so nothing was written.

So the new fields did **not** enter the assertion hash: g-dec-04's identity is unchanged, the
instance's history is intact, and the change is idempotent at 9 662 assertions rather than at the
handful a unit fixture exercises. The fixpoint's wall-time moved (509 ms → 307 ms) while its triple
counts did not, which is the right shape — time varies, the graph does not.

### 2.7 A finding about the instance, not about the extractor

G0 ruled 512 accepted and 49 rejected, and recorded that the merge acts were the ratifier's and
pending. **The instance holds all 561.** PR #2 (`Proposal: 561 assertions…`) merged 561 assertion
files and no later commit removes any, so the ruled rejection of Group A and Group B is not
reflected in the ratified graph.

Reported as an observation, not a conclusion about intent. It does make the recoverability argument
concrete rather than hypothetical: the 49 are ratified ground today, they were unfindable until this
change, and after it they are two queries.

### 2.8 Gates at this hold

| Gate | Result |
|---|---|
| `cargo t` | **1 491 passed, 0 failed** (baseline 1 454 — 37 new) |
| `cargo clippy --workspace -- -D warnings -D clippy::unwrap_used` | exit 0 |
| `cargo xtask check` | 0 errors |
| `ddd validate` | conformant — 277 entries, 0 warnings |
| Contract surface (`ddd diff-contracts main`) | **85 events, 0 undischarged** — 81 bindings filed, five new seams given their own `verdict_knowledge` |
| `product registry check` on the produced instance | conformant — 9 667 data files, 42 constraints over the join |
| File length ≤ 400 · function length · single-responsibility | no hard-limit violation |

---

## 3. Gate 3 — the template, the PRD

### 3.1 Template 0.3.0

| Change | File |
|---|---|
| The registry's own vocabulary — axis individuals, source-kind individuals, the six declaration-level operations, and the equivalence axiom | **new** `vocabulary/reg.ttl` |
| The two axes with the derivation record, constrained | **new** `shapes/derivation.ttl` |
| SHACL runs over `graphs/` ∪ `runs/` ∪ `vocabulary/`; the file rule stays scoped to `graphs/`; a new **sidecar-entries-resolve** step reports orphans per run | `.github/workflows/validate.yml` |
| The two axes, where derivation lives, why the split is compatible | `TEMPLATE.md` |
| The join named as a projection-build step, with its Rust form | `scripts/build-projection.sh` |

**Backward compatible with the ratified instance — no ruling needed, and none taken.** Verified, not
argued: `product registry check` over the working copy holding all 561 pre-0.3.0 assertions plus the
new run evidence reports conformant, 42 constraints, file rule clean. No file in the instance
changed.

The compatibility rests on one decision, and it is the one Emil caught at Gate 1. The shapes target
the **class** `reg:GradedAssertion`, stamped by the writer and entailed by nothing — not the presence
of `reg:derivationConfidence`, which the projection's equivalence entailment puts on every pre-0.3.0
assertion. A predicate-keyed shape would have been conformant in the authority repo and
non-conformant at the projection, and would have held only by accident of when validation runs.

### 3.2 PRD edits

- **§4.1** — the Reading tuple gains `derivation_confidence`, `domain_relevance`, `relevance_basis`.
- **§5.2** — the two-axis table replaces the single assurance column, with G0's evidence cited and
  the relevance column stated as **declared-empty** rather than undeclared, in those words.
- **§5.2** — the derivation record, the evidence layer, and the three consequences (identity
  untouched, the older cohort auditable, the join checked where it is read).
- **§5.2** — the provisional qualifier on the mapping verdicts **discharged**, with the full-solution
  confirmation stated.
- **§5.2** — the `modules` row's LSP cell corrected from `workspaceSymbol` to `documentSymbol`,
  measured.
- **§5.2** — the `properties` row records the open **simple-name collision** defect and names
  `definition` as the fix.
- **§5.2** — *Capability flags are claims* gains the second over-advertising case, with the
  server × workspace-size nuance.
- **§5.2** — the under-advertising bullet **corrected**: it cited Roslyn, which advertises
  `typeHierarchyProvider: true` under both handshakes and never under-advertised as a server. The
  case is `csharp-ls`, which does not advertise the provider at all. *(The correction owed from G0
  Gate 2.)*
- **§5.5.1** — review batched by the pair with the derivation as sub-key; the batching belongs in the
  output contract, not the logs.
- **§12** — new open item 15: **widening the declaration-level subset**, with the closure trade
  stated and the path-convention rule named as a candidate deliberately not taken here.

### 3.3 What this change does not do

It does not filter Group A. It does not filter Group B. It does not widen the declaration subset.
It makes both groups visible and batchable, and it stops the table claiming a property nobody
computed. Filing the widening is PRD §12 item 15.
