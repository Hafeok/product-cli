# C0 — the factoring boundary

**Status:** `[PROPOSED]` — session output, not ratified. Emil ratifies via merge.
**Dated:** 2026-08-26.
**Session:** `docs/c-track/c0-session-prompt-2026-08-26.md`, Gate 2.
**Basis available to this session:** the C-track PRD (uploaded, `[PROPOSED]`, not in any repository
reachable from here). `prd-…-r2.md`, `annex-a-corpus-acquisition-2026-08-26.md` and the sibling
ruling were **not** found in `Hafeok/product-cli` or `Hafeok/decision-driven-design`. Every citation
below to "r2 §7.1" or "Annex A A-6" is therefore a citation to the **PRD's restatement** of those
documents, not to the documents. Where the restatement is lossy, this boundary inherits the loss.

**Register discipline.** Every row is `[PROPOSED]`. `[OPEN]` marks what this session could not
settle. **Finding** marks something that contradicts the PRD and must not be smoothed over.

---

## 0. Method, and how it was actually followed

The prompt binds the method: factor **from the claim schema**, not by extraction from a shipped
track. The principal added a third prohibition in-session — do not factor by extraction from
`ledger-core` either — and fixed the order: schema first, then map, overlap reported as finding.

That order was followed. §1 derives the boundary from the schema alone. §2 maps the result onto
what already exists. §2 was written after §1 and did not revise it.

**One honest wobble, reported rather than smoothed.** The schema is not corpus-neutral throughout.
The embodiment field's own gloss in PRD §4.1 reads "generalised from `eli:legal_value`" — the PRD
hands the reader a corpus concept as the field's definition. Understanding what that field was *for*
therefore required reading a corpus term. That is the schema carrying a corpus fingerprint, not this
session reaching for a corpus, but it is the closest this factoring came to an FC-1 hit and it is
the substance of C-O-6. See §1.3.

---

## 1. The boundary, derived from the schema

### 1.0 The disposition vocabulary

The prompt offers two dispositions, **in-core** and **in-track**. Two is not enough to describe what
was found. Four are used:

| Disposition | Meaning |
|---|---|
| **in-core** | The core defines it, and the core's behaviour depends on it. |
| **in-core, opaque** | The core carries the field and refuses on its absence, but never reads its content. The corpus supplies the vocabulary; the core resolves nothing. |
| **in-track** | The core does not carry it. A track defines it. |
| **routed** | The core records a routing to a named actor or a corpus-supplied predicate, and does not itself decide. C-O-3's shape, applied more widely than C-O-3 anticipated. |

"in-core, opaque" is not a hedge. It is the pattern `ledger-core` already uses for `BasisRef` and
`RevisitRef` — a typed pointer, hashed, never dereferenced, over a vocabulary left open because
nothing resolves it yet. It is what lets a field be structurally required without the core knowing a
single corpus term. Most of the contested fields land here, and that is the finding of §1: **the
claim schema is mostly a schema about structure, and its corpus content is concentrated in
vocabularies the core can decline to read.**

### 1.1 The claim model (r2 §7.1 fields)

The proposition is a four-place relation: *decision `D`, under applicability `A`, represented by
implementation element `I`, evidenced by checks `E`.* Each field is disposed by asking what a
mechanism must do to carry it, and whether that duty can be discharged without knowing a corpus.

| Field | Disposition | Reason |
|---|---|---|
| Decision identity | **in-core, opaque** | The PRD already rules it opaque. A newtype with syntactic validation and no resolution. The core never parses the scheme, so no scheme can shape it. |
| Authoritative source | **in-core, opaque** | A pointer. Resolution is corpus work. |
| Source version | **in-core, opaque** — **ordering routed** | The core stores the version token and can compare two tokens for *identity*. It cannot compare them for *precedence*: whether `v3` supersedes `v2` is corpus knowledge (a consolidation, a re-issue, a withdrawal each order differently). Staleness therefore needs a corpus-supplied precedence predicate. **This is a real dependency and §1.5 depends on it.** |
| Embodiment | **in-core** | See §1.3. |
| Authority grade | **in-core, opaque** — **vocabulary in-track** | See §1.3. |
| Applicability | **in-core, structure only** | See §1.2. |
| Interpretation | **in-core** | Required, stated, free text to the core. Every transcription selects one reading among several; that a selection happened is structural, what was selected is not. |
| Residue | **in-core** | The cleanest element in the schema. "May be empty, may not be absent" is a pure schema property with zero corpus content, and it is the one field whose refusal can be proved by construction. See §4.2. |
| Implementation binding | **in-core, opaque** + optional content pin | A pointer plus, optionally, a content hash and base revision. The *pointer* is corpus-free; *resolving* it (does this symbol exist, did it change) needs a language host and is in-track. |
| Verification binding | **in-core, opaque** + three-state discharge | Same treatment. The three states are structural; what counts as a check is not. |
| Principal, per boundary | **in-core** | The per-boundary quantifier is the structural difference from every existing record in this workspace. See §1.4. |
| Supersession | **in-core** | Structural. See §1.5. |

### 1.2 Applicability — and a correction to the schema

r2 §7.1 gives applicability five slots: **actor, act, position, time, other**.

Tested against both synthetic corpora (§3) the five slots fit. They also fit a software
configuration decision. So they generalise.

**But `[OPEN]` C-O-7 (new): the five-slot vocabulary is itself unverified, and its shape is
norm-flavoured.** *Who, what, in what capacity, when* is the structure of a norm. `position` is the
weakest of the five and the hardest to state without reaching for status vocabulary. A five-slot
struct that happens to fit two corpora I designed is weak evidence — I chose the corpora after
reading the slots, and that is exactly the circularity the anti-shaping discipline exists to catch.

**`[PROPOSED]` factoring, which is strictly stronger than adopting the five slots.** Model
applicability as a **set of named dimensions the corpus declares**. The core requires only that the
set be non-empty, that each dimension be named, and that the same dimension set be used consistently
within a corpus. `actor / act / position / time / other` then becomes r2's **proposed default
dimension set**, testable rather than assumed — and if a corpus needs a sixth dimension, or has only
three, the core does not need changing and the r2 default is falsified rather than worked around.

This is the one place where the factoring **departs from the schema rather than implementing it**,
and it is flagged as such for ratification.

### 1.3 Embodiment and authority grade — C-O-6, tested

The PRD flags this as suspect and asks whether it is an ELI-ism smuggled into the core. It was
tested three ways rather than assumed.

The generalised statement is: *the parseable artefact is not always the authoritative one, so the
record must say which artefact was interpreted and what authority it carries.*

- **Against corpus A** (§3.1): the machine-readable class table is parseable; the committee minute it
  derives from is authoritative. They can disagree, and a transcription that reads the table has
  interpreted the derived artefact. **The distinction survives, and it does real work.**
- **Against corpus B** (§3.2): there is exactly one artefact and it is authoritative. **The field is
  present but degenerate.**
- **Against a software analogue** (not a corpus, a sanity check): a published interface description
  versus the behaviour it describes. **Survives.**

**`[PROPOSED]` disposition: the field is in-core; the grade *vocabulary* is in-track and opaque; the
*authoritativeness predicate* is routed.**

The core does exactly two things with it, both corpus-free:

1. **Refuses** a record whose embodiment differs from its source but states no grade. You may not
   silently interpret a derived artefact.
2. **Reports** "interpreted a non-authoritative embodiment" as its own audit row — but only where the
   corpus has supplied a predicate saying which grades are authoritative. Absent that predicate the
   core reports the routing, not a verdict. C-O-3's shape.

**The falsifier, so this is not a rubber stamp.** If both synthetic corpora end up with
embodiment == source, the field is dead weight and C-O-6 resolves *against* it. Corpus B is already
close to that. Corpus A is therefore designed with a genuine embodiment/source split (§3.1), which
makes it a real test — and if corpus A is ever simplified to remove that split, FC-1 loses its grip
on this field and C-O-6 reopens.

**Finding, recorded against C-O-6 rather than resolved by it.** The field generalises, but its
*provenance* is corpus-derived and the PRD says so in the field's own definition. That is one
corpus-shaped element reaching the schema before either corpus was formally examined. It is not an
FC-1 hit — the element survives generalisation on its merits — but it is evidence that "the schema
predates examination of either corpus" is not quite true of every field, and the anti-shaping
discipline should not treat the schema as clean ground.

### 1.4 The three boundaries and per-boundary ratification

Annex A A-6, as restated: **source→normalised**, **decision→specification**,
**specification→implementation**. Boundary 0 (ingest normalisation) is corpus-specific and out of
scope, per the PRD and the prompt.

**Disposition: in-core, all three, with the count itself held open.**

The structure is corpus-free: each boundary is separately ratifiable, each carries its own principal,
and a signature discharges exactly one. The refusal "one signature discharging two boundaries"
follows directly from that and needs no corpus.

**`[OPEN]` C-O-8 (new): is *three* structural, or is it this programme's transcription pipeline?**
Corpus B has no executable target, so its boundary 3 has no far side. That is not a defect — it is
the case that proves the core must not require all three to be dischargeable. But it does mean the
core should carry boundaries as an **ordered, corpus-declared chain** with three as the default,
rather than three hardcoded variants. Same argument as §1.2, same reason.

**Finding — this contradicts PRD §4.5.** §4.5 rules "decision with no implementation binding →
**undelivered**". Corpus B shows that is wrong as stated: it conflates *not delivered* with *nothing
to deliver*. A prose-only requirement with no executable target is not an undelivered transcription;
it has no boundary-3 far side at all. The core must distinguish three cases, and §4.5 gives it two:

| Case | §4.5 says | Should be |
|---|---|---|
| Implementation expected, absent | undelivered | **undelivered** |
| No implementation target exists | undelivered | **inapplicable** — and this is *not* an escape and *not* a presumption |
| Implementation expected, deliberately not built | undelivered | **escaped**, priced |

Collapsing case 2 into "undelivered" would make every prose corpus report permanent false gaps,
which is the failure mode that teaches people to ignore a gate. Collapsing it into "escaped" would
be worse: it would price an exposure that does not exist. **§4.5 needs a fourth state and C2's
acceptance condition changes with it.**

### 1.5 Supersession and staleness propagation

**Disposition: in-core, with the precedence predicate routed (§1.1).**

Supersession as a structure — a new record with an edge to the one it supersedes, never a rewrite,
with the latest derived from the parent DAG rather than from insertion order — is entirely
corpus-free. So is propagation: superseding at any of the three boundaries flags the boundaries below
it potentially stale.

What is **not** corpus-free is knowing that a source version *has been* superseded. That requires
ordering two corpus version tokens, which the core cannot do (§1.1). So staleness has two halves:

- **in-core:** given "source `S@v2` is superseded by `S@v3`", flag every claim pinned to `S@v2`.
- **routed:** the assertion that `v3` supersedes `v2`. Either the corpus files it as a fact, or a
  named actor does.

**This makes FC-3 sharper than the PRD states it.** FC-3 says "a claim pinned to a superseded source
is not flagged". Under this factoring the core can only fail FC-3 by failing to propagate a
supersession *it was told about*. A corpus that never tells it is a corpus-side failure, and the two
must be reported separately or the core will be blamed for the corpus's silence.

### 1.6 The remaining candidates

| Candidate | Disposition | Reason |
|---|---|---|
| **The event log and its projections** | **in-core** | The acts are the events; every projection derived. Structural, corpus-free, and PRD §4.7's own reasoning holds without modification. The *object* logged is the C-track's, not the ledger's — see §2. |
| **Write-time refusals** | **in-core** | All five of §4.2 are schema-level or identity-level. None needs a corpus term. This is the strongest in-core case in the list. |
| **Evidence binding and three-state discharge** | **in-core**, states structural, pointers opaque | *Delivered / presumed / escaped* is a three-way distinction about the epistemic standing of a check, not about what the check is. "Presumed is not evidence" (§4.4) is enforceable without knowing what was presumed. |
| **Coverage and gap queries** | **in-core**, amended per §1.4 | Structural, with the fourth state above. |
| **The mutation harness** | **in-core** | It is the falsifier apparatus for the whole track and it runs against the synthetic corpora, which are themselves in-core fixtures. Out of scope for this session (C5) but in-core when it comes. |
| **Ingest normalisation (Boundary 0)** | **in-track** | Ruled by Annex A A-6 and the prompt. Not revisited. |
| **Identifier schemes, grade vocabularies, dimension vocabularies** | **in-track** | The core declines to read them. That declining is what makes it a core. |
| **Resolution of any pointer** | **in-track** | Symbol resolution needs a language host; source resolution needs a corpus. The core stores and refuses; it does not dereference. |
| **Version precedence** | **routed** | §1.1, §1.5. |
| **Competence of a ratifying principal** | **routed** | C-O-3, unchanged. The core records the routing either way, as the PRD requires. |

---

## 2. The map onto what already exists — the finding

Written after §1, and §1 was not revised in light of it.

### 2.1 The principal's hypothesis, tested

The hypothesis put to this session: *different object, same substrate.* If it holds, the C-track is a
crate depending on `ledger-core`, duplicating none of it. If instead the claim is a ledger record
with extra fields, the C-track collapses into the L-track — and that is the better outcome.

**It holds, on the object. It holds more narrowly than stated, on the substrate.**

**Different object — confirmed, and by absence rather than by resemblance.** The claim is a
four-place relation `D × A × I × E`. A `ledger-core` decision-version is a lifecycle over one thing.
Searched directly: `residue`, `interpretation`, `embodiment`, `authority grade`, `applicability`, and
the three transcription boundaries **do not appear anywhere in `ledger-core`** — the only textual hits
are unrelated (`batch.rs` on listing completeness, `discharge.rs` on a contract seam, a day boundary
in date handling). Of the four places:

| Place | In `ledger-core` |
|---|---|
| `D` decision | **yes** — `DecisionId` |
| `A` applicability | **no** — `set` is a governance/tolerance scope, not an applicability |
| `I` implementation element | **no** — nothing binds a decision to an implementation element |
| `E` evidence | **partial, and deliberately inert** — `DischargeRef` names a *mechanism* (`analyzer:`, `test:`, `contract:`), and `discharge.rs` states plainly that "L0 parses and canonicalises these. Nothing *resolves* them." There is no evidence *state* and no record of a check having run. |

Nor is there per-boundary ratification. `Acceptance` is one identity signing one version hash, scoped
`Version` or `Class(DischargeRef)`. There is no boundary to sign *against*, so "one signature
discharging two boundaries" has no analogue and could not be expressed.

The claim is therefore not a ledger record with extra fields. It does not collapse into the L-track.

**Same substrate — confirmed, but the reusable part is narrower than "its event log, refusal harness,
canon and graph."** This matters, because "reuse the substrate" is cheap to say and was worth
pricing:

| Piece | Reuse | Price |
|---|---|---|
| `canon::{norm, put, put_set}` | **Direct, by design.** The module documents these as "the law's primitives" for "a payload type outside this crate … under its own domain-separation prefix — one canonicalisation law, explicit per-payload field sets, never a second scheme." | **Nil.** And this is not speculative: `ddd-core`'s `SeamBinding` already does exactly this under prefix `ddd.seam-binding.v1`. A second consumer exists. |
| `identity::Identity` | **Direct.** The L006 model/bot rejection, whole-token matching, normalisation-at-parse. | **Nil.** This is the §4.2 model-principal refusal, already built and already gated in CI. |
| `mint::UlidMint`, `id::validate_ulid` | **Direct.** Injectable time and entropy, so tests are deterministic. | Nil. |
| `hash::VersionHash`, `canon::canonical_json/canonical_bytes` | **Pattern only.** These are bound to `VersionRaw` specifically — including the load-bearing no-`..`-rest-pattern destructure that makes a new wire field a compile error. | Re-instantiate over the C-track's own wire type, under its own `CANONICAL_FORM`. The *guard* is the thing worth copying, and copying it is a dozen lines. |
| The event log (`ChangeSet`, parent DAG, append-only files) | **Pattern only.** `ChangeSet` carries `decisions`/`versions`/`acceptances`/`revocations` — the ledger's own act vocabulary. | Re-instantiate over the C-track's acts. Structure and discipline inherited; code not. |
| The verify harness (one pass, closed class set, verbs refuse writes introducing findings) | **Pattern only.** `Store` loads `.decisions/`; classes are ledger classes. | Re-instantiate. The *shape* — closed class set, no second validation copy, one sanctioned intermediate state — is the valuable part and it is a design rule, not a library. |
| The graph stage (deterministic Turtle, SPARQL shapes) | **Pattern only.** | Re-instantiate; it already rides `product_core::pf::sparql_rules`, which the C-track can too. |

**So: `[PROPOSED]` the C-track is a crate depending on `ledger-core` for `canon`, `identity`, `mint`
and `id`, defining its own canonical form under its own prefix, and re-instantiating the log, the
verify harness and the graph stage over its own object.** That is "same substrate" in the sense the
substrate is actually offered — primitives plus a proven extension pattern — not in the sense of
inheriting an engine.

The alternative, generalising `ledger-core` so both objects share one engine, is **not proposed**.
Its price is a change to a crate whose format document is normative for an outside implementation
(the Org Ledger imports `docs/ledger-format-v1.md`), and whose `CANONICAL_FORM` bump "invalidates
every acceptance and needs a migration note — it is never a quiet fix." Paying that to avoid
re-instantiating three patterns is the wrong trade, and it would couple the two objects at exactly
the point the sibling ruling wants them separable.

**The principal's caution, endorsed and sharpened.** Overlap between `ledger-core` and the r2 schema
is weak evidence: same author, same framework, same year. The finding above is deliberately built on
**absence** rather than resemblance — six named concepts that are not there, and a fourth relation
place that is not there — because absence is not vulnerable to the common-author objection in the way
agreement is.

### 2.2 What `ddd-core` contributes, and why the core must not depend on it

`ddd-core` has a `Claim` type. It is **a different sense of the word** — a closure claim carrying
`status` (projected/reported/established/retired), `evidence`, `falsifier`, `owner`, `version_index`.
It is the epistemic-status carrier of the DDD programme, not the decision-to-behaviour claim of r2
§7.1. Conflating them would be a canon defect. Hence the naming problem in §5.

`ddd-core` also has `SeamBinding`, which is genuinely a decision→implementation binding: subject
symbol, file, before/after content hashes, base revision — signing the *transition*, not the
resulting state, with the reasoning that signing post-change content alone "would let one declaration
cover a different edit that happens to land on identical content." **That shape is directly
applicable to the C-track's implementation binding, and it should be copied.**

**But the core must not depend on `ddd-core`, and this is not hypothetical.** `ddd-core` carries
`bicepconfig.rs`, `stylelintconfig.rs`, `htmlvalidateconfig.rs`, and a `.ddd/decisions/` store full of
`cs-*` C# rulings. It is the most corpus-shaped crate in the workspace, in exactly the C# direction
rule 7 prohibits. And the pull is real rather than theoretical: `ledger-cli` already reaches for
`ddd-core`. The prohibition has to be mechanical, which is §4.1.

---

## 3. The two synthetic corpora

They are the falsifier for the factoring, so a lazy choice disables FC-1. The prompt's own candidates
are argued with before being amended.

**On the prompt's first candidate — "numeric bands and delegation to a subordinate authority."**
Argued **against as worded**. "Delegation to a subordinate authority" is administrative-law shaped;
a corpus built to that description is statute with the words changed, and a corpus that is statute in
disguise cannot falsify statutory shaping. The *mechanisms* wanted — numeric bands, delegated
discretion, versioned source — are right. The framing must move out of governance-by-authority.

**On the prompt's second candidate — "prose-only obligations and no executable target."** Argued
**for**, essentially unchanged. Its value is precisely that boundary 3 has no far side, which is the
pressure that produced the §1.4 finding against PRD §4.5. That it broke the PRD within an hour of
being sketched is the argument for it.

**A caveat that must be stated rather than engineered away.** Both corpora are governance-shaped —
some body settles something, and something downstream must carry it. That is unavoidable: the
C-track is *about* transcribing decisions into behaviour, so a corpus with no decision-issuing
structure would exercise nothing. The requirement is unlike **statute** and unlike **C#**, not unlike
**governance**. The residual risk — that "unlike statute" is being satisfied by vocabulary rather
than by structure — is real, is not fully retired by these two, and is why §3.3 proposes a control.

### 3.1 Corpus A — **Bandmaster**: a competition equipment-class schedule

A hobbyist competition federation defines equipment classes by numeric thresholds — mass, span,
stored energy — and assigns entries to classes at events.

| Property | How it appears | What it exercises |
|---|---|---|
| Numeric bands | Class boundaries as numeric thresholds on three measured quantities | Applicability by `position`; interpretation where a measurement sits on a boundary |
| Delegated discretion | Event scrutineers assign class within a band and may re-measure | Applicability by `actor`; routed competence (C-O-3) with a real instance |
| **Embodiment ≠ source** | A machine-readable class table, published per season, **derived from** a committee minute that authorises it. The table is parseable; the minute governs. They can disagree. | **The C-O-6 test with teeth.** A transcription that reads the table has interpreted the derived artefact and must say so. |
| Versioned source | One table revision per season, plus mid-season corrections | Supersession; staleness propagation; the routed precedence predicate (§1.5) |
| Executable target | A scrutineering checker that decides a class from three measurements | All three boundaries have far sides; implementation and verification bindings both bind |
| **Genuine residue** | The minute says an airframe must be "substantially rigid". The table has no threshold for it, and none is proposed. | Residue that is **non-empty and stays non-empty** — the case that proves residue is a statement, not a defect to be closed |

Unlike statute: no penalties, no citations, no jurisdiction, no enforcement. Unlike C#: no types, no
language, no compiler. Bands and delegation arise from measurement and event logistics, not from
delegated authority.

### 3.2 Corpus B — **Steward**: a prose-only handover protocol

A shared workshop publishes, in prose only, how an instrument is prepared, logged and handed to the
next user. There is no executable target and none is intended.

| Property | How it appears | What it exercises |
|---|---|---|
| Prose-only | Numbered steps in continuous prose; no table, no schedule, no schema | Interpretation with no machine-readable anchor |
| **No executable target** | Nothing is built, nothing is checked automatically, and this is deliberate | **The §1.4 case.** Boundary 3 has no far side. Tests that the core distinguishes *inapplicable* from *undelivered* and from *escaped* |
| Embodiment == source | One artefact, and it is the authoritative one | **The C-O-6 counter-case.** If the grade field does nothing here, that is data about whether it earns its place |
| Contested reading | "Leave the bench as you found it" — two readings, both defensible, one selected | Interpretation as a recorded selection |
| Large residue | Most of the protocol is not reducible to anything checkable, and the residue field says so | Residue as the honest majority of a claim, not an afterthought |
| Role-based applicability | Steps differ by whether you are the outgoing or incoming user | Applicability by `actor` and `position` with no numeric content at all |

Unlike statute: it is a workshop courtesy protocol. Unlike C#: there is no code anywhere in it, which
is the point.

### 3.3 `[OPEN]` C-O-9 (new) — a third corpus as control

Two corpora both chosen by the same session that did the factoring is the same circularity C-O-2
warns about, one level up. **`[PROPOSED]`, not for this session:** a third corpus chosen by someone
other than whoever factored, or drawn from an existing public domain unrelated to this programme, as
a control on whether the two above were built to fit. Cost is real; the alternative is that FC-1
measures the factoring against corpora the factoring designed.

---

## 4. FC-1, stated so it runs on every commit

> **FC-1.** A core element cannot be exercised against either synthetic corpus without importing a
> statutory or C# concept. That element does not belong in the core.

Three checks, deliberately in decreasing order of strength, because the strong one is the slow one to
build and the weak ones are worth having on day one.

### 4.1 Dependency closure — the strongest, and the one the PRD wrongly assumed exists

**Finding, owned by the principal in-session.** PRD §2 mechanism 2 says the workspace graph enforces
"no dependent may be a dependency", "mechanically … not by discipline". **It does not.** `cargo xtask
check` registers three rules — CTX001 file length, CTX004 single responsibility, CTX005 function
length — and none is a dependency-direction check. The mechanism has to be built. It is **a C1
acceptance condition, not an assumption.**

`[PROPOSED]` check: read every workspace manifest, compute the C-crate's full transitive dependency
closure, and assert it contains no track crate and no corpus-shaped crate — `ddd-core`, `ddd-lsp`,
`ddd-mcp`, `ddd-cli` named explicitly, plus any future S-track member. Failing closed on an unknown
new member is preferable to failing open. Registered as a new `xtask` check so it runs under the
existing `cargo xtask check` CI step, with its `conventions/docs/CTX*.md` entry, which
`xtask check --self-test` already enforces.

### 4.2 Vocabulary tripwire — cheap, weak, worth having

`[PROPOSED]` a denylist scan over the C-crate's `src/` and `tests/` for statutory, ELI and C# tokens,
covering **comments and test names** as well as code, since standing rule 6 binds all three.

Its weakness must be stated where the check lives: **a denylist proves nothing about structure.** A
core thoroughly shaped by statute while scrupulously avoiding the word is exactly what FC-1 is for,
and this check would pass it. It catches careless imports, not shaping.

### 4.3 Exercise join — the real FC-1

`[PROPOSED]` and this is the one that actually implements the falsifier. Structurally the same as the
§14.4 authoring-scope completeness join already in `product-cli`, so there is precedent in-repo.

1. Enumerate every public core element.
2. Enumerate the elements each corpus-exercising test touches, where corpus fixtures may import only
   the core.
3. Join. Report per element: **exercised by both corpora**, **exercised by one**, **exercised by
   neither**.
4. **Fail on "neither."** An element no synthetic corpus can exercise is FC-1's target, named.
5. **Report "one" without failing.** Corpus B is deliberately degenerate in places — the authority
   grade is expected to be exercised by A and not B (§1.3). Failing on "one" would force corpora to
   be made artificially alike, destroying the falsifier to satisfy the check.

Step 5 is where a well-meaning tightening would quietly disable FC-1, so it is written down here as a
ruling-to-be rather than left to whoever implements it.

### 4.4 What none of the three catches

Stated so it is not mistaken for covered: **none of these detects a core element that is exercisable
by both corpora but whose *shape* was determined by a corpus.** §1.3's authority grade is exactly
that risk, and it was caught by argument, not by a check. FC-1 as mechanised is a floor. The
factoring gate is the ceiling, and it only runs when someone runs it.

---

## 5. Naming — candidates only, no mint

Minting a term is a canon act and belongs to the principal. `Claim` is unavailable: `ddd-core` owns it
in the epistemic sense (§2.2), and re-using it across two crates in one workspace would be the
`slice` overload again, which this repository already documents as a mistake it has to keep
explaining.

**For the record type:**

| Candidate | For | Against |
|---|---|---|
| **`Transcription`** | Already the PRD's own word — "the three transcription boundaries", "untested transcription claim" (§4.5). Names the act; the boundaries are already named after it; nothing in the workspace uses it. | Suggests faithful copying, which the PRD explicitly refuses to claim (§3, carried from r2 §5). Arguably a feature — it names the aspiration the register declines to certify — but it can mislead a reader who has not read §3. |
| **`Transposition`** | Means carrying a structure into another system while preserving relations, *without* claiming identity — mathematically and musically exact for what this records. Unused in the workspace. | **It is the standing term for implementing an EU directive.** Live legal usage, and rule 7 prohibits statutory vocabulary. Flagged rather than recommended for that reason alone. |
| **`Warrant`** | "What warrants that this element carries this decision" is precisely the proposition. Short, epistemically apt, unused in the workspace, and has a strong non-legal usage in epistemology. | Also has a legal usage. Weaker association than `Transposition` but not nil. |
| **`Carriage`** | Fully neutral, unused, and "the decision is carried into behaviour" reads correctly. | Weakest imagery of the four; slightly archaic; the noun is doing less work than the others. |

Ranked as offered: **`Transcription`** first, on the strength of already being the PRD's vocabulary;
**`Warrant`** second; **`Carriage`** third as the safe neutral; **`Transposition`** last despite being
the most precise, on rule 7.

**For the crate**, following the workspace's `<noun>-core` convention: whichever noun is minted,
plus `-core` — or `register-core`, which is what the PRD calls the thing and collides with nothing.
Placement and the full C-O-5 answer are Gate 3, and Gate 3 does not open until this boundary is
ratified.

---

## 6. New open items this session opened

| Ref | Item |
|---|---|
| C-O-7 | The five applicability slots are unverified and norm-flavoured. §1.2 proposes corpus-declared dimensions with r2's five as the default; ratification needed, since it departs from the schema. |
| C-O-8 | Whether *three* boundaries is structural or is this programme's pipeline. §1.4 proposes an ordered corpus-declared chain defaulting to three. |
| C-O-9 | A third corpus as a control on §3, chosen by someone other than whoever factored. |
| C-O-10 | PRD §4.5 needs a fourth state (**inapplicable**), distinct from undelivered, escaped and presumed. C2's acceptance condition changes with it. |
| C-O-11 | Version precedence is routed, so FC-3 splits into a core failure and a corpus failure. They must be reported separately. |

## 7. Existing open items this session touched

| Ref | Movement |
|---|---|
| C-O-1 | The boundary is now written (§1). Not ratified. |
| C-O-2 | Corpora named and sketched (§3); the prompt's first candidate argued against as worded and replaced. |
| C-O-3 | Unchanged in substance, but its *shape* — routed rather than checked — is now used for two further elements (§1.1, §1.3), so it is load-bearing beyond its own row. |
| C-O-5 | Not answered; Gate 3. Name candidates offered in §5 without minting. |
| C-O-6 | Tested three ways (§1.3), `[PROPOSED]` resolved **in favour with a narrowed disposition** and a stated falsifier — plus a finding that its provenance is corpus-derived and the PRD says so itself. |
