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
