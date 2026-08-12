# Cross-repo provenance audit — declared, implicit, absent (2026-08)

**What is under test.** Not a claim. This audit measures a *property of the
corpus*: for each decision in scope, does it rest on something that is canon one
level up, and if so is that dependence **declared**, **implicit**, or **absent**?

The corpus is five levels, each basing on the one above:

```
product-cli → product-framework → ai-development-foundations
            → decision-driven-design → actor-indexed-determination
```

There is no upstreams manifest, no SHA pin, no lockfile. Every cross-repo basis
is convention. L4 will make *declared* edges mechanical; it cannot create edges
that were never declared. This audit counts what L4 would have to work with.

**The prediction, stated so the result can contradict it.** The tooling repos
will show heavy *implicit* upstream dependence — decisions reasoning from seams,
closure, escapes, allocation, tolerance without an edge — because author and
canon were the same person, which is exactly the condition under which
provenance goes unrecorded. **If implicit dependence turns out to be rare, the
corpus is better connected than assumed and the gap is only mechanical (L4 alone
fixes it).**

**Section 1 was written and committed before any decision rationale, statement,
or `notes` field was read, and before any classification was made.** Nothing in
it was adjusted after results were seen. If the criteria look badly chosen in
hindsight, that is a finding about the pre-registration, not a licence to re-cut
it.

*Disclosed limit on that claim, stated rather than glossed:* before writing §1 I
read file **shapes** — the `Decision` / `BasedOn` / `VersionRaw` structs in
`ddd-core` and `ledger-core`, directory listings, `format:`/`kind:`/`based_on:`
field counts — and, while inspecting the ledger file layout, saw the header
comment and the first version `statement` of
`.decisions/log/01KZNF77DG2RJPY7CA63HYTRJ1.yml` on screen. No classification was
made from it and no other decision text was read. That one exposure is recorded
here rather than claimed away.

---

## 1. Method, as pre-registered

*(Committed before the data was gathered. Fixed.)*

### 1.1 Population

Two stores in this repo, counted and reported **separately**, never pooled into
a single headline rate:

- **P1 — `.ddd/decisions/`**: every decision file, all formats. **45 files.**
  The unit is the decision. Format is recorded per row but does **not** gate
  inclusion: this audit is about cross-repo reach, and the format-5 typed-basis
  vocabulary is orthogonal to it.
- **P2 — `.decisions/` (the ledger)**: every decision's **latest version** (per
  spec v1.2, the unique tip of the `parent`/`merged_from` chain). **12
  decisions, 12 versions**, all in set `ledger-design`. The unit is the version,
  because the version is what carries `statement` and `based_on`.

**Total population: 57 decision units.**

Other corpus repos are **not present on disk** and are **not in this session's
repository scope**. Per the brief's constraint, nothing is cloned or fetched.
The audit is therefore scoped to this repo and the reading is **partial by
construction**; §2.3 states exactly what could not be seen.

### 1.2 The text that is graded

Grading is from the decision's own text and its basis edges — nothing else. No
claim file, no commit message, no spec section is read *in order to decide a
grade* (they are read to build the §1.3 inventory, which happens first and is
independent of any decision).

- **P1**: `title`, `rationale`, `notes`, and the `based_on` edges.
- **P2**: `statement`, `expectation`, `exposure`, the file-level `note`, the
  YAML comments attached to the version block, and the `based_on` refs.

### 1.3 Upstream construct inventory — sourcing rule fixed in advance

Built **before** any decision text is read. A term enters an upstream level's
inventory only if one of these holds:

- **(a)** the brief for this audit names it as belonging to that level; or
- **(b)** a document **present in this repo** explicitly attributes it to that
  upstream repo (a mirror notice, a conformance statement, a "canon wins"
  pointer).

Nothing is inferred from resemblance. Where a level's canon is unavailable the
inventory is marked **incomplete** and grading is **conservative**: a term not
admitted by (a) or (b) is **not** counted as an upstream reach, even where it
plausibly is one. Per constraint 5, an unverifiable upstream is not an absent
one — undercounting is the intended direction of error.

Per-level sourcing, fixed here:

| Level | Repo | Canon available here? | Inventory basis |
|---|---|---|---|
| **U1** | `product-framework` | **Yes, vendored** — `docs/product-framework-open.md`, declared in `CLAUDE.md` as a mirror of the canonical repo | The §-numbered construct vocabulary of that document. **Complete enough to grade.** |
| **U2** | `ai-development-foundations` | **No** — only a self-declared conformance statement (`docs/ai-foundations-conformance.md`) referencing the upstream repo | Terms that statement attributes upstream, plus (a). **Incomplete.** |
| **U3** | `decision-driven-design` | **No** — local `docs/ddd-*.md` are *this repo's tooling spec*, not upstream canon, though they restate method constructs and one names canon as ground truth | The brief's enumerated list (seam, escape, closure, predicate, store allocation, specification demand, floor, …) plus terms local docs explicitly attribute to canon. **Incomplete.** |
| **U4** | `actor-indexed-determination` | **No**, and no local mirror | The determination-tuple constructs named in the brief only. **Maximally incomplete.** |

*Naming note, recorded not corrected:* the brief writes
`ai-foundation-development` and "actor-general determination"; the repos that
exist under the account are `ai-development-foundations` and
`actor-indexed-determination`. Treated as the same levels.

### 1.4 Source attribution of a construct — three values, fixed in advance

The corpus's hard case: this repo **implements** the upstream method, so a
construct can be upstream canon *and* local vocabulary at once. Without this
split, every DDD decision reads as implicit upstream dependence trivially and
the prediction is confirmed by construction. So each construct invoked is
attributed:

- **upstream-only** — no local filed home. Its definition lives only upstream.
- **domesticated** — an upstream origin **and** a local filed home in this repo:
  a schema type, a store directory, or a normative section of a local spec.
- **local-only** — originates here; not an upstream reach at all.

### 1.5 The grade — fixed in advance

Per decision, per construct reached:

- **declared** — a machine-resolvable reference to an upstream entry: an id, a
  scheme-prefixed `based_on` ref, a URL, or a §-number **of an upstream
  document**. An edge or an id. Nothing else.
- **implicit** — the construct is named in prose (one quoted phrase recorded),
  with no id and no edge.
- **absent** — the reasoning depends on the construct and the text does not name
  it. Recorded **only** when the construct is the decision's *operative
  mechanism* — removing it makes the decision unintelligible — and neither its
  name nor a direct synonym appears. Otherwise the decision is `n/a`.
- **n/a** — no upstream construct invoked.

**Per constraint 4, prose mention is never sufficient for `declared`.** Declared
requires an id or an edge. That distinction is the whole point of the audit and
is not negotiable after the fact.

### 1.6 Kind of the upstream thing — fixed in advance

Per the brief, each reach records what is being reached for:

- **claim** — a contestable proposition about the world, citable as ground.
- **definition** — a term or its meaning. Arguably not a basis at all; counted
  separately for exactly that reason.
- **method-rule** — a rule the method imposes. Arguably a mandate rather than a
  ground.

### 1.7 Relation — grounding vs watched-not-grounding, recorded not resolved

The re-typing session surfaced a relation with no machine-readable home. It is
**counted separately and left unresolved**, per the brief:

- **grounding** — the decision would be different, or unsupported, were the
  upstream thing false or absent.
- **watched-not-grounding** — the decision tracks or conforms to the upstream
  thing without deriving from it ("we mirror the spec version", "we stay
  conformant", "canon wins if we diverge").

Recorded for every reach, not only declared ones — the brief asks what the edge
*would be* if filed.

### 1.8 Canon gaps — the count that may matter most

A construct is a **canon gap** iff it is invoked by at least one decision in
scope **and** has no citable upstream entry: no id, no §-number, no filed home
at any upstream level that this audit can see. Counted as constructs (not as
decisions), with the number of invoking decisions alongside. Because U2–U4 are
unavailable, every canon gap is stated as **"not citable from here"**, never as
"does not exist upstream".

### 1.9 The reading

Reported at two widths, **both pre-registered, neither chosen after the fact**:

- **Reading A (strict)** — over **upstream-only** reaches. The decisions that
  depend on something with no local home at all.
- **Reading B (broad)** — over **all** reaches including domesticated. The full
  set of provenance edges an upstreams manifest would have to pin.

For each: counts and rates per grade, per level reached, per upstream-thing
kind; plus the canon-gap count from §1.8.

### 1.10 Verdict bands — fixed in advance

On the **implicit rate** = implicit ÷ (declared + implicit + absent), computed
under **Reading B** (the broad width — the prediction is about the corpus as a
whole, and Reading A's denominator may be small):

| Implicit rate | Verdict on the prediction | What it means for L4 |
|---|---|---|
| **≥ 50%** | Held strongly. | Provenance is a **filing job** at scale; L4 alone is insufficient. |
| **20–49%** | Held. | Filing job, bounded. |
| **< 20%** | **Contradicted.** The corpus is better connected than assumed. | The gap is **mechanical**; L4 alone fixes it. |

The `< 20%` band is the kill condition and is stated first so it cannot be
softened later.

### 1.11 Borderline handling

Per constraint 3, a reach whose grade or attribution genuinely pulls two ways is
marked **borderline** and listed with its quoted phrase. It still receives its
best-reading value in the table; borderline is an annotation, not a fourth
grade. **Borderline counts are a result, not a failure.**

### 1.12 Underpower test — fixed in advance

**Underpowered** is claimed only if **both**:

- fewer than **10** decision units in the population invoke any upstream
  construct at all; **and**
- more than **30%** of the reaching decisions are marked borderline.

Population size alone is not grounds. 57 is not a small N for this question.

### 1.13 Constraints on this audit

Nothing in either store is changed. **No edges filed, no re-typing, no claims
filed, no id references inserted, no cleanup of anything noticed.** No repo is
cloned or fetched. The pre-existing `graph_bench.rs` lints stay untouched. Every
finding is a **proposal for the principal's ruling**, filed unactioned.

If, after seeing results, the criteria above look wrong, the audit **stops and
says so** rather than adjusting them.

---

## 2. Population, and what could not be seen

### 2.1 The two stores

| | Unit | Count |
|---|---|---|
| **P1** | `.ddd/decisions/*.yaml` — the decision | **45** (1 at format 1, 34 at format 2, 10 at format 5) |
| **P2** | `.decisions/` — the latest version of each decision | **12** (12 decisions, 12 versions, all in set `ledger-design`; no chain has a second tip) |
| | **Total** | **57** |

P1 carries **59** basis edges: 51 `claim`, 3 `mandate`, 2 `constraint`, 1 `preference`, 1 `experiment`, 1 `risk-acceptance`. P2 carries **28** basis refs across its 12 versions.

### 2.2 The upstream construct inventory, as built

Built before any decision text was read, per §1.3. Terms admitted:

- **U1 — `product-framework`** *(vendored canon, complete enough to grade)*. From `docs/product-framework-open.md`: What / How / Delivery; system (§3.2.5); context mapping (§3.1); journey and journey crossing (§3.0.1); Translation and the four patterns (§3.2.0); command, event, trigger, read model / View, entity, value object, UI step; Decider (§3.3); Projector (§3.4); named-algorithm primitive / Polanyi floor (§3.5); quality demand (§3.6); How contract and the Why cascade (§4.1); repository layout model (§4.3); work unit (§5); Build seam and verdict event (§5.1); codegen seam (§5.2); verification kinds (§6.3); feature ⊇ flow ⊇ slice (§7.1); done-as-predicate (§7.2); versions and direction (§7.3); the derivation contract and `derived_from` (§9); payload signature.
- **U2 — `ai-development-foundations`** *(canon absent; inventory incomplete)*. From `docs/ai-foundations-conformance.md`, which cites the upstream repo by URL: Pillar One / Pillar Two; Specification Framework; Execution Contract; SPMC and its four axes (Schema, Prompt, Model, Context); frozen context; the derivation contract's five rules; layer separation; scale discipline; the seam (its Section 7); autonomy level; closure contract; the eight Execution building blocks.
- **U3 — `decision-driven-design`** *(canon absent; inventory incomplete)*. From the brief plus the two local documents that name canon as ground truth (`docs/way-of-working-decision-allocated-delivery.md` §0–1, `docs/decision-ledger-prd.md` §2 "Compressed canon"): specification demand and its conservation; the four stores and store allocation (encoded constraint · mechanical verification/criterion · judgment · escaped); escape, priced vs silent; coverage-not-pass-rate; the governing decision set; the granularity bound; tolerance / assurance level / tiers T0–T2; the floor; ground and ground state; the closure principle; predicate; claim and falsifier; seam; discharge; enumeration; extra-actor; recon branch; blame.
- **U4 — `actor-indexed-determination`** *(absent, no local mirror; maximally incomplete)*. From the brief only: the determination-tuple constructs — **determination** itself; plus terms `docs/decision-ledger-prd.md` §9.3 uses without defining (recognition-over-recall, low-transfer-floor, pinnability, outcome-accountability).

*Application note, not a new criterion.* §1.5 grades a construct **reached**. Read as: the decision invokes the construct's **content** — its definition, or a rule about it — as reasoning material. Merely storing data in the tool's own schema (`based_on: {claim: …}`) is plumbing, not a reach. Recorded here because it is the single judgement that most moves the counts.

*Second application note.* §1.4's "local filed home" is read to exclude a **vendored mirror of upstream canon**: `docs/product-framework-open.md` is the U1 canon readable here, not a local spec. A U1 construct is therefore **domesticated** only where it also has a local schema type in `product-core` — which, this repo being product-framework's reference implementation, is nearly always.

### 2.3 What this audit could not see — stated, not glossed

**No upstream repository was cloned or fetched.** Per the brief's constraint and its anti-goals, availability was checked and nothing was pulled.

| Repo | On disk? | In session scope? | Exists on the account? |
|---|---|---|---|
| `Hafeok/product-framework` | no (vendored mirror only) | no | yes, public |
| `Hafeok/ai-development-foundations` | no | no | yes, public |
| `Hafeok/decision-driven-design` | no | no | yes, public |
| `Hafeok/actor-indexed-determination` | no | no | yes, public |

So all four are *cloneable in principle* and **none was cloned**. Consequences, carried into every number below:

1. **U2, U3 and U4 inventories are incomplete.** Grading was conservative per §1.3 — a term not admitted by (a) or (b) was not counted, so the reach counts are **floors, not estimates**.
2. **No canon gap is asserted as "does not exist upstream."** Every one is stated as *not citable from here*. Constraint 5, applied literally.
3. **U2 returning zero reaches (§4.2) may be an artifact of its inventory**, not a fact about the corpus. Flagged there rather than reported as a finding.
4. The brief's names `ai-foundation-development` and "actor-general determination" resolve to the repos `ai-development-foundations` and `actor-indexed-determination`. Recorded, not corrected.

A further limit, internal to this repo: `dec/ddd/prd-split` split the PRD into four documents, so the many `PRD §N` references in P1 rationales now point at a section numbering that no longer exists. That is a *local* citation-rot problem, not a cross-repo one, and is noted once here rather than counted.

---

## 3. The per-decision table

One row per **reach**. A decision with no reach appears in §3.3. `Attr` is §1.4; `Grade` is §1.5; `Kind` is §1.6; `Rel` is §1.7. ★ marks a row listed as borderline in §6.

### 3.1 P1 — `.ddd/decisions/` (35 of 45 reach; 38 reaches)

| Decision | Construct reached | Lvl | Attr | Grade | Kind | Rel | The phrase |
|---|---|---|---|---|---|---|---|
| `ddd/adapter-policy-tables` | claim / falsifiability | U3 | dom | implicit | definition | ground | "the tables are themselves claims, falsifiable against where boundary defects occur" |
| ★ `ddd/amend-explicit-evidence-frozen` | closure principle (own output is not ground) | U3 | dom | **absent** | method-rule | ground | "the correspondence dataset is only evidence while those fields stay machine-authored" |
| ★ `bicep/linter-at-error` | encoded-constraint store | U3 | dom | implicit | method-rule | ground | "The DAD §5.2 rule set at error is cheap and demonstrated" |
| `bicep/psrule-in-pr-stage` | predicate closure · judgment store | U3 | dom | implicit | method-rule | ground | "Neither predicate is closable in an arrangement that runs only the linter"; "staying with judgment" |
| `cs/async-policy-rules-at-error` | predicate case / closure | U3 | dom | implicit | definition | ground | "adopting this decision closes forwarding and leaves presence open" |
| `cs/authorization-fallback-deny` | predicate closure | U3 | dom | implicit | definition | ground | "This closes the predicate by composition rather than by a rule" |
| `cs/disposal-rules-at-error` | judgment store | U3 | dom | implicit | definition | ground | "Those stay with judgment." |
| `cs/general-catch-at-error` | escape · allocation to review | U3 | dom | implicit | method-rule | ground | "so the escape has to be argued for rather than defaulted into"; "That residual is real and stays allocated to review" |
| `cs/policy-rules-at-error` | predicate closure | U3 | dom | implicit | definition | ground | "close the comparison and culture cases of pred/code/explicit-policy-argument" |
| ★ `cs/policy-rules-at-error` | **determination** | **U4** | **upstream-only** | implicit | definition | ground | "the catalog is where determinations live" |
| `cs/startup-assertions-required` | closure (closable vs closed) | U3 | dom | implicit | definition | ground | "Both close nothing unless the assertion actually runs" |
| `cs/unsafe-reinterpretation-banned` | rejection witness · allocation to review | U3 | dom | implicit | method-rule | ground | "A residual with no rejection witness cannot be allocated to review, because review has nothing to find" |
| `ddd/curation-over-mining` | named accountable actor | U3 | dom | implicit | definition | ground | "every entry has a principal who can explain it" |
| `ddd/enforce-matching-tightens-to-symbol` | priced escape | U3 | dom | implicit | method-rule | ground | "Priced, not paid, in M5." |
| `ddd/enum-member-gap-priced` | silent vs priced escape | U3 | dom | implicit | method-rule | ground | "So the gap is silent twice: no seam demanded, and no trace in the dataset" |
| `ddd/interceptor-not-extension` | arrangement vs residual discretion | U3 | dom | implicit | method-rule | ground | "part of the arrangement the agent edits through, not the agent's residual discretion" |
| ★ `ddd/lsp-as-seam` | seam | U3 | dom | implicit | definition | ground | "The LSP protocol is the seam to language intelligence" |
| `ddd/m6-proceeds-no-flip` | escaped decision | U3 | dom | implicit | definition | ground | "An unfiled condition evaluated by default is an escaped decision in the making" |
| `ddd/m8-enforcement-closure` | closure · discharge | U3 | dom | implicit | method-rule | ground | "declarations that sign the change they discharge"; "durable discharge" |
| ★ `rust/no-unwrap` | encoded constraint vs judgment | U3 | dom | implicit | method-rule | ground | "Enforced twice by arrangement, not exhortation" |
| `ddd/pins-at-m2` | escape | U3 | dom | implicit | definition | ground | "the warning swamp is the canonical escape" |
| ★ `ddd/predicates-carry-no-status` | predicate/claim split rule | U3 | dom + **upstream doc named** | implicit | method-rule | ground | "The framework corpus's predicate format (predicate-format.md, predicate-definition.md) requires the split" |
| `ddd/rejection-facts-prefilled` | judgment store | U3 | dom | implicit | definition | ground | "Pre-filling the judgment invites rubber-stamping" |
| `ddd/report-coverage-explicit` | coverage-not-pass-rate | U3 | dom | implicit | method-rule | ground | "separates checked-and-clean from not-checkable and names the uncheckable set" |
| `ddd/rust-class-enforced-here` | falsifier | U3 | dom | implicit | definition | ground | "the first live reading of the friction falsifier on a real repo" |
| `ddd/rust-host-is-real` | coverage-not-pass-rate (vacuous gate) | U3 | dom | implicit | method-rule | ground | "a green exit code standing in for a check that never ran" |
| `ddd/seed-lands-in-claims-not-shared` | closure finding | U3 | dom | implicit | definition | ground | "it is a closure finding that does not hold there, carrying the authority of the catalog" |
| ★ `ddd/typed-basis` | claim as basis | U3 | dom | implicit | definition | ground | "The basedOn -> claim edge remains load-bearing where a claim is the basis" |
| `risk/ddd/undeclared-what-boundaries` | priced escape / exposure accepted | U3 | dom | implicit | method-rule | ground | "Exposure accepted"; "Review by 2027-02-07" |
| `risk/ddd/undeclared-what-boundaries` | What boundary kinds | **U1** | dom | implicit | definition | ground | "24 undeclared boundaries over 121 classified elements" |
| `web/htmlcss-enforce` | seam · arrangement vs exhortation | U3 | dom | implicit | method-rule | ground | "agents drifting past ungoverned HTML↔CSS seams, and `warn` is exactly the exhortation that failure mode ignores" |
| ★ `web/render-status-palette` | closure | U3 | dom | implicit | definition | ground | "The stylelint rule … is the closure" |
| `ddd/what-boundaries-priced-not-paid` | priced escape | U3 | dom | implicit | method-rule | ground | "The finding stays standing and visible rather than being suppressed or satisfied cheaply" |
| `ddd/what-boundaries-priced-not-paid` | boundary kinds (system, quality demand, context mapping, journey crossing, command, event, read model) | **U1** | dom | implicit | definition | ground | "2 system, 2 quality demand, 1 command, 1 event, 1 read model, 1 journey crossing" |
| `ddd/what-policy-table` | boundary kinds | **U1** | dom | implicit | definition | ground | "a policy table over **the framework's boundary kinds**"; "system, context mapping, journey crossing, event, command, payload signature, quality demand" |
| **`ddd/what-published-qualifier`** | **Translation (§3.2.0)** | **U1** | dom | **declared** | definition | ground | "the adapter computes a published/internal visibility from the **§3.2.0** Translations" |
| `ddd/why-resolves-three-ways` | escape | U3 | dom | implicit | definition | ground | "an ungoverned detected rule is a real escape" |
| ★ `ddd/workspace-member-delivery` | What/How vocabulary | **U1** | dom | implicit | definition | **watched** | "it does not extend the What/How vocabulary" |

### 3.2 P2 — `.decisions/` (6 of 12 reach; 6 reaches)

Every P2 version **declares** its bases — 28 refs across 12 versions — and **all 28 point inside this repo** (`prd:`, `format:`, `ruling:`, `constraint:`, `preference:`). Not one names an upstream repo. The `Declared-local` column records that; the `Grade` column is the *upstream* grade, per §1.5.

| Decision (tip) | Construct reached | Lvl | Attr | Grade | Kind | Rel | Declared-local | The phrase |
|---|---|---|---|---|---|---|---|---|
| ★ `…68QNJ0H9` → `…9HV1X61F` (acceptor identity) | acceptance cannot be delegated / accountable actor | U3 | dom | **absent** | method-rule | ground | `prd:…#9.3`, `#12.OD-3`, `#4.4`, `format:…#3.2` | "refusing model and CI identities … requiring an acceptance's actor to match the author of the commit" |
| `…VYR60TD3` (tolerance floor) | tolerance · floor | U3 | dom | implicit | definition | ground | `prd:…#12.OD-4`, `#4.2.1`, `format:…#3.3` | "A set's tolerance is a floor and a version may override it only upward" |
| `…7EJWN9EK` (L010) | judgment store | U3 | dom | implicit | definition | ground | `format:…#5.2`, `…migrations#spec-v1.1`, `prd:…#9.3` | "a judgment whose actor resolves to a model or CI identity fails the gate" |
| `…Z5Y366Y7` (disposition vocabulary) | priced escape | U3 | dom | implicit | definition | ground | `format:…#8`, `prd:…#8` | "escaped-priced, escape-review-due" |
| `…5ERWSCWJ` (merge two mechanisms) | judgment store | U3 | dom | implicit | definition | ground | `prd:…#7`, `#11-L3`, `ruling:emil-2026-08-11#…` | "refuses anything requiring judgment with the conflict preserved and named" |
| `…RJ7QYFDK` (certificate signing) | tolerance · floor · tier gating | U3 | dom | implicit | definition | ground | `prd:…#4.5`, `#11-L6`, `ruling:…`, `constraint:…`, `preference:…` | "signature required above the tolerance floor … because the floor is up-only, the signature requirement cannot be shopped out" |

### 3.3 Decisions that reach for nothing upstream (16 of 57)

**P1 (10):** `ddd/correspondence-rows-are-stratified` · `ddd/fixtures-not-sdk` · `ddd/git-is-the-amend-trail` · `ddd/internal-not-surface` · `ddd/prd-split` · `ddd/sarif-unification` · `ddd/v1-scope` · `web/markup-validity` · `web/plain-pair-scope` · `web/token-discipline`

**P2 (6):** `…68QNJ0H9` (workspace member) · `…KT2G4M0K` (write refusal) · `…S3ACXJDJ` (revise refusal) · `…E2RQ5W0Q` (graph stage) · `…PZDFYK5Q` (index determinism) · `…AXMFRCWV` (acceptance never transfers)

These are tool-mechanics decisions, and `n/a` is the right reading: each is intelligible without any upstream construct. `web/token-discipline` is the closest call — its severity argument runs on encoded-constraint logic — but "ban raw hex so colours go through tokens" stands on its own, so it is `n/a` per §1.5's tight rule for `absent`.

---

## 4. The reading

### 4.1 Reading B (broad) — the pre-registered verdict width

**41 of 57 decision units (71.9%) reach for at least one upstream construct.** Graded at the decision level, taking the best grade present:

| Grade | Units | Rate over the 41 reaching |
|---|---|---|
| **declared** | **1** | **2.4%** |
| **implicit** | **38** | **92.7%** |
| **absent** | **2** | **4.9%** |

> **Implicit rate = 38 / 41 = 92.7%.** Per §1.10 that is the **≥ 50%** band: **the prediction held strongly.** Cross-repo provenance in this corpus is a **filing job**, not a mechanical one. L4 alone does not fix it.

The single declared edge is `dec/ddd/what-published-qualifier` → product-framework **§3.2.0** — and even that is a section number in prose, not a basis edge.

### 4.2 By level reached (44 reaches; a decision may reach twice)

| Level | Reaches | declared | implicit | absent |
|---|---|---|---|---|
| **U1** `product-framework` | 5 | **1** | 4 | 0 |
| **U2** `ai-development-foundations` | **0** | 0 | 0 | 0 |
| **U3** `decision-driven-design` | **38** | 0 | 36 | 2 |
| **U4** `actor-indexed-determination` | 1 | 0 | 1 | 0 |

Two things to say plainly. **U3 carries 86% of the reaches and has not one declared edge** — the level the corpus depends on most is the level it cites least. And **U2 returning zero is not a finding**: its inventory is the thinnest of the four (§2.3), so zero may be the inventory's shape rather than the corpus's.

### 4.3 By kind of the upstream thing

| Kind | Reaches | Share |
|---|---|---|
| **definition** (a term) | 27 | 61.4% |
| **method-rule** (a rule the method imposes) | 17 | 38.6% |
| **claim** (citable as ground) | **0** | **0%** |

**Not one decision in 57 reaches upstream for a claim.** Every cross-repo reach is for a *definition* or a *method rule* — which is to say, for vocabulary and for mandates, never for ground.

This has a structural explanation the audit can point at, and it is not authorial neglect. In P1, a **claim** basis is `BasisPin { claim, status, changed }` — an in-repo claim id plus that claim's in-repo state. A **non-claim** basis is `TypedBasis { type, statement, ref }`, whose `ref` is a free-form pointer. So a typed non-claim basis *can* carry an upstream pointer today (unpinned, unresolved); **a claim basis structurally cannot** — the one kind of upstream thing that would be citable as ground is the one kind the format has no slot for. P2's `BasisRef` is an open string and can hold anything, but nothing resolves or pins it.

### 4.4 By relation (§1.7) — recorded, not resolved

| Relation | Reaches |
|---|---|
| grounding | 43 |
| watched-not-grounding | **1** |

The one is `dec/ddd/workspace-member-delivery` — "it does not extend the What/How vocabulary" — which tracks U1's vocabulary in order to stay clear of it, deriving nothing from it. Reported as asked, and with this observation: **at the cross-repo boundary the relation barely fires.** The watched-not-grounding relation the re-typing session surfaced is an *intra-repo* phenomenon — the secondary claim edges retained on `m6-proceeds-no-flip`, `internal-not-surface`, `rust/no-unwrap` and `what-boundaries-priced-not-paid` — and the ontology question it raises is not, on this evidence, blocking for L4.

### 4.5 Reading A (strict) — upstream-only reaches

Constructs with **no local filed home at all**:

| Decision | Construct | Level | Grade |
|---|---|---|---|
| `cs/policy-rules-at-error` | **determination** | U4 | implicit |
| `ddd/predicates-carry-no-status` | **`predicate-definition.md`** (named upstream document, not present here) | U3 | implicit |

**2 reaches, 2 decisions, 0 declared, 2 implicit.** The implicit rate is 100% and the N is 2 — far too small to carry a verdict, which is why §1.10 fixed Reading B as the verdict width in advance.

### 4.6 The tension between the two readings — flagged, not resolved

Reading B says 92.7%; Reading A says the strictly-upstream population is two decisions. Both are true, and the gap between them is the entire content of §1.4's `domesticated` category: 42 of the 44 reaches are for constructs that have *both* an upstream origin *and* a local home.

A critic can say Reading B measures the corpus reasoning in its own implemented vocabulary, which is not cross-repo dependence at all. A defender can say a local restatement is precisely how provenance goes unrecorded — the construct got copied instead of cited, which is the failure the audit was built to find.

**Per constraint 2, I say this rather than act on it:** knowing the numbers, I would now argue that a third width — domesticated reaches whose *local home is itself an uncited restatement of canon* — is the sharpest measure of the three. **I did not pre-register it and I am not substituting it.** The verdict above stands on Reading B as fixed in §1.10, and the principal should rule with both numbers in view.

### 4.7 Borderline rate

**10 of the 41 reaching decisions (24.4%) are borderline** (§6). Reported as a result, per §1.11.

### 4.8 The underpower test, run as pre-registered

§1.12 required **both** arms. First: fewer than 10 units reach — **not met** (41 reach). Second: more than 30% borderline — **not met** (24.4%). The conjunction fails, so **underpowered is not claimed**, and neither arm came close.

---

## 5. Canon gaps (§1.8) — constructs used, filed nowhere citable

Split by what the audit can actually assert, per constraint 5.

### 5.1 (i) Not citable from anywhere this audit can see — and not defined locally either

| Construct | Invoking decisions | Why it is a gap |
|---|---|---|
| **determination** | 1 — `cs/policy-rules-at-error` ("the catalog is where determinations live") | Used as a **defined term of art** with no definition anywhere in this repo. The only other occurrence is a bare use in `docs/ddd-cli-prd.md` line 17 ("determinations"), also undefined. Its home is the actor-general level, which this audit cannot see at all. |

**This is the one true canon gap the audit can assert** — a construct load-bearing enough to justify where an entry is filed, with no filed home on either side of the boundary that is reachable from here.

### 5.2 (ii) Not citable *from here* — a missing pin, not necessarily a missing entry

Each has a local restatement; none has a citable upstream entry reachable from this repo. Seven constructs, **38 invoking reaches**:

| # | Construct | Reaches | Local restatement (its only home here) |
|---|---|---|---|
| C-2 | **the predicate/claim split rule** | 1 | `docs/predicate-format.md`. The decision also names **`predicate-definition.md`**, which **does not exist in this repo** — direct evidence the upstream entry exists and is unpinned. |
| C-3 | **specification demand · conservation · the four stores** | ~12 (the whole C#/Bicep store-allocation family) | `decision-ledger-prd.md` §2.1; `way-of-working-decision-allocated-delivery.md` §0 |
| C-4 | **escape, priced vs silent** | 8 | `decision-ledger-prd.md` §2.1; `ddd report escapes` |
| C-5 | **the closure principle** | 6 | `decision-ledger-prd.md` §2.3 — one sentence |
| C-6 | **coverage-not-pass-rate** | 2 | `decision-ledger-prd.md` §2.4 |
| C-7 | **tolerance · floor · assurance level** | 3 | `decision-ledger-prd.md` §2.2; `ledger-format-v1.md` §3.3 |
| C-8 | **seam** | 2 explicit (11 mentions corpus-wide) | `.ddd/seams/`; no definitional section anywhere local |

### 5.3 The sharpest single number in this section

**The corpus's foundational principle is never named by any decision in it.** The strings *"specification demand"*, *"conservation"* and *"governing decision set"* appear **zero times** across all 57 decision texts — while roughly twelve decisions reason directly from the store allocation that principle produces ("stays with judgment", "allocated to review", "arrangement, not exhortation", "encoded constraint").

That is the audit's clearest picture of implicit dependence: the mechanism is everywhere and the ground is nowhere.

---

## 6. Borderline set — for the principal's ruling

Per constraint 3. Each carries its best-reading value in §3; these are the ten where the text pulled two ways. **★★ marks the two that would move a headline number.**

| # | Decision | Recorded as | Pulled toward | The phrase, and the tension |
|---|---|---|---|---|
| ★★ | `ddd/predicates-carry-no-status` | **implicit** | **declared** | "The framework corpus's predicate format (predicate-format.md, **predicate-definition.md**) requires the split" — it names the upstream corpus *and* the upstream document. It is the closest thing in the corpus to a declared cross-repo edge, and it fails §1.5 only on having no id, no URL, no §-number and no pin. Ruling it `declared` doubles the declared count from 1 to 2. |
| ★★ | `cs/policy-rules-at-error` | **U4 reach, implicit** | n/a | "the catalog is where determinations live" — a single word. If "determinations" is ordinary English here rather than the term of art, **Reading A drops from 2 to 1**, §4.2's U4 row goes to zero, and §5.1 — the audit's only asserted canon gap — empties. Everything in §5.1 rests on this one word. |
| | `ddd/amend-explicit-evidence-frozen` | **absent** | n/a | "the correspondence dataset is only evidence while those fields stay machine-authored" — the closure principle exactly (an actor's own output is not ground for that actor), or ordinary data hygiene about self-reported fields. Graded `absent` on the reading that the argument is unintelligible without it. |
| | `…9HV1X61F` (ledger, acceptor identity) | **absent** | n/a | "refusing model and CI identities" — the acceptance-cannot-be-delegated rule is the whole mechanism, and the version cites `prd:…#9.3` where that rule lives locally; but the version's own text never names it. The two `absent` grades in the whole audit are this row and the one above. |
| | `ddd/lsp-as-seam` | **U3, implicit** | U2 / local-only | "The LSP protocol is the seam to language intelligence" — *seam* is a DDD-canon construct, an AI-foundations construct (its Section 7), and a generic software term from Feathers. Level assignment is a judgement; the grade is `implicit` under all three. |
| | `bicep/linter-at-error` | **implicit** | n/a | "The DAD §5.2 rule set at error" — cites a local document by section, and that document declares canon its ground truth. The strongest form of implicit, and arguably not a reach at all. |
| | `rust/no-unwrap` | **implicit** | n/a | "Enforced twice by arrangement, not exhortation" — the encoded-constraint-vs-judgment distinction in canon's own idiom, or an ordinary phrase. |
| | `web/render-status-palette` | **implicit** | n/a | "The stylelint rule … is the closure" — the DDD closure construct, or plain English for "the closing rule". |
| | `ddd/typed-basis` | **implicit** | local-only | "The basedOn -> claim edge remains load-bearing where a claim is the basis" — the format-5 basis vocabulary (claim · constraint · mandate · preference · experiment · risk-acceptance) rhymes with canon's four stores but is not them, and was ruled here. Counted as a `claim`-construct reach only. |
| | `ddd/workspace-member-delivery` | **watched-not-grounding** | n/a | "it does not extend the What/How vocabulary" — a non-interference assertion. It is the audit's only watched-not-grounding row, so ruling it `n/a` empties §4.4's second line entirely. |

---

## 7. Findings — filed unactioned, in the three buckets the brief asked for

**Nothing below was acted on.** No edge filed, no re-typing, no claim filed, no id inserted, nothing cloned. Per constraint 1 these are proposals for the principal's ruling.

### 7.1 Bucket one — edges that should be filed as **grounding**

Grouped by upstream target, because the tail is one target serving many decisions.

| # | Decisions | Upstream target | Note |
|---|---|---|---|
| **G-1** | `ddd/what-published-qualifier` | `product-framework` **§3.2.0** (Translation) | The section number is already in the prose. This is a **prose-to-edge promotion plus a pin** — the cheapest edge in the corpus and the natural first test of the manifest. |
| **G-2** | `ddd/what-policy-table` | `product-framework` §3.2.5, §3.1, §3.0.1, §3.6 (system · context mapping · journey crossing · quality demand) | The table *is* a projection of the framework's boundary kinds; "the framework's boundary kinds" is prose today. |
| **G-3** | `ddd/what-boundaries-priced-not-paid`, `risk/ddd/undeclared-what-boundaries` | same U1 boundary kinds | Both enumerate the kinds by name; both cite none. |
| **G-4** | `ddd/predicates-carry-no-status` | `decision-driven-design` — `predicate-format.md`, `predicate-definition.md` | The mandate already names both documents. Filing the edge is pinning what the mandate says. **The highest-value single edge in the audit**, because it is the only place a decision states an upstream document as its requirement. |
| **G-5** | `cs/general-catch-at-error`, `cs/unsafe-reinterpretation-banned`, `cs/disposal-rules-at-error`, `cs/startup-assertions-required`, `bicep/psrule-in-pr-stage`, `bicep/linter-at-error`, `ddd/interceptor-not-extension`, `rust/no-unwrap`, `web/htmlcss-enforce` | `decision-driven-design` — the four stores / store allocation | **9 decisions, one target.** The largest single unrecorded dependence in the corpus. |
| **G-6** | `ddd/pins-at-m2`, `ddd/why-resolves-three-ways`, `ddd/m6-proceeds-no-flip`, `ddd/enum-member-gap-priced`, `ddd/enforce-matching-tightens-to-symbol`, `ddd/what-boundaries-priced-not-paid`, `risk/ddd/undeclared-what-boundaries`, ledger `…Z5Y366Y7` | `decision-driven-design` — escape, priced vs silent | 8 decisions, one target. |
| **G-7** | `ddd/report-coverage-explicit`, `ddd/rust-host-is-real` | `decision-driven-design` — coverage-not-pass-rate | |
| **G-8** | `ddd/seed-lands-in-claims-not-shared`, `ddd/m8-enforcement-closure`, `cs/authorization-fallback-deny`, `cs/async-policy-rules-at-error`, `cs/policy-rules-at-error`, `ddd/amend-explicit-evidence-frozen` | `decision-driven-design` — closure / predicate closure | Includes the audit's one P1 `absent` grade. |
| **G-9** | ledger `…VYR60TD3`, `…RJ7QYFDK` | `decision-driven-design` — tolerance · granularity bound · floor | |
| **G-10** | ledger `…7EJWN9EK`, `…5ERWSCWJ`, `…9HV1X61F` | `decision-driven-design` — the named accountable actor / acceptance cannot be delegated | Includes the audit's one P2 `absent` grade. |
| **G-11** | `ddd/adapter-policy-tables`, `ddd/rust-class-enforced-here`, `ddd/curation-over-mining`, `ddd/typed-basis` | `decision-driven-design` — claim · falsifier | Lowest confidence in the bucket; each could be ruled ontology-plumbing rather than a reach. |

**Blocking prerequisite, not a proposal:** none of G-1…G-11 is filable today for a **claim** target. §4.3 — a claim basis is `{claim, status, changed}` over an in-repo id. Filing an upstream *claim* edge needs a format change first. Non-claim types can carry an unpinned `ref` now.

### 7.2 Bucket two — edges that should be filed as **watched-not-grounding**

| # | Decision | Upstream target | Note |
|---|---|---|---|
| **W-1** | `ddd/workspace-member-delivery` | `product-framework` — the What/How vocabulary | "it does not extend the What/How vocabulary" — a *boundary-maintenance* dependence: the decision must keep tracking the vocabulary in order to keep not extending it. If U1 adds a node kind, this decision needs re-checking; nothing in it derives from U1. |
| **W-2** | `ddd/adapter-policy-tables` | `decision-driven-design` — claim / falsifiability | Candidate only. The tables are *typed as* claims — conformance to the ontology rather than derivation from it. Listed so the bucket is not artificially a single row; ruling it `grounding` (G-11) is equally defensible. |

**The bucket is one row, possibly two, out of 44 reaches.** On this evidence the missing relation type is not a cross-repo blocker: it is intra-repo, where the re-typing session already found it. Recorded, not resolved, per §1.7.

### 7.3 Bucket three — constructs used but not filed upstream (the canon gaps)

| # | Construct | Reaches | Assertion the audit can make |
|---|---|---|---|
| **C-1** | **determination** | 1 | **A true gap.** No definition in this repo, no citable entry anywhere reachable. Rests on one word — see the ★★ borderline. |
| **C-2** | the predicate/claim split rule (`predicate-definition.md`) | 1 | Upstream entry evidently exists — a decision names the file. **Missing pin, not missing entry.** |
| **C-3** | specification demand · conservation · the four stores | ~12 | Not citable from here. Never named by any decision (§5.3). |
| **C-4** | escape, priced vs silent | 8 | Not citable from here. |
| **C-5** | the closure principle | 6 | Not citable from here; the local restatement is one sentence. |
| **C-6** | coverage-not-pass-rate | 2 | Not citable from here. |
| **C-7** | tolerance · floor · assurance level | 3 | Not citable from here. |
| **C-8** | seam | 2 explicit | Not citable from here, and **no definitional home locally either** — `.ddd/seams/` stores instances, nothing defines the construct. Second-closest to a true gap after C-1. |

**Eight constructs.** Only C-1 is asserted as a genuine canon gap; C-2 is evidence of an unpinned entry; C-3…C-8 are stated as *not citable from here*, per constraint 5, and would resolve to pins the moment the upstream repos are readable.

---

## 8. Scoping line for L4

**Declared cross-repo edges that exist today for an upstreams manifest to pin: one — and it is not machine-readable.**

The full inventory:

- **P1** — 59 basis edges, **0** naming an upstream repo. One prose section reference (`ddd/what-published-qualifier` → §3.2.0) and one prose document reference (`ddd/predicates-carry-no-status` → `predicate-definition.md`).
- **P2** — 28 basis refs, **0** naming an upstream repo. All 28 resolve inside this repo.
- **`.decisions/upstreams.yaml`** — does not exist. **`upstreams.lock`** — does not exist.
- Neither store can pin an upstream **claim** at all (§4.3).

> **Federation delivers value only *after* an edge-filing pass, not before.** Building the manifest, the transitive resolver, the lockfile, the diamond-conflict finding and the basis-cone loader today would produce machinery with **one edge** to operate on — and that one edge would have to be hand-promoted from prose first. Every L4 mechanism the ledger PRD §9.4 specifies is correct and none of it has input.
>
> The ordering this implies: **file the edges first** (§7.1 is a work-list of roughly 40 reaches over ~10 upstream targets, most of them one-to-many), and let L4 pin what exists. The one caveat is the format prerequisite in §7.1 — a claim-typed upstream edge needs a slot before it can be filed — so a **minimal format change enabling a cross-repo basis ref is the true first step**, ahead of both the filing pass and L4 proper.

---

## 9. One line on the bottom of the stack

**Yes — one decision, and its construct's home is not a filed claim anywhere this audit can reach.**

`dec/cs/policy-rules-at-error` reaches below `decision-driven-design` for **determination** — "the entry is filed here because the catalog is where determinations live" — the actor-general level's own construct. It is used as a settled term of art to justify where an entry lives, and it is defined nowhere in this repo; the only other occurrence, `docs/ddd-cli-prd.md` line 17, uses it equally undefined. Whether its upstream home is a filed claim, a definition, or a paper's related-work **cannot be determined from here**, because `actor-indexed-determination` was not cloned and has no local mirror.

So the honest answer is two-part: **the stack's bottom is reached exactly once in 57 decisions, and the audit cannot tell you what it reached.** That is a smaller exposure than the corpus's shape would suggest — but it is also the one reach where "unverifiable, not absent" (constraint 5) does the most work, and where a single ruling on one word (§6) decides whether the finding exists at all.
