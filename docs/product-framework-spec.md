# The Product Framework

**An open standard for specifying software as What, How, and Delivery.**

*A conformant instantiation of the Two Pillars Specification Framework. Version 1.0 — open specification.*

---

> **What this is.** An **open, catalog-agnostic standard** for describing a software product as three connected models — **What** it is, **How** it is built, and how it is **Delivered** — in a form that is reproducible, verifiable, and traceable. It defines the **shapes and rules**, not any particular product. Anyone can build catalogs, archetypes, and tooling against it.
>
> It is a conformant instantiation of the **Two Pillars Specification Framework** for software; every construct maps to a named Two Pillars concept ([§8](#8-conformance-to-the-two-pillars)).

*Published under a permissive license. Contributions welcome. This document describes the format; it ships no proprietary catalog content.*

---

## Contents

1. [Purpose and scope](#1-purpose-and-scope)
2. [The three models and the split](#2-the-three-models-and-the-split)
3. [The What — structure and behaviour](#3-the-what--structure-and-behaviour)
4. [The How — realising the What](#4-the-how--realising-the-what)
5. [Work units and the rationale trace](#5-work-units-and-the-rationale-trace)
6. [Verification — the conformance bar](#6-verification--the-conformance-bar)
7. [Delivery — bringing the What to a verifiable 'done'](#7-delivery--bringing-the-what-to-a-verifiable-done)
8. [Conformance to the Two Pillars](#8-conformance-to-the-two-pillars)
9. [Encoding and the derivation contract](#9-encoding-and-the-derivation-contract)
10. [Conformance rules (normative summary)](#10-conformance-rules-normative-summary)

---

## 1. Purpose and scope

Software is usually described in scattered, human-only artifacts — a wiki of requirements, diagrams that drift from the code, tickets that lose their rationale. This framework defines a single, connected, machine-readable way to describe a product so that the description can **drive generation, gate verification, and explain itself** — and so that delivery against it has a precise, queryable notion of "done."

It rests on one chain of dependencies: **reproducibility → measurement → improvement.** You cannot measure what you cannot reproduce; you cannot improve what you cannot measure. The framework's purpose is to make a product description reproducible and verifiable, so the rest follows.

**In scope**

- The structure of the three models — What, How, Delivery — and the typed links between them.
- The conformance rules every instance must satisfy.
- The mapping to the Two Pillars ([§8](#8-conformance-to-the-two-pillars)).

**Out of scope (deliberately)**

- Any specific product, archetype, or pattern library — those are built *on* the framework, not part of it.
- Quality-criteria content (the actual audits/checks) — the framework requires that conformant verifications exist and meet a strength bar ([§6](#6-verification--the-conformance-bar)); it does not supply them.
- Delivery cadence and team practice — the framework defines the delivery *model*, not how an organization runs it ([§7](#7-delivery--bringing-the-what-to-a-verifiable-done)).

> **The open/closed line.** This framework is the **empty form and its rules**. The forms you fill in — your domain models, your reusable How (archetypes), and especially your verification libraries — are yours. The framework is designed so that adopting it never requires disclosing them.

---

## 2. The three models and the split

A product is described by three models, in strict seniority. Each derives from or serves the ones above it.

```
WHAT      what the system is and does — owned by product & design
  |  domain model (structure)  +  event model (behaviour)
  |  ----- the split: business meaning  ->  technical realisation -----
HOW       how the code expresses the What — owned by engineering
  |  decisions / principles / patterns  +  contracts  +  interface standards
  v
DELIVERY  how the What is brought to a verifiable 'done' — model only
          features / releases as partitions of the What; done as a predicate
```

**The split is the central idea.** Above it lives **meaning**, expressed in the language of the business and owned by the people who own the product. Below it lives **realisation**, owned by engineering. The What is authored and agreed **before** the How is written. The line between them is where business meaning becomes technical reality, and keeping it explicit is what lets non-engineers own the What and engineers own the How without either guessing the other's intent.

**Everything is one graph.** All three models, and the links between them, form a single machine-readable graph (RDF is the reference encoding; [§9](#9-encoding-and-the-derivation-contract)). "Describe this system" is therefore a query, not a stale document, and impact analysis ("what depends on this?") is a graph traversal.

---

## 3. The What — structure and behaviour

The What is the specification every role reads and agrees on. It has two halves, expressed in one graph: the **domain model** (what exists) and the **event model** (what happens).

### 3.1 The domain model — structure

The shared definition of **what concepts mean** and how they relate. Conformance requirements:

- **Bounded contexts, not one flat model.** Within a context a term has exactly one meaning (the **ubiquitous language**); the same word may mean different things in different contexts. Cross-context correspondences are **explicit declared mappings**, never assumed. (This is what resolves "is a User a Customer?" rather than restating the confusion.)
- **Entities, relations, value objects, invariants.** Relations carry cardinality **and rationale**. Invariants are stated as machine-checkable constraints.
- **Machine-readable, with a constraint language.** The model must be expressible as a graph with validatable shapes (RDF + SHACL is the reference; an equivalent is conformant). A diagram alone is not a conformant domain model — it cannot generate or validate.

### 3.2 The event model — behaviour

The description of **what happens over time**, peer to the domain model, in the same graph. Conformance requirements:

- **Built from domain-typed primitives.** Events, commands, read models, and UI steps each reference domain concepts: an event **changes** an entity, a command **targets** an aggregate, a read model **projects** entities. Behaviour may not reference structure that does not exist.
- **Depth is proportional to behavioural complexity.** Concepts with rich or historical behaviour get full event models; simple create/read/update/delete concepts get a minimal one or none. The framework requires that the *interesting* behaviour be modelled, not that every triviality be ceremony.
- **Owned by product and design.** Its natural form — a timeline with interface steps — is readable and signable by non-engineers; it is the bridge between concepts and screens.

> **Why structure and behaviour are one graph.** An event is never free-floating: it always changes a domain entity. Modelling them in one graph means structure and behaviour cannot drift apart, and a single question — "what happens to this concept?" — returns both its shape and the flows that change it.

---

## 4. The How — realising the What

The How is how code expresses the What. Wherever a system shape recurs, the How should be **captured once and reused** (an "archetype" — a reusable How). The framework defines three sub-layers and their conformance rules; it does not prescribe any particular technology, layering, or pattern.

### 4.1 Decisions, principles, patterns — the Why, made traceable

The reasoning that shapes code is captured as explicit, linked artifacts, so every output can carry a **rationale trace**.

```
DECISIONS    foundational choices (project shape, layout, layering) + WHY
   | license
PRINCIPLES   the rules those decisions imply (stated checkably)
   | realized by
PATTERNS     the concrete shapes that implement the principles
   | applied by
WORK UNITS   reference the above by pointer; emit a rationale trace
```

- **Declared once, referenced by pointer.** Principles and patterns are declared at the How level and referenced by work units — never re-declared per unit (that is how they drift).
- **Decisions carry rationale.** Each foundational decision states what it decides, why, when it applies, and when it does not — making structure auditable, teachable, and generatable.
- **Earn-their-place rule.** A principle or pattern belongs in the model only if a work unit applies it or a verification enforces it. Otherwise it is documentation, not part of the framework instance.

### 4.2 Contracts — the realisation surface

The How fixes the realisation through contracts. The framework requires that each contract be stated **checkably** — precisely enough that a verification can confirm conformance — but does not prescribe their content:

- **An application contract** — the invariant code-shaping decisions (language, layering, organization, persistence model). Stable across instances of an archetype.
- **An infrastructure/runtime contract** — the concrete runtime choices for an instance. May vary per deployment; once chosen, frozen. It must **satisfy** the application contract, and the satisfaction is recorded.
- **The seam between them is verified.** Where application and runtime are described separately, a verification must confirm they agree (configuration, identity/permissions, resources expected vs. provided). This seam is a required verification, because nothing else makes the two halves agree.

### 4.3 Interface contracts — use the standards

For any interface or dependency that has an **industry-standard description format, the How uses that standard as the contract.** Bespoke description is permitted only where no standard exists. Reinventing a standard is non-conformant: it forfeits the standard's tooling and ecosystem.

| Surface | Reference standard |
|---|---|
| REST interface | OpenAPI |
| Async / event stream | AsyncAPI |
| RPC · message payloads | Protobuf / gRPC · JSON Schema / Avro |
| Cloud events | CloudEvents |
| Auth / identity | OIDC / OAuth2 metadata |

**Generated from the domain model.** Interface contracts are derived from the domain and event models, not hand-written, so the published surface cannot drift from the meaning. The standard document is the *surface*; the domain model remains the *meaning*; the derivation link is the traceability between them.

---

## 5. Work units and the rationale trace

A **work unit** is the smallest reproducible unit of realisation: a single bounded transformation that produces one artifact from a fixed, declared input. The framework's conformance requirements for a work unit:

- **Single-purpose and bounded.** One unit produces one artifact; its input (its context) is explicitly declared and frozen, so the same input yields the same output — the reproducibility guarantee.
- **References, never re-derives.** A unit reads the What concepts and the How principles/patterns it depends on by pointer; it does not re-decide them. Hard reasoning concentrates upstream, in the What and the How.
- **Emits a rationale trace.** Each unit's output carries a trace to the decisions that produced it — the domain concept (what), the flow (behaviour), the principle/pattern (why), the foundational decision (structure).

> **The trace must be true.** A rationale trace that claims a principle the artifact violates is worse than no trace — it misleads. Therefore: every principle a unit claims to apply must be **enforced by a passing verification**, or the claim is retracted from the trace. Verifications gate which trace claims survive. The same checks that ensure correctness also keep the explanation honest.

---

## 6. Verification — the conformance bar

The framework is built on verification, but it deliberately ships **no verifications**. It defines what verification must *do*, and how strong it must be; the actual checks are the adopter's (and, typically, their most valuable proprietary asset).

### 6.1 Requirements on verification

- **No accepted output without a verdict.** Every artifact kind has versioned acceptance criteria; an artifact is accepted only when its verifications pass.
- **The coherence bar.** When realisation is split across work units, a verification must confirm the parts agree **at least as well as a single unsplit author would achieve from shared context.** If a split makes coherence weaker than the unsplit baseline, the split is not worth it. This is the framework's central quality guarantee.
- **Verifications are deterministic gates.** Conformance is established **by construction, not by instruction** — a check that fails the build, not a request to be careful.
- **Every verification names what it protects.** A verification cites the principle, contract, or model element it enforces — which is what makes the rationale trace ([§5](#5-work-units-and-the-rationale-trace)) honest and impact analysis possible.

### 6.2 The required verification kinds

A conformant instance must have verifications covering, at minimum:

| Kind | Confirms |
|---|---|
| **Internal coherence** | the parts of one work unit's output agree with each other |
| **Contract conformance** | realised code obeys the How's contracts |
| **Seam** | separately-described parts (e.g. application vs. runtime) agree |
| **Domain conformance** | realised code matches the domain model; no structural drift |
| **Behavioural conformance** | realised behaviour matches the event model; flows are complete |

*The framework specifies these kinds; the adopter supplies the checks. The content of those checks is out of scope by design — see the open/closed line in [§1](#1-purpose-and-scope).*

---

## 7. Delivery — bringing the What to a verifiable 'done'

Because the What is a graph, delivery is **partitioning that graph into shippable slices**, and "done" is a **verifiable predicate** rather than a judgement.

### 7.1 Units of delivery

- **A feature** is the smallest independently valuable and verifiable slice — typically one behavioural flow over its concepts.
- **A release** is a chosen, coherent set of features that ship together.

Both are **subgraphs of the What**, not free-floating tickets.

### 7.2 'Done' as a predicate

```
feature_done(f) := every concept in f is realised & passes domain conformance
               and every flow in f is realised & passes behavioural conformance
               and every verification citing an f-element is green
               and f's agreed acceptance criteria pass

release_done(r) := all member features done
               and the cut is closed: no included element depends on an excluded one
```

- **Build order is read from the graph.** Dependency links give the valid orderings (topological); which valid ordering to take is a value/risk judgement.
- **Progress is computed, not estimated.** It is the fraction of in-scope elements that pass their verifications.
- **Done is exactly as honest as the verifications are strong** — delivery inherits the verification layer's credibility.

### 7.3 Model, not practice

This section defines the delivery **model** — the partition and the predicate. It deliberately does **not** define delivery **practice** — cadence, ceremonies, who sequences work, release rhythm. Practice is organization-specific; two teams may run the same model very differently. Practice is out of scope.

> **A note for the parent standard.** The Two Pillars defines a verdict on an individual output. `release_done` generalises it to a verdict over a **composition** — all members pass, and the composition is closed (no dangling dependency). This composition-verdict is offered upward as a candidate refinement to the Two Pillars verification concept.

---

## 8. Conformance to the Two Pillars

This framework is a conformant instantiation of the Two Pillars Specification Framework. Each construct maps to a named Two Pillars concept; an instance is Two-Pillars-conformant when it satisfies the mapped requirements.

| Two Pillars concept | This framework's construct | Section |
|---|---|---|
| **What specification** | Domain model (structure) + event model (behaviour) | [§3](#3-the-what--structure-and-behaviour) |
| **How specification** | Decisions/principles/patterns + contracts + interface standards | [§4](#4-the-how--realising-the-what) |
| **Work unit (e.g. SPMC)** | Work unit: one bounded transformation, frozen input, one artifact | [§5](#5-work-units-and-the-rationale-trace) |
| **Criteria + judge + verdict** | Verification — the required kinds and the coherence bar | [§6](#6-verification--the-conformance-bar) |
| **Derivation contract** | The typed links: derived-from, conforms-to, applies, realizes, enforces | [§9](#9-encoding-and-the-derivation-contract) |
| **Verdict (extended)** | `release_done` — a verdict over a composition | [§7](#7-delivery--bringing-the-what-to-a-verifiable-done) |

### 8.1 Conformance levels

- **Level 1 — Described.** A conformant What (domain + event model) exists as a machine-readable graph with declared bounded contexts and mappings.
- **Level 2 — Realised.** A conformant How exists; work units reference the What and How by pointer; interface contracts use standards and are generated from the domain model.
- **Level 3 — Verified.** Verifications of all required kinds ([§6.2](#62-the-required-verification-kinds)) exist, meet the coherence bar, gate acceptance, and back the rationale trace.
- **Level 4 — Delivered.** Features and releases are graph partitions; "done" is computed by predicate; progress is the fraction passing verification.

*Levels are cumulative. A claim of conformance states the highest level satisfied.*

---

## 9. Encoding and the derivation contract

The reference encoding is **RDF** for the graph and **SHACL** for constraints; any encoding that supports typed nodes, typed links, and validatable shapes is conformant. The **derivation contract** is the set of typed links that make the whole graph traceable:

| Link | Meaning |
|---|---|
| `derived_from` | this artifact/contract was produced from these upstream elements |
| `conforms_to` | this element obeys this contract or convention |
| `applies` | this work unit emits code shaped by this principle/pattern |
| `realizes` | this pattern implements this principle |
| `enforces` | this verification proves this principle/contract/model element holds |
| `changes` / `projects` | this event changes this entity / this read model projects these |

These links are what make the framework **queryable**: impact analysis ("what depends on X?"), onboarding traces ("why is this shaped this way?"), and the verification-to-principle linkage all fall out of graph queries rather than hand-maintained documents.

---

## 10. Conformance rules (normative summary)

1. A product is described as three models — What, How, Delivery — in one machine-readable graph.
2. The What has two halves: a domain model (structure) and an event model (behaviour), authored and agreed before the How.
3. The domain model is bounded contexts with explicit cross-context mappings — never one flat model; with a constraint language for invariants.
4. The event model is built from domain-typed primitives; every event changes a real entity; depth is proportional to behavioural complexity.
5. The How captures decisions/principles/patterns (declared once, referenced by pointer, each decision carrying rationale), contracts (stated checkably), and interface contracts (industry standards, generated from the domain model).
6. A work unit is single-purpose with frozen input, references the What/How by pointer, and emits a rationale trace.
7. No output is accepted without a verdict; verifications meet the coherence bar, are deterministic gates, and each names what it protects.
8. The rationale trace must be true: every claimed principle is enforced by a passing verification or the claim is retracted.
9. Delivery units (features, releases) are subgraphs; "done" is a verifiable predicate; a release cut must be closed.
10. The delivery model is in scope; delivery practice (cadence, ceremonies) is not.
11. Conformance is claimed at the highest cumulative level satisfied (Described / Realised / Verified / Delivered).
12. The framework defines shapes and rules only. Specific products, archetypes, patterns, and the content of verifications are built on the framework, not part of it.

---

> **In one line.** Describe the What and How as one graph, realise it through referenced work units, gate every output with verifications that also keep the explanation honest, and deliver by partitioning the graph until a computable predicate says you are done.

---

## License

This specification is published under a permissive license (e.g. [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/) for the text, or Apache-2.0 if you prefer a code-style license). Replace this section with the chosen license before publishing.

## Contributing

Proposals, conformance reports, and reference tooling are welcome. See `CONTRIBUTING.md` (to be added). The specification is versioned; breaking changes follow a documented deprecation policy so that existing conformant instances are never silently invalidated.
