# The Product Framework

**An open standard for specifying software as What, How, and Delivery.**

*A conformant instantiation of the Two Pillars Specification Framework. Version 1.6.0 — open specification.*

---

> **What this is.** An **open, catalog-agnostic standard** for describing a software product as three connected models — **What** it is, **How** it is built, and how it is **Delivered** — in a form that is reproducible, verifiable, and traceable. It defines the **shapes and rules**, not any particular product. Anyone can build catalogs, archetypes, and tooling against it.
>
> It is a conformant instantiation of the **Two Pillars Specification Framework** for software; every construct maps to a named Two Pillars concept ([§8](#8-conformance-to-the-two-pillars)).

*Published under a dual license: the specification text under CC BY 4.0, and accompanying shapes/code under Apache-2.0 ([License](#license)). This document describes the format; it ships no proprietary catalog content.*

---

## Contents

1. [Purpose and scope](#1-purpose-and-scope)
2. [The three models and the split](#2-the-three-models-and-the-split) · [2.1 Authoring order](#21-authoring-order--the-funnel-as-a-process)
3. [The What — structure and behaviour](#3-the-what--structure-and-behaviour) · [3.0 The product and its boundary](#30-the-product-and-its-boundary) · [3.0.1 Journeys](#301-journeys--flows-that-span-systems)
4. [The How — realising the What](#4-the-how--realising-the-what)
5. [Work units and the rationale trace](#5-work-units-and-the-rationale-trace) · [5.1 The Build seam](#51-the-build-seam--the-work-unit-emission-contract)
6. [Verification — the conformance bar](#6-verification--the-conformance-bar)
7. [Delivery — bringing the What to a verifiable 'done'](#7-delivery--bringing-the-what-to-a-verifiable-done)
8. [Conformance to the Two Pillars](#8-conformance-to-the-two-pillars)
9. [Encoding and the derivation contract](#9-encoding-and-the-derivation-contract)
10. [Conformance rules (normative summary)](#10-conformance-rules-normative-summary)
11. [Design System Conformance Profile](#11-design-system-conformance-profile) — 🅿 *Preview*
12. [Content Store Conformance Profile](#12-content-store-conformance-profile) — 🅿 *Preview*
13. [Data Conformance Profile](#13-data-conformance-profile) — 🅿 *Preview*

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
  |  decisions / principles / patterns  +  contracts  +  interface & screen standards
  v
DELIVERY  how the What is brought to a verifiable 'done' — model only
          features / releases as partitions of the What; done as a predicate
```

**The split is the central idea.** Above it lives **meaning**, expressed in the language of the business and owned by the people who own the product. Below it lives **realisation**, owned by engineering. The What is authored and agreed **before** the How is written. The line between them is where business meaning becomes technical reality, and keeping it explicit is what lets non-engineers own the What and engineers own the How without either guessing the other's intent.

**Everything is one graph.** All three models, and the links between them, form a single machine-readable graph (RDF is the reference encoding; [§9](#9-encoding-and-the-derivation-contract)). "Describe this system" is therefore a query, not a stale document, and impact analysis ("what depends on this?") is a graph traversal.

### 2.1 Authoring order — the funnel as a process

The seniority above is not only a layering; it dictates an **order in which the artifacts must come into existence**, because each is *derived from* the ones before it (§9). You cannot author a Decider before the events it is derived from exist, nor a reification rule before the AIO it reifies, nor run data conformance before the structure is declared. The order is therefore **not a practice the framework invents** — it is the topological reading of the derivation contract, the same way build order is read from the graph (§7.2). This section is normative about that *order*; it stays silent, like §7.3, about *practice* — cadence, ceremonies, who authors, when, in what tool. Those are organization-specific. What follows is forced by dependency, not by methodology.

The organizing principle is the **funnel**: constraint density rises toward the value action, and the most constrained, most *shared* things are authored first so everything downstream can reference rather than re-decide them. Hard reasoning concentrates upstream; what remains downstream is increasingly determined.

**The forward order (each step derives from the ones above):**

1. **System(s) and domain structure** (§3.2.5, §3.1). Name what is being built and the shared meaning it rests on — bounded contexts, entities, relations, invariants as validatable shapes. This is the most shared layer; everything references it. It is also adoptable *alone* (§13): a structure plus production data already yields continuous data conformance, before any behaviour is modelled.
2. **Reference data** (§3.1). Declare the constitutive instance data behaviour will depend on (valid methods, categories, taxonomies), since events and invariants reference it.
3. **Events and flows** (§3.2). Model what happens over the structure — the domain-typed events, commands, read models (with their state spaces), and the flows that sequence them.
4. **Deciders and Projectors** (§3.3, §3.4). Make the interesting behaviour and the non-trivial projections executable. These are *derivable only once events exist*, and they are **simulated sound and complete before any realisation** — the first gate, and the cheapest. Genuinely irreducible computation is named and bounded as a primitive here (§3.5).
5. **UI steps, AIOs, and the page graph** (§3.2.1–§3.2.4). Where there are screens, derive each step's data and actions from the projections and commands already modelled, typed against AIOs; lay out the page graph. Intent is permitted here but is *debt to be driven down* (§3.2.1), not licence to decide downstream.
6. **The How** (§4). Only now resolve the abstractions the What declared: reification rules per context (§4.5), content per locale (§4.6), the contracts and layout model (§4.2–§4.3), interface standards (§4.4). The How is written *against* a settled What, never alongside an unsettled one.

**Verification is interleaved, not a final phase.** The gates do not wait for the end. Behavioural simulation runs *before* realisation (step 4); layout conformance is the cheapest gate and runs *first* among the realisation checks; data conformance runs *continuously* once structure exists (step 1). Each narrowing of the funnel has its own gate, authored alongside the artifact it protects (§6). A conformant process never reaches "now we test" — it tests at every step, soonest where cheapest.

**The feedback loop — when the cycle sends you back upstream.** The order is a forward chain, but it is not a waterfall, because two signals re-enter it as fresh input:

- the **intent-reliance rate** (§3.2.1) — a measure of *under*-specification. A high rate means a downstream author had to lean on prose intent to decide something; the remedy is to go *back upstream* and model what was missing (an AIO, a context, a reification rule), then retire the intent. Feedback re-enters the funnel as a new forward pass, not as an inversion of it.
- the **data-divergence rate** (§3.1) — a measure of *over*-confidence, and the one signal that can fire *after* deployment. Rising divergence means production data no longer satisfies the declared structure: either the data is wrong or **the spec has gone stale**, and the loop returns to step 1 to reconcile the model with reality.

Both signals turn the order into a **cycle**: author down the funnel, gate at each step, and let the two metrics pull you back up when the model says too little or claims too much. This is the reproducibility → measurement → improvement chain (§1) made procedural — the spec improves because its own gaps and staleness are measured as you work and after you ship.

> **What this section does not say.** It fixes the *order* artifacts come into being, because the derivation contract forces it. It does not fix *when* on a calendar, *who* does each step, how big a slice is, or how the work is reviewed and scheduled — that is practice (§7.3), and it remains the organization's to choose. Deviate from the *order* and you will author something before the thing it derives from exists; deviate from the *practice* and you are simply running the framework your own way.

---

## 3. The What — structure and behaviour

The What is the specification every role reads and agrees on. It is organised by a three-level ownership hierarchy that fixes *where each part lives* and *what is shared* — so that several surfaces of one product draw on the same concepts and the same rules rather than re-deciding them.

**Product → Domains + Systems.** At the top is the **product** — the thing being built. A product **owns one or more domains** (the meaning and rules of its entities) and **one or more systems** (the surfaces that use them). A **system references whole domains**; it does not own them. This is what makes domain concepts genuinely shared: they are owned one level up, by the product, not duplicated inside each system.

```
Product                       the thing being built (§3.0)
 ├─ owns ─▶ Domain(s)         meaning + rules — entities, structure, data,
 │            invariants, AND behaviour (events, Deciders, Projectors)
 └─ owns ─▶ System(s)         surfaces — page graph, flows, UI steps, platform/class
              └─ references ─▶ Domain(s)   (whole domains; finer slicing is by
                                            bounded context *inside* a domain)
```

The dividing line is a single principle: **the domain owns meaning and rules; a system owns surface and journey.** Everything true *regardless of who is looking* — what a `Refund` is, that `refund_total ≤ paid_total`, which events an `Order` emits — belongs to the **domain**. Everything about *a particular way of using* the domain — which screens, which navigation, which platform — belongs to the **system**. The decisive test: **if two systems would disagree about it, it is system-level; if they must agree, it is domain-level.** Behaviour must agree (a refund is a refund in the customer app and the admin tool alike), so behaviour lives in the domain; navigation differs between them, so it lives in the system.

The payoff is that two systems over the same domain share not just *vocabulary* but *rules* — the same Deciders, the same invariants, the same events — and differ only in how they surface them. "Use the same concepts across systems" taken to its end: same concepts *and* same behaviour, authored once.

### 3.0 The product and its boundary

The **product** is the root of the What: the named thing being built, the owner of the domains and systems that compose it. It carries little of its own — a name and purpose — because its job is *ownership and boundary*, not behaviour:

- it **owns one or more domains** (§3.1) — the shared meaning and rules. Multiple domains are bounded from each other (an `Ordering` domain and a `Catalog` domain), with cross-domain correspondences declared, never assumed.
- it **owns one or more systems** (§3.2.5) — the surfaces. A single-system product is the common case; a product becomes multi-system when one body of meaning is surfaced more than one way (a customer app, an admin website, an operator CLI).
- it is the home for **product-wide concerns**: the direction the product is moving toward (§7.3), product-level quality demands (§3.6), and **journeys** (§3.0.1) — end-to-end paths that span systems — sit here, because they are properties of *the product*, not of any one system.

The product's boundary is exactly the set of domains and systems it owns. A second product is a second root: it may *reference* a shared domain across a declared boundary (the Translation pattern, §3.2.0, is how its systems would communicate), but it does not absorb it. The product node answers "what, in total, is being built, and what shared meaning holds it together?" — a question the framework previously left to folklore.

#### 3.0.1 Journeys — flows that span systems

A flow belongs to exactly one system (§3.2.5). But a *business* process often does not: a customer places an order in the shop, and staff fulfil it in the admin tool — one journey, two systems. Modelled only as two disconnected per-system flows, the end-to-end path is invisible; no one can trace "order to fulfilment" as a whole. A **journey** makes that path first-class **without re-coupling the systems**.

A journey is a **product-level, derived composition**: a declared path through single-system flows, linked at their crossing points, where **every crossing is a Translation** (§3.2.0). It owns nothing — it references flows and Translations that already exist — so it adds *visibility*, not ownership or new behaviour. Flows stay single-system and system-owned; the journey is a view *over* them, owned by the product because only the product sees across its systems.

```
product  "order to fulfilment" journey
  acme-shop  flow place-order      ──┐
                                     │  ⟶ Translation: ev-order-placed (acme-shop)
                                     │                 → cmd-accept-fulfilment (acme-admin)
  acme-admin flow fulfil-order     ◀─┘
```

The rules that keep it from undoing the system boundary (§3.0):

- **Every crossing is a Translation, or it is a finding.** A journey may not hop between systems by any other means — not a direct reference to another system's screen, not a shared table, not an assumed call. The only sanctioned cross-system link is the event-driven Translation channel (§3.2.0): the crossing reads an event from the source system and issues a command into the target. This is what guarantees a journey adds *no new coupling* — it composes exactly the links the framework already permits.
- **A journey references; it does not contain.** It names flows (by reference) and the Translations between them. It introduces no new flow, no new behaviour, no new screen. Remove a journey and nothing else changes — the flows and Translations it pointed at are untouched. This is the same derived-view discipline as a feature (a referenced slice, §7.1) and direction (a computed gap, §7.3).
- **Systems stay independent.** Because the only link is a Translation reading published events, `acme-shop` still does not depend on `acme-admin`'s internals — the journey sees the path, but neither system's spec reaches into the other. Either system can be specified, realised, and versioned alone; the journey simply records that their flows compose into a larger path.

The payoff is end-to-end visibility that disconnected flows cannot give: a journey is a path through Translations, so the framework's **value-anchoring** (every path terminates in a value action, §6/§7) and **blast-radius** (the footprint of a change, a pure graph traversal) now extend *across* systems. "What is the full footprint of order-to-fulfilment?" becomes a single traversal spanning the shop and the admin tool — which a pile of per-system flows could never answer. A journey is to cross-system process what a feature is to single-system behaviour: a derived, referenced, checkable slice of the graph.

### 3.1 The domain model — structure and data

The shared definition of **what concepts mean** and how they relate. A domain model has **two sides**, and conflating them is a common defect:

- the **structure** — what *constitutes* a valid model: the entity types, relations, cardinalities, and invariants. This is what developers usually mean by "domain model" (the DDD, type-level sense), and it is the SHACL-*shape* side of the reference encoding (§9).
- the **data** — the actual populated facts: *which* products exist, what they are called, the real entities. This is the side data practitioners work from; they validate a model against what the data *actually is*, and they catch what the structure side cannot see (a field that is null in 90% of rows, an enum value the schema never declared, two "distinct" entities that are one row in practice). It is the RDF-*instance* side of the same encoding.

A model trusted only from the structure side is asserted top-down and never reconciled with ground truth — which is exactly why data practitioners distrust developer domain models. The framework therefore makes **both sides first-class**, and distinguishes two kinds of data by their relationship to the model.

**Structure — conformance requirements:**

- **Bounded contexts, not one flat model.** Within a context a term has exactly one meaning (the **ubiquitous language**); the same word may mean different things in different contexts. Cross-context correspondences are **explicit declared mappings**, never assumed. (This is what resolves "is a User a Customer?" rather than restating the confusion.)
- **Entities, relations, value objects, invariants.** Relations carry cardinality **and rationale**. Invariants are stated as machine-checkable constraints. An **entity** has identity that persists while its attributes change; a **value object** is defined wholly by its attributes and is immutable (a `Money`, an `OrderNo`). Which fields an entity carries is not a free choice: an aggregate holds *exactly its invariant-bearing state* — what its Deciders must read at decision time — and everything else a screen wants is a Projector's output, not a stored field (§3.4).
- **Machine-readable, with a constraint language.** The model must be expressible as a graph with validatable shapes (RDF + SHACL is the reference; an equivalent is conformant). A diagram alone is not a conformant domain model — it cannot generate or validate.

**Data — reference data is What; production data is an oracle:**

- **Reference data is part of the What.** Some instance data is *constitutive* — the system's behaviour is undefined without it: the set of valid shipping methods, the tax categories, the product taxonomy, the currencies accepted. This is **reference data**, and it lives in the What because the model and behaviour reference it: a `single-select` of shipping options (§3.2.2) draws its options from declared reference data, not from thin air, and an invariant like "tax category must be one of the declared set" is checkable only because the set is modelled. Reference data is versioned and governed like the rest of the What.
- **Production data is the validation oracle, not part of the What.** The live entities — the 4,200 products, the real orders — are *not* specification; they are *populated* facts the running system holds. But they serve a role the framework values: they are an **oracle the structure is checked against.** "Does real data conform to the declared shapes?" is a verification — **data conformance** (§6.3) — and it catches model defects nothing else will. A structure that no production data satisfies is as suspect as a Decider no scenario exercises: the model earns trust by real data passing its shapes, not by being declared. This is the data practitioner's lens given teeth.

> **Data conformance reads in both directions — and never closes.** Every other conformance check in the framework runs one way: the artifact is judged against the spec, and a failure means the *artifact* is wrong. Data conformance is the first check whose failure is **genuinely ambiguous**: when production data violates a shape, either the data is wrong *or the spec is stale*, and the framework does not presume which. That ambiguity is the mechanism, not a weakness — it is the one place the running system can **indict the specification.** A shape that real data persistently violates may not be catching bad data; it may be reporting that the model no longer describes the system, and the correct response may be to fix the spec, not the data. This inverts the usual authority: elsewhere the spec is the truth the world is held to; here, production data is a *witness* against a stale spec.

> **Authored once, asserted continuously.** Every other verification is a build-time gate — it runs once, before acceptance. Data-conformance shapes are different in kind: authored once in the What, they can be evaluated against live data **forever** — on every write, or on a schedule — so the same artifact is both a build-time gate on the model's coherence *and* a standing **production monitor**. This dual-temporality (write once, assert continuously) is what lets a system know, at any moment, whether the data it holds is valid against what it believes should be true.

> **Spec staleness becomes measurable — the data-divergence rate.** Because the assertion runs continuously, the rate at which production data diverges from the declared model is a direct, ongoing signal of how far the spec has fallen behind reality. This is the **data-divergence rate**, a first-class metric and the *over-confidence* counterpart to the intent-reliance rate (§3.2.1): where intent-reliance measures under-specification (the model says too little), data-divergence measures over-confidence (the model claims something reality no longer supports). A spec with a rising divergence rate is a spec going stale — now *visible as it happens*, rather than discovered when something breaks. This closes the reproducibility → measurement → improvement chain (§1) onto the spec's own fidelity.

> **The constitutive/populated test.** If changing a datum changes what the system *means* (remove a shipping method and a behaviour becomes impossible), it is **reference data** and belongs in the What. If it is just more rows of the same kind (one more product), it is **production data** and belongs to the running system, where it serves as the conformance oracle. The line is not "is it data" but "is it specification."

### 3.2 The event model — behaviour

The description of **what happens over time**, peer to the domain model, in the same graph. Conformance requirements:

- **Built from domain-typed primitives.** Triggers, events, commands, read models (Views), and UI steps each reference domain concepts: a **trigger** issues a command, an event **changes** an entity, a command **targets** an aggregate, a read model **projects** entities. Behaviour may not reference structure that does not exist. The blocks wire into four named patterns (§3.2.0). A read model also declares (or makes inferable from its query shape) its **state space** — at minimum `present`, plus any of `loading`, `empty`, or `failed` it can actually exhibit: a projection over a possibly-empty collection can be `empty`, one backed by something that can be pending can be `loading`, one whose delivery can fail can be `failed`; a guaranteed-singleton, synchronous projection has none of these. The state space is mostly *derived* from the projection's shape and declared only where the shape does not settle it.
- **Depth is proportional to behavioural complexity.** Concepts with rich or historical behaviour get full event models; simple create/read/update/delete concepts get a minimal one or none. The framework requires that the *interesting* behaviour be modelled, not that every triviality be ceremony.
- **Owned by product and design.** Its natural form — a timeline with interface steps — is readable and signable by non-engineers; it is the bridge between concepts and screens.

> **Why structure and behaviour are one graph.** An event is never free-floating: it always changes a domain entity. Modelling them in one graph means structure and behaviour cannot drift apart, and a single question — "what happens to this concept?" — returns both its shape and the flows that change it.

#### 3.2.0 Building blocks and patterns

The event model is built from **four building blocks**, wired into **four patterns** (the framework adopts the Event Modeling vocabulary here, because it is a precise, widely understood naming of shapes the framework already uses; the patterns name the *topology* of the event graph, while the Decider (§3.3) and Projector (§3.4) supply the *executable semantics* inside the Command and View blocks). The blocks:

- **Trigger** — *what initiates a command.* A Trigger has exactly one of three **sources**: a **user** (through a UI — this is the UI step, §3.2.1), an **external caller** (an API request from outside the system), or an **automated process** (a step that reads a View and issues a command with no human in the loop). Generalising "what issues a command" into one block is what makes automation and cross-system communication expressible as patterns rather than special cases — a robot and a user are the *same shape*, differing only in source.
- **Command** — an intention to change state, carrying its parameters. A Command targets an aggregate and is decided by that aggregate's Decider (§3.3), which accepts it into events or rejects it.
- **Event** — a business fact that happened and was recorded. The irreducible unit of truth; everything else is derived from events. Named in the past tense, in the ubiquitous language, carrying the data the fact entails.
- **View** — a query that reads, interprets, and curates already-recorded events for a consumer (a UI step, a report, or an automated Trigger). A View is a read model, made executable by a Projector (§3.4). Its defining honesty: **a View can present only information some event already carries** — if a View needs a field no event supplies, the model is incomplete, surfaced before any code (the Projector's event-coverage check, §3.4).

The blocks compose into exactly four **patterns**, each a named, checkable shape:

| Pattern | Shape | Meaning |
| --- | --- | --- |
| **Command** | `Trigger → Command → Event(s)` | a state change, from what initiates it to the facts it records |
| **View** | `Event(s) → View` | recorded facts curated for a consumer; reads only events that exist |
| **Automation** | `Event(s) → View → Trigger(automated) → Command → Event(s)` | the system acting on itself — a process observes a View and issues a Command |
| **Translation** | `Event(s) [system A] → View → Trigger(automated) → Command → Event(s) [system B]` | one system telling another that something happened |

- **Command and View** are the everyday shapes, and the framework already enforced them under other names: a Command pattern is a Decider invocation (`decide` produces the events); a View pattern is the Projector's no-foreign-events / event-coverage discipline (§3.4). Naming them makes the shapes themselves queryable — *every Command traces to a Trigger and produces ≥1 event; every View reads only declared events* — as conformance checks (§6.3).
- **Automation is the normative way to model self-acting behaviour.** When the system should *do something automatically*, it is modelled as an Automation pattern: an automated Trigger watches a View and issues a Command, exactly as a user would — observe, then act. The discipline the pattern enforces is that **the automated Trigger contains no business logic**: it reads a View (typically a *to-do list* projection — one row per outstanding action) and, for each row, issues one Command; the resulting Event updates the View so the row is struck off and the same work is never triggered twice. All decision-making stays in the Decider the Command reaches; the Trigger only *notices and calls*. This keeps automated behaviour as verifiable as user behaviour — same Decider, same simulation, same gates — with the only addition being the View-driven trigger.
- **Translation is the normative way sibling systems communicate.** Where a What describes several systems over one domain (§3.2.5), a system tells another that something happened **only** through a Translation pattern: an automated Trigger reads a View over the *source* system's events and issues a Command into the *target* system, producing the target's own events. The framework imposes the pattern's standing restriction: **the read side reads events from exactly one source system; the write side may fan out** to several targets (a publish/subscribe translation). This makes cross-system coupling an explicit, directional, traceable edge in the graph — never an implicit shared table or an assumed call — and it is the event-model counterpart to the federation discipline the domain side already requires (no shared mutable state across systems; communication is by recorded fact).

> **Slicing — the unit of work.** A single pattern instance on the graph is a **slice**: the smallest coherent unit that can be handed to an implementer, because it carries everything from trigger to recorded fact. This is the same object the framework calls a work unit (§5) and the building block a feature slices into (§7.1) — Event Modeling's "slice" and the framework's "feature as a flow-slice" are the same cut, named from two traditions. A slice is implementable precisely because it is a complete pattern: a Trigger, the Command it issues, the Decider that decides it, and the Events that result (or a View and the events it reads).

#### 3.2.1 The UI step — the What of a screen

A **UI step** is the point in a flow where a person sees something and acts. It is **What**, owned by product and design: it says what a screen is *for* and *means*, never how it looks. How it is composed and rendered is the screen-composition contract in the How (§4.5); the two meet at a verified seam. The split is not a matter of discipline but of **type**: a UI step may reference only **Abstract Interaction Objects** (§3.2.2) — `single-select`, `trigger-action`, `text-entry`, and the like — never a concrete control (a dropdown, a button) from a design system. An AIO is meaning; a concrete control is realisation. Because the two are distinct kinds of node in the graph, "a UI step naming a dropdown" is not a style lapse to be caught by review — it is a structural violation a verification rejects (§6.3), the same way the layout allowlist rejects a misplaced file. The fusion the framework forbids (§2) is made impossible by construction, not discouraged by instruction.

A conformant UI step declares two layers. The first is the **buildable core** — four facts, each already a node or edge in the model, so the step is an *annotation on the flow*, not a new artifact:

- **Intent** — one sentence, in the ubiquitous language, for what the user is trying to accomplish here. Intent is **not modelled specification** — it is the marked residue of what the model does not yet determine (see *Intent is specification debt* below). It is permitted, but it is a liability to be driven down, never the field that "gives the implementer good judgment."
- **Information shown** — the read-model **projection** this step surfaces (it `projects`, §3.2), presented through one or more **display AIOs** (`display-value`, `display-collection`, …). The What names *which projection* and *which abstract interaction*, never how it is laid out. ("Surfaces the order-summary projection as a `display-collection`" — not "in a table.")
- **Actions available** — the **commands** the user may issue here (exactly the commands valid at this step — the ones a Decider `handles`, §3.3), each offered through an **action or input AIO** (`trigger-action`, `single-select`, `text-entry`, …). The What names *which decision* and *which abstract interaction*, never the control. ("`single-select` of a shipping option, `trigger-action` to confirm" — not "a dropdown and a primary button.")
- **Transitions** — given an action or a resulting event, which page follows. A transition is a `navigate` edge in the application's single **page graph** (§3.2.4); the step names its position in that graph. (These edges were undersold as "a flow's internal topology" — they are in fact the edges of the whole application's navigation.)

The second layer is **modelled meaning** — declarations that shape realisation without prescribing it. These are *not* residue: each is a checkable model element the seam verification (§6.3) enforces, and each is a thing intent gets *promoted into* (see below):

- **Emphasis** — what matters most at this step, so the How knows what to make prominent. ("The total owed is the decisive figure" — not "make it large and bold.")
- **Projection states as meaning** — a projection is not always a present value; it can occupy any state in its declared **state space** (§3.2), and what each state *means to the user* is a behavioural fact, not styling. A UI step's state annotations are both **constrained** and **covering**: it may annotate only states the projection actually has (no "empty" meaning for a projection that cannot be empty), and it must give a meaning to *every* state the projection can be in, or explicitly waive one with a reason. This is the UI analogue of the Decider's command-coverage rule (§3.3) — exhaustiveness over the projection's state alphabet — and the dangerous case it catches is the *forgotten* state: a projection that can fail whose screen never says what failure means. The waiver is the escape hatch a Decider doesn't get, because some states are legitimately ignorable (a load too fast to perceive); the waiver must say why. For `failed` specifically, the *meaning* is What ("the user must know it can't be shown and how to recover"); the failure *mechanism* is How — the same split as an accessibility criterion versus its discharge (§3.2.3).
- **Accessibility obligations** — *not* prose about what "must be perceivable," but a set of **WCAG 2.2 success criteria** the step must satisfy, referenced as ingested entities (§3.2.3). Most are inherited from the AIOs the step uses (a `text-entry` carries its labelling criteria, an image-bearing `display-value` carries 1.1.1 Non-text Content); a step may add screen-specific criteria. This replaces the old free-text "accessibility intent": a criterion is a checkable entity with a known verification type, where prose was an unverifiable wish.
- **Content references** — the standing, authored words a screen carries that are *not* projected data and *not* a control label: a heading, explanatory body copy, the prose of an empty or error state, help text, legal text. These are referenced by **content key with a declared role** (heading / body / empty-message / error-message / help / legal / …), never written as literal strings in the UI step. The role and the obligation are What ("this page needs a heading that states its purpose; its empty state needs a message conveying the way forward"); the words themselves are resolved by the How against a **content store** (§4.6). This is the same move as tokens-not-literals and AIOs-not-widgets, applied a fourth time to *copy* — and for the same reason: a literal baked into the What cannot be translated, swapped, or verified, while a keyed reference can. A page may consist *entirely* of content references and no projection or command — a pure-content page (an "about" or informational page) is simply a page whose interaction is "read," needing no new page kind.

> **Intent is specification debt, not implementer autonomy.** It is tempting to call intent the field that lets an implementer exercise good judgment — but that would run the funnel principle backwards. Everywhere else the framework drives unmodelled specification *out*, concentrating hard reasoning upstream so execution is determined; a UI step that leaned on an implementer's taste to decide its realisation would be exactly the under-specification the funnel is meant to expose. So intent is treated as what it is: **specification that is not modelled yet.** It is allowed (the framework refines incrementally; not everything can be modelled upfront), but it is a *measured liability*, governed by one rule —
>
> **Whatever intent was used to decide must be promoted into the model.** If an implementer consulted intent to choose a control, a grouping, or what to stress, that choice was a missing model element — a missing AIO, a missing context of use, a missing reification rule (§4.5), or a missing emphasis/state/accessibility declaration above. The remedy is to model it as data and *retire that part of the intent*. Intent therefore shrinks monotonically toward a pure statement of purpose; the residue that remains is only the irreducibly-human "why this step exists," itself a candidate for the task/flow model later.
>
> **The metric.** Because intent is debt, it is measurable, which is what makes it improvable (reproducibility → measurement → improvement, §1): the **intent-reliance rate** — how often realising a screen required consulting intent to settle something the model did not determine. Zero means the model fully determines the UI; a high rate means the funnel is leaking at the UI stage, the precise UI analogue of an implementer role needing an over-capable model. Driving the rate down *is* the act of moving UI specification from prose into the graph.

> **Why this is symmetric to the Decider — and where intent sits.** The Decider (§3.3) makes behaviour *executable from the model*: its signature is derived, and the one thing genuinely authored is its decision *logic* — real, irreducible specification. The UI step makes a screen *buildable from the model* in the same way: its data and actions are derived, and the genuinely authored specification is the **modelled** part — the AIO typing, the emphasis/state/accessibility declarations, and (in the How) the reification rules. *That* is the honest peer of Decider logic: modelled choices, checked not assumed. **Intent is not part of this symmetry.** It is not authored specification of equal standing; it is the residue that has not been modelled yet, marked as debt and slated for promotion into exactly those modelled elements. Both modelled sides impose a checked How-side constraint — the Decider constrains the pure core (§4.2), the UI step constrains the screen composition (§4.5) — and this is the funnel principle for UI: the What fixes what the screen *means* as modelled data, leaving the How only *how it presents*, with intent measuring how far that modelling still has to go.

#### 3.2.2 Abstract Interaction Objects and the context of use

The vocabulary a UI step is typed against is the set of **Abstract Interaction Objects (AIOs)** — the *kinds of interaction* a user can have, independent of any device, modality, or design system. This is the Abstract User Interface layer of the established four-level model the framework follows for UI (Task & Concepts → Abstract UI → Concrete UI → Final UI); the domain model and flows are the Task & Concepts layer, AIOs are the Abstract UI layer, the design system's components (§4.5) are the Concrete UI layer, and running code is the Final UI. Adopting this layering is what lets one What drive several different concrete UIs without changing its meaning.

**An AIO is meaning, not a widget.** `single-select` means "choose exactly one from a set"; it is *not* a dropdown, a radio group, or a segmented control — those are the concrete forms it can take. The distinction is load-bearing precisely because the same abstract interaction reifies to different controls in different situations: a `single-select` over three options on a tablet is well served by a segmented control, the same `single-select` over forty options on a phone by a searchable list. The choice between them is a real UX decision — but it is a decision about *realisation in a context*, not about what the step means, which is why it belongs below the What, not in it.

**The base set is normative; the set is extensible.** A conformant instance recognises a stable core of AIOs, and may register additional ones against the same definition (an AIO is a named, modality-independent kind of user interaction, with a declared arity over domain data where applicable). The core, at minimum:

| AIO | Means | Typed over |
|---|---|---|
| `trigger-action` | invoke an operation | a command |
| `single-select` | choose one from a set | a command parameter / a domain enumeration |
| `multi-select` | choose any number from a set | a command parameter / a collection |
| `text-entry` · `numeric-entry` · `date-entry` | supply a typed value | a command's payload field (its type from the domain model) |
| `display-value` | show a single datum | a projected field |
| `display-collection` | show many of a kind | a projected collection |
| `navigate` | move between interaction spaces | a transition (§3.2.1) |
| `edit` | revise an existing value in place | a field of a projected entity + the command that updates it |

An adopter adding, say, `range-select` or `reorder` declares it the same way and it becomes referenceable; the core stays small so that the common cases are interoperable across instances.

**Input AIOs derive their shape from the domain model.** This closes the form/data gap: a `text-entry` or `single-select` is not a free-floating field — it is bound to a **command payload field**, and that field's type, constraints, and (for selects) its allowed values come from the domain model (§3.1). A form is therefore a composition of input AIOs over a command's payload, derived — its fields are exactly the command's parameters, its validation exactly the domain invariants — not authored on the screen. What the user can *enter* is fixed by the command they are composing, just as what they can *do* is fixed by the commands valid at the step.

**The context of use is a declared model.** Per the four-level model, context-of-use enters at the Abstract→Concrete boundary, not above it: the AUI is deliberately context-independent, and the *same* AIO is reified differently depending on context. A conformant instance declares its relevant **contexts of use** — at minimum **form factor** (e.g. phone, tablet, desktop), **modality** (e.g. pointer, touch, voice), and **platform** (e.g. iOS, Android, web), and optionally user type and physical environment — as named elements. **Platform is a context dimension like any other**: the same `single-select` reifies to a UIKit picker on iOS, a Material control on Android, and a `<select>` on web, so "iOS, Android, or both" is simply *which platform contexts the system targets* (§3.2.5), and "both" means two declared contexts the design system (§11) must reify into. Context-of-use is What-side knowledge (a fact about who uses the system and where), but it carries no realisation; it is the parameter the How's reification rules (§4.5) are written against. This is what makes "phone → searchable list, tablet → segmented control" — and "iOS → picker, web → select" — a *declared, traceable rule* rather than an implicit decision buried in code.

**Interaction class is the dimension that gates the others.** One context dimension is not peer to the rest but *senior* to them: the **interaction class** — at minimum **GUI** (a graphical, 2D pixel canvas) and **TUI** (a text/terminal character grid), extensible to others (voice-only, for instance). It is senior because **the class determines which other dimensions are even meaningful.** Choosing GUI brings *form factor* (phone / tablet / desktop screen) and *pointer/touch modality* into scope — the variations all share one substrate: a pixel canvas, arbitrary layout, rich colour, images, a pointer. Choosing TUI brings a *different* set of dimensions into scope and rules form factor out: a TUI has no "phone vs tablet," it has **terminal capabilities** — grid dimensions (e.g. 80×24 vs resizable), colour depth (monochrome / 16 / 256 / truecolor), glyph set (ASCII vs Unicode/box-drawing), and key support — and it is keyboard-driven, with focus moved by tab/arrow rather than placed anywhere by a pointer. So the dimensions are not a flat list; the class is chosen first, and it selects the sub-dimensions that apply:

| Interaction class | Substrate | Dimensions it brings into scope |
| --- | --- | --- |
| **GUI** | 2D pixel canvas, pointer/touch, rich colour, images | form factor (phone/tablet/desktop), pointer-vs-touch modality |
| **TUI** | character grid, keyboard-only, constrained colour, no images | terminal capabilities: grid size, colour depth, glyph set (ASCII/Unicode), key support |

The AIO vocabulary itself does **not** change across classes — `single-select`, `trigger-action`, `text-entry` are abstract enough to mean something in a terminal (a `single-select` becomes a highlighted list the user arrow-keys through, a `trigger-action` a labelled key or reverse-video button). That a TUI is just a different *reification* of the same abstractions is the dividend of typing against abstract interaction objects rather than widgets (§3.2.1): one What, realised over substrates as different as a phone screen and an SSH session, without changing its meaning. A system declares the interaction classes it targets (a CLI tool is TUI-only; a consumer app is GUI; a developer tool may ship both), and the design system (§11) must supply reification for each declared class — or honestly declare an abstraction unreifiable in it (§4.5).

#### 3.2.3 Accessibility as ingested criteria, not prose

Accessibility is the same problem as the AIO/CIO split, one level down: "the error must be perceivable without colour" is *prose* — an unverifiable wish, the very thing the framework forbids everywhere else. The standard already exists and is precise, so the framework does not restate it — it **ingests it as entities** (the §4.4 "use the standard" discipline). The normative reference is **WCAG 2.2**, version-pinned; an instance ingests its structure from the W3C machine-readable source as a graph of **principle → guideline → success criterion → conformance level (A / AA / AAA)**. These are referenceable nodes, not authored text.

**Each criterion carries a verification type — and this is the honest core.** Not all of accessibility is machine-decidable; in practice automated tooling covers only a minority of the criteria, the rest requiring human judgement (the same "globs match paths, not meaning" limit as §4.5). The framework refuses to pretend otherwise, so every criterion an instance references is tagged:

| Verification type | Means | In the verdict |
|---|---|---|
| **machine** | deterministically checkable (e.g. contrast ratio 1.4.3, target size 2.5.8, presence of a programmatic name 4.1.2) | a real gate — fails the build |
| **assisted** | a tool can flag candidates but a human confirms (e.g. is this text *actually* alternative text for the image) | tool-surfaced, human-confirmed, recorded |
| **manual** | requires human evaluation (e.g. is the focus order *meaningful*, is the alt text *descriptive*) | a recorded **attestation**, never asserted automatically |

A conformant accessibility verdict therefore states a **conformance level and its basis** — e.g. "Level AA: all machine criteria green; assisted and manual criteria attested by «who», «when»" — and never a bare "accessible: true." The machine criteria are deterministic gates exactly like every other verification (§6.2); the manual ones are **human knowledge entering through a frozen boundary** (the same discipline as a discovery record or SME input, ADR-style provenance): an attestation is a dated, attributed assertion that a named criterion was evaluated and met, and it is consumed as a frozen input, not re-litigated mid-verdict.

**Obligations attach in two layers, and are mostly derived.** Because AIOs are entities, accessibility obligations can hang off the *interaction type*, where they naturally belong:

- **AIO-level defaults (inherited).** Each AIO carries the success criteria intrinsic to its kind — `text-entry` carries labelling and name/role/value criteria (1.3.1, 3.3.2, 4.1.2); a `display-value` over an image carries 1.1.1; `trigger-action` carries target-size and focus-visibility criteria; `single-select` carries the criteria for grouped controls. A UI step that uses these AIOs **inherits** their criteria without restating them — the obligation set is *derived*, the same way the domain footprint of a feature is derived (§7).
- **Step-level additions (declared).** A step may add criteria its particular content demands (a step presenting a timed offer adds the timing criteria; a media step adds captions/transcript criteria). These are the small declared delta on top of the inherited union.

The screen's full obligation is the union, computed — so adding a `text-entry` to a screen automatically brings its labelling obligations, and removing it removes them, with no hand-maintained per-screen checklist to drift. The How (§4.5) discharges these obligations by reifying to design-system components that satisfy them; the seam verification (§6.3) checks the machine criteria and confirms the attestations exist for the rest.

#### 3.2.4 The page graph — navigation as one graph

Navigation is not a separate "shell" layered above the screens; it is **one graph whose nodes are pages (UI steps) and whose edges are `navigate` transitions.** A page's transitions (§3.2.1) are edges in this graph, and every way a user moves through the application — within a task or across tasks — is an edge of the same kind. There is no second navigation model to maintain; there is the page graph, and navigation *is* its edges. This reuses the `navigate` AIO (§3.2.2) at graph scope: no new primitive, just interaction one level up.

**A flow is a named subgraph.** Just as a feature is a subgraph of the event model (§7), a flow is a named, connected region of the page graph with a declared **entry page**. Flows do not *own* their navigation — they partition one shared graph. This is why "the checkout flow" and "the browse flow" can share pages and link to each other without either containing the other: they are two named regions of the same graph.

**Each system has a declared root.** The graph has a distinguished node per system (§3.2.5), the **system root** — the place a user is before entering any of that system's flows. Its **out-edges are the global destinations**: the flows reachable from anywhere within the system, which a renderer draws as the system's primary navigation. A `navigate` AIO *at the root* is exactly the burger menu / tab bar / sidebar, and it reifies per context of use by the same rules as any other AIO (§4.5) — phone → drawer, tablet → rail, desktop → persistent sidebar. Global actions (a `trigger-action` valid app-wide, such as sign-out or a global search) are likewise edges out of the root rather than out of a step. When a What describes several systems (§3.2.5), each has its own root and its own page subgraph; they share the domain model but not their navigation.

**"Top-level" is derived, not a category.** A page is a top-level destination iff it has an inbound edge from the root; a page is *nested* (like a checkout step) iff it is reachable only from another page. Nothing is tagged "top-level" by hand — it falls out of the graph's edges, the same way a feature's domain footprint falls out of its flow slice (§7). The application's primary navigation is therefore *computed*: it is the set of the root's out-edges, and it changes automatically as flows are added to or removed from the root.

> **Why this is the right shape.** The alternative — a separate application-shell model sitting above flows — would duplicate the transition machinery at a second level and create two places navigation can drift. Treating the app as **one page graph with named subgraphs** keeps a single source of truth: the same `navigate` edge, the same reification, the same impact analysis ("what can reach this page?" is a graph query). The chrome a renderer draws around a screen is not separate content; it is the reified root-navigation of the very graph the screen lives in.

#### 3.2.5 The system — what is being described

A **system** is a *surface*: one way a product (§3.0) is used. The product owns it; it **references the domains** whose meaning and rules it surfaces (§3.1), and it owns the *journey* — the page graph and flows — over that shared behaviour. It carries:

- a **name** — what the system is called ("Acme Shop");
- a **kind** — what sort of thing it is (application, website, service, CLI, …);
- a **purpose** — one sentence of what it is for, in the ubiquitous language;
- the **domains it references** — whole domains (§3.1) it draws on (`Ordering`, `Catalog`); finer slicing is by bounded context *inside* a domain, not by a system cherry-picking entities;
- its **target platform contexts** — the platform dimension of context of use (§3.2.2): a customer app might target `{iOS, Android}`, an admin tool `{web, desktop}`. "Both" is simply two declared contexts, and the design system (§11) must reify into each.

A system **owns its page graph** (its root, §3.2.4) and **its flows** — but a flow does not own behaviour; it *arranges* it. A flow is a journey that triggers domain commands and reads domain projections (§3.2.0), referencing the domain's events and Deciders rather than containing them. So the behaviour a flow invokes is the domain's (shared, identical across systems); only the *sequence and surface* are the system's. A flow belongs to exactly one system; the behaviour it invokes is shared by all systems referencing that domain.

This is the point of allowing more than one system. A customer-facing app and an internal admin website may both reference the `Ordering` domain — and they get the **same** `Order` Decider, the **same** refund invariant, the **same** `ev-order-placed` event for free, because those live in the domain. The two systems differ only in *surface*: different roots, different flows, different platforms, screens the other does not have. Same concepts and same rules, authored once; divergent journeys, authored per system.

The system node is also the home for **system-specific quality demands** (§3.6) — the latency or uptime of *this* surface (the customer app's checkout latency differs from the admin tool's). Product-wide demands (a residency rule binding every system) sit on the product (§3.0); per-system ones sit here; per-flow ones on the flow.

> **System identity is What; deployment identity is How.** The system node carries what the surface *is and reaches* — its name, kind, purpose, referenced domains, target platforms. It does **not** carry its concrete address: the production domain name (`shop.acme.com`), the App Store bundle identifier, the chosen runtime. Those are realisation, and they live in the **infrastructure/runtime contract** (§4.2) — they vary per deployment (staging and production differ), they are frozen once chosen, which is exactly that contract's remit. The store-facing *display name* may itself be content (§4.6), resolved and localized. The What says "Acme Shop is a consumer e-commerce application, referencing the Ordering and Catalog domains, targeting iOS, Android, and web"; the How says "iOS bundle `com.acme.shop`, web served at `shop.acme.com` from these runtimes." Meaning and reach above the line; concrete address below it.

### 3.3 The Decider — the executable form of behaviour

The event model says *what* happens; a **Decider** says it *executably*. For a consistency boundary (typically an aggregate), a Decider is a pair of pure functions:

> **In business terms.** A Decider is where the **business rules** live — but it is more than the rules. The rules are its *rejections* (an empty cart cannot check out; a refund cannot exceed the payment), and a Decider also says *what results* when a command is allowed (the events it emits). So it captures both halves of a behavioural rule that prose usually leaves implicit: *what is permitted* **and** *what then happens*. It is the executable specification of that behaviour — not its implementation (that is the How); the implementation is later checked against it (§6).

```
decide(state, command) -> Accepted[events] | Rejected[reason]
evolve(state, event)   -> state
```

It is **optional and proportional** — author one for the consistency boundaries whose behaviour is interesting (real decisions, real invariants), and none for trivial create/read/update/delete, exactly as the depth rule (§3.2) already says.

What makes the Decider conformant is that its **signature is derived from, and validated against, the event model** — only its decision *logic* is authored. The graph already specifies every part of the signature:

- the boundary it **decides for** is an aggregate entity;
- the commands it **handles** are exactly the commands that `target` that aggregate;
- the events it may **emit** are exactly those its handled commands are declared to `emit`;
- how it **evolves** state comes from the events that `change` that aggregate;
- the **rejections** are the aggregate's invariants, now executable rather than merely stated.

Three conformance rules keep the authored Decider from drifting from the model it claims to execute:

- **No foreign commands** — it may only handle commands that target its aggregate.
- **Command coverage** — it must handle *every* command that targets its aggregate, or behaviour is left unspecified for some command.
- **Output-alphabet containment** — it may only emit events that a command it handles is declared to emit; it may not invent outputs.

The Decider sits at the **boundary between the What and the How**: its signature is pure What (derived from the model), its logic is the executable behavioural specification, and it becomes the **oracle** the realised behaviour is later checked against (§6). It earns its place twice over —

- **before realisation**, the Decider is *simulated* against scenarios drawn from the flows (a flow gives a *given* of prior events, a *when* command, and a *then* of expected events). This proves the behaviour is **sound and complete before any code exists** — invalid commands are rejected for the right reason, valid ones produce the right events, and no view needs a field no event carries. This is the first gate, and the cheapest, because it runs as pure function calls with no infrastructure.
- **after realisation**, the same scenarios run against the realised code's behaviour, which must produce **identical** outputs (§6.3, behavioural conformance) — turning that check from "looks complete" into "computes the same thing."

> **The realisation constraint this implies.** For the after-realisation check to be possible, the realised code must keep its decision logic in a pure core, separable from input/output. A conformant How therefore states this as a contract (§4.2): decision logic is pure and isolable. A What-side artifact (the Decider) thus imposes a How-side constraint — and that constraint is itself verified, not assumed.

> **The Decider is only half the pair.** The signature above includes `evolve(state, event) → state`, but everything in this section formalizes `decide`. The `evolve`/projection half — how an event folds into a read model — is a peer artifact, the **Projector** (§3.4), and it must be specified to the same depth, or the read model is a name with no computation behind it.

### 3.4 The Projector — the executable form of a read model

The Decider formalizes `decide`; the **Projector** formalizes `evolve` (here `project`, since it folds events into a read model rather than into aggregate state):

```
project(state, event) -> state          -- the read-model fold
```

A read model's `projects` link (§3.2) names the events it is built from, but a name is **not a function**: it says *which* events feed the view, not *how*. For a simple create/read/update/delete view the fold is trivial (`EntityCreated → add a row`) and the naming nearly suffices — which is exactly why the gap hides. For a derived view — a report, an aggregate metric, a conformance verdict — the fold is a real computation, and the `projects` pointer derives nothing. The Projector is the missing artifact that makes the projection executable, **symmetric to the Decider in every respect**:

- its **signature is derived** from the event model — it folds exactly the events the read model `projects`, over exactly the entities those events `change`; only the fold *logic* is authored;
- it is **optional and proportional** — a one-line declaration for a CRUD view, a delegation to a declared rule-set for a derived view, and absent where a view is too trivial to model;
- it is **simulated before realisation** against flow-derived scenarios (a *given* of prior events, a *then* of expected read-model state), proving the projection **sound and complete before any code exists** — no read model claims a field no event it folds can supply, and every event that changes a projected entity is accounted for;
- after realisation, the realised projection must produce **identical** output to the Projector across the same scenarios (§6.3, behavioural conformance extends to projections);
- it becomes the **oracle** for its read model, exactly as the Decider is the oracle for its aggregate's behaviour.

Three conformance rules mirror the Decider's, keeping the authored fold from drifting:

- **No foreign events** — it may fold only events the read model `projects`.
- **Event coverage** — it must fold *every* event that changes an entity the read model projects, or the view is left under-specified for some event.
- **Output containment** — it may produce only fields the read model declares; it may not invent projected data.

> **Why this completes the model — and what it reveals.** Event modelling is `decide` **and** `evolve`; formalizing only the Decider captures half of it, and leaves the entire UI stack (§3.2.1, §4.5) standing on an unspecified floor — "a page's data *is* its projection" bottoms out in a name unless the projection computes. With the Projector, "derived from the projection" means derived from a *function*, and the UI seam's data check (§4.5) finally has something computable beneath it. But the Projector also exposes a boundary the framework must be honest about: a fold is mechanical only when its variability is **declared data over a generic interpreter**. When the fold delegates to a rule-set, the rule-set is that declared data and the projection stays mechanical. When it delegates to a genuinely novel algorithm, it does not — and that is the Polanyi floor (§3.5).

> **What goes on an aggregate vs. in a projection.** The Decider/Projector split answers a recurring modelling question: *which fields belong on an entity?* An aggregate carries **exactly the state its own Deciders need to evaluate their rejections and emit their events — its invariant-bearing state — and no more.** If an invariant must read a value at decision time (a refund cannot exceed the amount paid, so the Order must know its paid and refunded totals when it decides), that value is a field the aggregate `evolve`s, because the Decider only sees aggregate state, never a read model (the consistency-boundary rule, §3.3). Everything *else* a screen or report wants is a **Projector's** output — derived by folding, never stored on the aggregate. The same total may therefore appear twice for different reasons: as enforced aggregate state (because an invariant guards it) *and* as a projected field (because a screen shows it). The test is not "does anything display this?" but "does an invariant need it at decision time?" — that, and only that, makes it a field on the entity.

> **The reverse is a model-gap detector — state justification.** Read the rule the other way and it becomes an *iff*, and the reverse direction is the more useful one: **a field belongs on an aggregate if and only if some Decider reads it.** So an aggregate field that *no* Decider reads is unjustified — and that is a **finding**, not a stylistic quibble. It is one of two things, both worth surfacing:
>
> - **dead state** — the field is genuinely unused; remove it (aggregates carry only invariant-bearing state); or, more often,
> - **an unmodelled invariant** — the field is unread because the rule that *would* read it has not been modelled as a Decider rejection yet. A `credit_limit` sitting on a Customer aggregate that no `decide` consults is usually not junk; it is the trace of a business rule ("an order may not exceed the customer's credit limit") that lives in people's heads but not in the model.
>
> This makes unread aggregate state a **detector of where the behavioural model is incomplete** — the structural peer of the intent-reliance rate (under-specified UI, §3.2.1) and the data-divergence rate (stale structure, §3.1). Where those measure UI and data drift, **state justification** measures *behavioural-model* drift: orphaned state points at the Deciders you have not written. The check is mechanical — for every field an aggregate `evolve`s, is there a `decide` that reads it? — and it is the state-side analogue of the Decider's command-coverage and output-containment rules (§3.3): those constrain its command and event alphabets; state justification constrains its *state*.
>
> **The mirror holds for projections.** The same minimality runs on the read side: a projected field has no value unless a UI step or report consumes it. The seam already says one half — "no view needs a field no projection supplies" (§4.5); state justification supplies the other — *no projection should supply a field no view needs.* So state minimality is a **global** property of a conformant model: **every piece of state has a reader — aggregate state a Decider, projected state a consumer — or it is a finding.** Dead state is structurally detectable everywhere; unread *aggregate* state additionally flags a missing invariant.

> **And the Decider must decide — Decider justification.** State justification asks whether every field has a reading Decider; the mirror asks whether every Decider actually *decides*. A Decider must have **at least one reachable rejection** — it must enforce at least one invariant. A "Decider" that can *never* reject — that accepts every command and emits a fixed event regardless of state or input — is not deciding anything; it is a trivial command→event relabelling, and it is a **finding**. It is one of two things:
>
> - **trivial behaviour mis-modelled** — the command genuinely has no invariant (a free-form `add-note`), so it should have *no* Decider at all (the proportionality rule, §3.3: trivial create/read/update/delete gets none); or, more often,
> - **an unmodelled invariant** — the behaviour is not trivial and the rule that *should* reject something has not been written. A refund `decide` that accepts any amount unconditionally is missing `ORDER-REFUND-1`.
>
> Note what the missing thing is: a **rejection**, not *fields*. A "Decider with no fields" is **not** the degenerate case — a *creation* Decider (`decide(∅, cmd-place-order)`) reads no prior state yet is a real Decider, because it still rejects (an order cannot be placed from an empty cart) and produces the foundational event. The substance of a Decider is the *decision* — the reachable rejection — not the state it happens to read. This is the exact mirror of state justification, and the two fail toward the **same diagnosis from opposite sides**: orphaned state and toothless Deciders are both flags for an invariant that exists in the business but not yet in the model — a pincer on unmodelled rules.

### 3.5 Named-algorithm primitives — the Polanyi floor

Not every computation reduces to declared rules over a generic interpreter. Betweenness centrality, SHA-256, a YAML grammar, a geometric solver — these are **irreducible**: there is no declarative fold or rule-set that *is* the computation; the algorithm itself is the specification. The framework states the governing principle plainly:

> A computation is mechanically derivable from spec **exactly to the degree its variability is declared data over a generic interpreter.** Push a projection's logic into declared rules and it rejoins the mechanical region; what remains — genuinely novel algorithms — is the irreducible kernel.

The honest move for the kernel is to **name and bound it, not pretend it is derivable.** A **named-algorithm primitive** is a first-class What-side element specified by *reference*, never by derivation:

- **Reference** — the named algorithm or standard it implements (e.g. "Brandes' betweenness centrality"; "SHA-256, FIPS 180-4"; "YAML 1.2 grammar"). The name *is* the specification.
- **I/O contract** — its input and output types, expressed in the domain model's terms, so it composes with the rest of the graph.
- **Oracle** — reference input/output pairs (or a reference implementation to differential-test against) that pin the behaviour. This is the only check available, because there is no Decider or Projector to simulate against.

A named-algorithm primitive is explicitly **exempt from derivation and from behavioural simulation**, and is checked by **oracle conformance** (§6.3) — its realised output matches the reference algorithm's, across the oracle's pairs — rather than by behavioural conformance against a fold it does not have. This is the deliberate counterpart to the Decider/Projector: those say "here is the logic, derived and simulated"; a primitive says "here is the *name and the proof obligation*, referenced and test-pinned." A conformant instance may use primitives freely, but each must be *declared as such* — an undisclosed algorithm masquerading as a mechanical projection is the dishonesty the floor exists to prevent.

> **Where the boundary falls in practice.** A `rm-conformance-report` projection *looks* irreducible but is not: its engine (evaluate clauses, collect verdicts) is generic, and only the clause-set varies — so once the clauses are a declared rule-set, the Projector folds mechanically over them and the report leaves the kernel. Betweenness centrality genuinely stays: no rule-set *is* the algorithm. The discipline is to push everything that *can* become declared data into the Projector's rule-set, and to name-and-bound only what truly cannot — keeping the irreducible kernel as small as the problem honestly allows.

### 3.6 Quality demands — the non-functional, made checkable

Functional specification says *what the system does*; a **quality demand** (a non-functional requirement — performance, capacity, availability, security posture, data residency) says *how well, under what load, within what bound* it does it. The framework admits these, but on its own terms: **a quality demand is never prose.** "Must support 10,000 concurrent users" or "must be secure" is exactly the unverifiable wish the framework replaced everywhere else (the deleted accessibility prose of §3.2.3, the UI intent that became debt in §3.2.1). A demand that cannot be made checkable is *not yet specified* — it is debt, the same way an un-modelled intent is. Admitted into the spec, a quality demand is one of exactly **two kinds**, distinguished by *how it is checked*:

**Kind 1 — a runtime bound, measured continuously against telemetry.** Performance, capacity, throughput, latency, uptime, error rate — these are only meaningful against a *measurement*. "p99 latency ≤ 200 ms" means nothing until p99 latency is measured and compared to the bound. This is **structurally identical to data conformance** (§3.1): a declared shape (`p99_latency ≤ 200`) asserted against a measured source (production telemetry), continuously, where a breach is a finding. It is the data-conformance pattern pointed at *operational* data instead of *domain* data, and it inherits everything that pattern already has:

- a **runtime-bound conformance** check (§6.3), the operational sibling of data conformance, run continuously in production;
- the same **bidirectional reading** — a breach means either the system underperforms *or the bound was wrong* (the projected load was never realistic): the telemetry is a witness that can indict the demand, exactly as production data can indict the structure (§3.1);
- a **divergence signal** — the rate and margin by which measured reality misses its bounds, a standing operational-staleness metric peer to the data-divergence rate.

**Kind 2 — an architectural constraint, designed in and checked at build time.** Security posture, data residency ("personal data stays in the EU"), fault-tolerance topology, scalability headroom — these *cannot* be checked by watching production, because by the time a breach is measured the harm is done. They are **constraints on the realisation**, and they bind to the **How** (§4): a declared contract checked at build time against the architecture and deployment, not at runtime against telemetry. "Data resides in the EU" is verified against the deployment's declared regions (§4.2) before release; "no plaintext secrets at rest" is verified against the realised code and config. These are How-side seam checks, gated like any other (§6), never observed-after-the-fact.

> **The test that keeps demands honest.** For any quality demand, ask: *is it checked by measuring production, or by inspecting the realisation?* If production telemetry can confirm it, it is a **runtime bound** (Kind 1) — declare the bound and its measured source. If only inspecting the built system can confirm it, it is an **architectural constraint** (Kind 2) — declare it as a How-side contract. If *neither* can confirm it, it is not a quality demand yet — it is prose, and prose is debt. There is no third kind, because there is no third way to check something.

**Demands are located, not listed.** A quality demand attaches to the element it scopes, so it is derived and traceable rather than a floating registry:

- **System-wide demands attach to the system node** (§3.2.5): concurrent users, uptime, total throughput, data-residency region, security baseline — these are properties of *the system*, and the system node is their home (one of the reasons it is first-class).
- **Scoped demands attach to the event-model element they bound**: a latency budget on a *flow* ("checkout completes in ≤ 3 s"), a response bound on a *UI step*, a throughput floor on a *Decider* or *Projector*. Attaching the budget to the flow makes it derivable ("what is this flow's latency budget?") and gives the runtime-bound check a precise subject to measure.

This keeps quality demands inside the framework's spine rather than beside it: each is a declared bound or constraint, attached to a located element, checked either continuously against telemetry or at build time against the architecture — and if it can be neither declared nor located nor checked, it is not done being specified.

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

- **An application contract** — the invariant code-shaping decisions (language, layering, organization, persistence model). Stable across instances of an archetype. Where the What carries Deciders (§3.3), the application contract states that **decision logic is kept in a pure, isolable core** separate from input/output — the constraint that makes behaviour verifiable against its Decider.
- **An infrastructure/runtime contract** — the concrete runtime choices for an instance, including the system's **deployment identity** (production domain name, App Store / Play bundle identifiers, the chosen runtimes) — the concrete address of the system whose *identity and reach* the What declares (§3.2.5). May vary per deployment; once chosen, frozen. It must **satisfy** the application contract, and the satisfaction is recorded.
- **The seam between them is verified.** Where application and runtime are described separately, a verification must confirm they agree (configuration, identity/permissions, resources expected vs. provided). This seam is a required verification, because nothing else makes the two halves agree.

### 4.3 The repository layout model — what files are legal where

The application contract's structural half is made **verifiable** by a declarative, glob-based **repository layout model**: a machine-readable artifact that binds file patterns to rules about placement, presence, and prohibition, so a verification can check an actual repository against it and fail the build on violation. It is the executable form of the foundational "where does each file go, and why" decisions — the same rationale, now checkable rather than prose.

A layout rule is expressed with **glob patterns**, because globs are the language the filesystem and every CI tool already speak (the same "use the standard" discipline as §4.4 below — do not invent a path-matching DSL). The model has five rule kinds:

| Rule kind | Asserts | Fails when |
|---|---|---|
| **must-exist** | a matching file is required (with a cardinality) | the match is absent (or the wrong count) |
| **may-exist-here** | the legal placement(s) for a file type | the file appears somewhere not permitted |
| **must-co-exist** | required siblings — a set that must be whole (completeness) | the set is incomplete |
| **must-not-exist** | a forbidden file or pattern | the match is present |
| **no-orphans** | every file matches at least one allow rule | a file matches no declared rule |

**Two directions.** Most rules are *reactive* — they judge files that exist (placement, completeness, prohibition). The **must-exist** rule is *proactive* — it fires on the **absence** of a required file, asserting the tree contains the spine an archetype needs. Together, `must-exist` and `must-not-exist` let the contract state the expected shape of the tree in both directions: nothing missing, nothing forbidden.

**Cardinality on presence.** A `must-exist` rule must declare how many and in what scope, or it is ambiguous: `exactly 1` (a global singleton — zero *and* two both fail), `at least 1`, or — the most useful and the ergonomic default — `1 per <scope-glob>` (quantified over each match of a parent pattern, e.g. "every feature folder must contain a test file").

```yaml
layout:
  - id: apphost-required
    must_exist: "*.AppHost/Program.cs"
    cardinality: "exactly 1"
    rationale: "the composition root; the solution is not runnable without it"
    enforces: [explicit-composition-root]

  - id: slice-has-tests
    for_each: "src/*/Features/*/"          # scope
    must_exist: "{dir}/*Tests.cs"           # one per scope
    rationale: "a slice is structurally incomplete without its tests"
    enforces: [every-slice-tested]

  - id: contracts-isolation
    may_exist_here: "src/*.Contracts/**"
    rationale: "consumers depend on shape, not implementation"

  - id: no-secrets-in-source
    must_not_exist: "**/appsettings.*.secrets.json"
    rationale: "secrets never live in source"
    enforces: [secrets-out-of-repo]

  - id: no-orphans
    rule: "every file under src/ matches at least one allow rule"
```

#### Allowlist semantics (the strength choice)

The model is **allowlist by default**, anchored by the **no-orphans** rule: every file must match at least one declared allow rule, and a file matching none **fails**. A denylist ("these patterns are forbidden") only catches the violations you anticipated; the allowlist makes the *unanticipated* file the failure case, which is what lets a repository be *provably* in its archetype's shape rather than merely free of known sins. This is the same "by construction, not by vigilance" choice as the coherence bar (§6.1).

#### The two guards (required)

A layout model is high-leverage but fails badly if mis-used. Two guards are **normative**, not optional:

1. **Every rule cites the principle it protects.** Each rule — and *especially* every `must-not-exist` — carries an `enforces` link to the principle or foundational decision behind it. A prohibition with no principle behind it is a superstition; it must be removed, not kept "just in case." This is what keeps the prohibition set small and meaningful instead of a graveyard of past incidents, and it is what lets the layout check participate in the rationale trace (§5) and impact analysis (§9).

2. **Prefer the allowlist to a pile of denials.** With no-orphans in force, most prohibitions are already redundant — a stray file fails by matching no allow rule. Reserve explicit `must-not-exist` for cases where presence is *actively dangerous and deserves a named, specific error* (a committed secret; a layer-folder that a coarser allow rule would wave through; a permitted-looking file in a semantically wrong place). The allowlist handles "not permitted"; explicit prohibition handles "permitted-looking but specifically banned — and here is the clear reason why."

#### Scope discipline

Constrain the **architecturally meaningful skeleton** — where slices live, where contracts live, what makes a slice complete — and leave the *interior* of a slice relatively free. The allowlist applies at the level of "what kind of thing goes where," not "every individual file must be blessed." A model so granular that every legitimate new file needs a contract amendment will be disabled by the people it constrains; constrain the skeleton, not the cytoplasm.

#### Globs match paths, not meaning

The layout model checks the **shape of the tree**, deterministically and cheaply, with no code parsing — which is why it is the first gate to run. But a glob can confirm a file named `*Handler.cs` exists in the slice; it cannot confirm the file *contains* a handler. The layout model is therefore **necessary but not sufficient**: it is the cheap structural gate, layered *below* the content audits (domain- and behavioural-conformance), never a replacement for them. Cheap structural check first; expensive semantic check second.

#### Dual-read — it scaffolds and it verifies

Because the layout is declared as data, a scaffolding work unit reads the **same artifact** to know where to place what it generates and what spine it must lay down (`must-exist`), while the verifier reads it to demand placement and reject violations. One declaration drives creation and gates it — the same dual-read property the work-unit and acceptance schemas have.

### 4.4 Interface contracts — use the standards

For any interface or dependency that has an **industry-standard description format, the How uses that standard as the contract.** Bespoke description is permitted only where no standard exists. Reinventing a standard is non-conformant: it forfeits the standard's tooling and ecosystem.

| Surface | Reference standard |
|---|---|
| REST interface | OpenAPI |
| Async / event stream | AsyncAPI |
| RPC · message payloads | Protobuf / gRPC · JSON Schema / Avro |
| Cloud events | CloudEvents |
| Auth / identity | OIDC / OAuth2 metadata |

**Generated from the domain model.** Interface contracts are derived from the domain and event models, not hand-written, so the published surface cannot drift from the meaning. The standard document is the *surface*; the domain model remains the *meaning*; the derivation link is the traceability between them.

### 4.5 The screen-composition contract — use a design system, structured by Atomic Design

The event model carries **UI steps** typed against **Abstract Interaction Objects** (§3.2.2) — the AUI layer. The screen-composition contract is the **Concrete UI layer**: it specifies how each AIO is **reified** into a concrete control, and how those controls compose into screens. Exactly as §4.4 binds an interface to an industry standard rather than a bespoke description, the How specifies a screen by **binding it to a design system**, not by inventing a UI description language. A bespoke screen-description format is non-conformant where a design system exists: it forfeits the system's components, tokens, accessibility, and tooling. The design system's components are the **Concrete Interaction Objects (CIOs)** that AIOs reify into.

**Atomic Design is the normative compositional structure.** A conformant screen-composition contract describes every screen as a composition over the five Atomic Design levels, and nothing in a screen may exist outside them:

| Level | Is | Conformance role |
|---|---|---|
| **Atoms** | indivisible primitives (button, input, label, icon) | the design system's leaf vocabulary; a screen may not introduce an atom the system does not define |
| **Molecules** | small functional groups of atoms (a labelled field, a search box) | the smallest reusable unit a UI step references |
| **Organisms** | composite sections (a form, a nav bar, a results table) | typically where a read model's projection or a command's controls are bound |
| **Templates** | a page's layout skeleton, content-agnostic | the placement contract a page conforms to |
| **Pages** | a template filled with a specific flow's data and controls | the realised UI step — one page per UI step in the flow |

- **A page is the realised form of a UI step.** Each UI step in the event model corresponds to a page; the page `conforms_to` a template, composes organisms/molecules/atoms drawn **only** from the design system, and is bound to its step's projected data and issuable commands.
- **The design system is the closed vocabulary.** Just as the layout model is allowlist-by-default (§4.3), the component set is closed: a screen composed of a component the design system does not define fails, which is what makes a UI *provably* on-system rather than merely styled to look like it. New components are added to the design system (and earn their place, §4.1), never improvised inside a screen.
- **Tokens, not literals.** Colour, spacing, typography, and the like are referenced as design-system **tokens**; a screen carrying literal style values instead of tokens is non-conformant, for the same drift reason interface contracts may not be hand-written.

**Derived from the event model.** A screen's data and controls are not authored on the screen — they are derived from its **UI step** (§3.2.1): the fields a page may show are exactly those its read model `project`s (§3.2), the controls it may expose are exactly the commands valid at that step (the same commands a Decider `handles`, §3.3), and the step's *intent, emphasis, state meanings, and accessibility obligations (the WCAG 2.2 criteria it inherits and adds, §3.2.3)* are what the composition must satisfy. The design system is the *surface*; the UI step remains the *meaning*; the binding link is the traceability between them — the precise parallel to §4.4.

#### Reification — AIO × context → CIO

The bridge from the AUI to the Concrete UI is a declared set of **reification rules**, each mapping an Abstract Interaction Object, *in a given context of use* (§3.2.2), to a concrete control:

```
reify(AIO, context) -> CIO
```

This is where the phone-vs-tablet decision lives, and where it belongs — below the What, as a rule that *is itself a traceable artifact* rather than a choice buried in code:

```yaml
reification:
  - aio: single-select
    when: { form_factor: phone, options: many }
    cio: searchable-list           # a design-system component
    rationale: "a phone has no room for many side-by-side options"
  - aio: single-select
    when: { form_factor: tablet, options: few }
    cio: segmented-control
    rationale: "few options, ample width — direct choice beats a menu"
  - aio: trigger-action
    when: { emphasis: primary }
    cio: primary-button
```

- **One AIO, many CIOs, by context.** The same `single-select` in the What reifies to different controls per context; the What is unchanged, which is the entire reason the AIO layer exists. A conformant instance must provide a reification rule for every (AIO, context) pair its UI steps can encounter, or some screen is left unspecified for some device — the same coverage obligation a Decider has over its commands (§3.3).
- **An AIO may be declared unreifiable in an interaction class.** Some abstractions do not honestly survive a change of substrate: a `display-collection` of product *images* has no faithful TUI form, and some terminal-native interactions have no GUI analog. Rather than force an awkward reification or fail silently, a design system may declare a (AIO, class) pair **unreifiable** — a *recorded, deliberate* coverage gap with a rationale, not an omission. The seam verification (§6.3) then treats a UI step that uses an unreifiable AIO in a class the system targets as a finding *at authoring time* (the step cannot run in that class; either the step is out of scope for the class or the class is wrong for the product), exactly as it surfaces a missing reification — but a declared-unreifiable pair is never a silent pass. This is the same honesty as tagging a WCAG criterion *manual* rather than pretending it is machine-checkable (§3.2.3), or naming the Polanyi floor (§3.5) rather than pretending an algorithm is derivable: the framework records the boundary instead of papering over it.
- **Root navigation reifies like any AIO.** The `navigate` AIO at the application root (§3.2.4) — whose targets are the global destinations — is reified per context by the same rules: phone → a drawer/burger, tablet → a rail, desktop → a persistent sidebar. The application chrome a renderer draws is nothing more than this reified root-navigation; it is not separately authored content.
- **The CIO is always an on-system component.** A reification rule may only target a component the design system defines; it cannot invent one (the closed-vocabulary rule above). Reification chooses *among* the system's components; it never escapes it.
- **Rules carry rationale.** Each rule states why this control suits this AIO in this context — the UX reasoning that is otherwise lost. This is what makes the choice teachable and reviewable, and it is the `enforces`-style citation that lets the rule participate in the rationale trace (§5).

> **The seam is verified.** A screen sits on the What→How seam, like application↔runtime (§4.2). A verification (the seam kind, §6.3) confirms the two halves agree: every datum a page displays is `project`ed by a read model in its flow (no view needs a field no projection supplies — the same completeness the Decider simulation proves for behaviour, §3.3), every control maps to a command valid at that step (no button issues a command the step cannot accept), **every AIO referenced by a UI step has a reifying CIO for each declared context** (reification coverage), every state in each surfaced projection's declared state space (§3.2) is either given a meaning by the UI step and composed, or explicitly waived (state coverage — the UI analogue of command coverage), **every content key the step references resolves in the content store for each declared locale** (content coverage, §4.6), and the step's **accessibility obligations are discharged**: every *machine* WCAG criterion (§3.2.3) passes as a deterministic gate, and every *assisted* or *manual* criterion has a recorded attestation. The verdict reports the conformance level and its basis, never a bare pass. A complementary, *cheaper* structural check runs first and belongs with the other by-construction gates: **a UI step may reference only AIO-typed nodes; if it references a CIO, the build fails** — the type boundary that makes the What/How split for UI structural rather than advisory (§3.2.1). This is what keeps the UI from drifting from the behaviour it serves, and it is *required* wherever screens are specified, because nothing else makes the screen and the flow agree.

### 4.6 Content resolution — words live in a content store

The What carries content by **reference** (§3.2.1): a content key with a declared role, never a literal string. The How resolves those keys against a **content store** — the swappable provider of words, precisely parallel to how the design system is the swappable provider of components:

```
resolve(content_key, locale) -> string
```

This is the fourth instance of the framework's one UI move — *do not bake the concrete thing into the What; reference an abstraction the How resolves*: widgets become AIOs (§3.2.2), styles become tokens (§4.5), components become CIOs (§4.5), and now **copy becomes content references resolved against a store.** The payoff is identical each time: a keyed reference can be translated, swapped, and verified, where a literal cannot.

- **Locale is the content store's context dimension.** Just as reification is parameterised by context of use (`reify(AIO, context)`), content resolution is parameterised by **locale** (`resolve(key, locale)`). A content store declares the locales it covers; a localized application is one resolved against a store covering its target locales, with the *same* content keys — the What does not change per language, exactly as it does not change per device.
- **Coverage is the central obligation.** A content store must resolve every (content key, locale) pair the application's UI steps and shell reference, or some screen has missing words in some language — the same coverage obligation reification has over (AIO, context) and a Decider has over its commands. The seam verification checks it.
- **Roles let content be checked, not just present.** Because each key carries a role (heading, empty-message, error-message, …), the store's resolution can be checked for more than mere existence — an error-message role that resolves to empty, or a heading longer than its role admits, is catchable. The role is the What-side meaning; the string is the How-side value; the seam confirms the value satisfies the role.

> **The content store is to words what the design system is to components.** Both are swappable providers behind a conformance profile (the design system's is §11; the content store's is §12); both have a context dimension (context of use; locale); both are resolved at render time against a reference the What carries. A renderer assembles a screen by resolving its AIOs to components *and* its content keys to strings — two resolutions, one surface.

---

## 5. Work units and the rationale trace

A **work unit** is the smallest reproducible unit of realisation: a single bounded transformation that produces one artifact from a fixed, declared input. The framework's conformance requirements for a work unit:

- **Single-purpose and bounded.** One unit produces one artifact; its input (its context) is explicitly declared and frozen, so the same input yields the same output — the reproducibility guarantee.
- **References, never re-derives.** A unit reads the What concepts and the How principles/patterns it depends on by pointer; it does not re-decide them. Hard reasoning concentrates upstream, in the What and the How.
- **Emits a rationale trace.** Each unit's output carries a trace to the decisions that produced it — the domain concept (what), the flow (behaviour), the principle/pattern (why), the foundational decision (structure).

> **A work unit is an SPMC bundle — what its frozen input comprises.** The parent Two Pillars Specification Framework defines a unit of bounded specification as **SPMC** — four axes, *Schema, Prompt, Model, Context* (defined by the parent standard; §8). A work unit is this framework's instantiation of an SPMC bundle, and naming the four axes makes concrete what "the frozen input bundle" of §5.1 actually contains:
>
> - **Schema** — the *shape of the output* the unit must produce: the artifact's type and its versioned acceptance criteria (§6.1). This is what "produces one artifact" is checked against.
> - **Prompt** — the *transformation* the unit performs: the single bounded instruction that turns the input into the artifact. This is the unit's "single purpose."
> - **Model** — *what produces the artifact*: the capability the unit is intended for. This is exactly what an executor matches a unit to when it admits it (§5.1) — the unit declares the kind of producer it needs, the executor supplies it.
> - **Context** — the *frozen, resolved input* the unit reads: every What concept and How principle it depends on, referenced by pointer and resolved (§5.1's "by value, fully inlined"). This is the axis the reproducibility guarantee rests on — freeze the Context and the same input yields the same output.
>
> These are not new requirements; they are the names for structure the section already relies on. "Frozen input" is **Context**; "one artifact" is governed by **Schema**; "single bounded transformation" is **Prompt**; and the tier/capability an executor matches at the Build seam is **Model**. The emission contract (§5.1) freezes and emits exactly this bundle; the verdict that returns is attributable to it by the bundle's content hash.

> **The trace must be true.** A rationale trace that claims a principle the artifact violates is worse than no trace — it misleads. Therefore: every principle a unit claims to apply must be **enforced by a passing verification**, or the claim is retracted from the trace. Verifications gate which trace claims survive. The same checks that ensure correctness also keep the explanation honest.

### 5.1 The Build seam — the work-unit emission contract

A work unit is where the framework's specification *ends* and execution *begins*. Everything to the left — the What, the How, the decomposition into work units — is specification; turning a frozen work unit into a realised, verified artifact is **execution**. The framework specifies *the seam between them*, and nothing on the far side of it: **how** a unit is scheduled, admitted, batched, or run is the executor's concern, authored by whoever builds an executor, and deliberately out of scope here (the open/closed line, §1). What the framework fixes is the **contract that crosses the seam**, so that any executor honouring it can consume the framework's output, and either side can be rebuilt against this contract alone.

This is the framework's **third verified seam**, peer to application↔runtime (§4.2) and screen↔UI-step (§4.5): a boundary the framework *defines* but does not *cross*. It is the cut that lets the execution pillar be **pluggable** — the framework becomes a specification pillar with a published socket, not a tool with execution baked in.

**The seam is a data contract, not a call.** The two sides share no live surface — no store, no endpoint, no address. They share exactly two data shapes and two emit-points:

- **the emitted work unit** — what the specification side emits, outbound;
- **the verdict event** — what the execution side emits, inbound.

The specification side never invokes the executor; the executor never reaches back into the graph. A unit is frozen and emitted, and from that moment it is self-contained — **the freeze is the decoupling** (the same frozen-input principle a verification obeys, §6.2). The return path is an **event**, which is looser than either push or pull: push would name a consumer (the executor would have to know the specification side's address), pull would name a poller (the specification side would have to know the executor's location), an event names *neither* — the executor emits a verdict to a stream and is done, with no knowledge of who consumes it or whether anyone does. **The producer holding no knowledge of any consumer is the operational definition of decoupled.**

**The emitted work unit travels by value, with a hash identity.** The unit carries its frozen input bundle *in full* — the SPMC bundle of §5, every reference (decisions, criteria, context) resolved and inlined — so the executor needs nothing but the unit in hand and resolves nothing by calling back. Alongside the body it carries a **content hash of the frozen bundle**, which *is* the unit's identity. Carrying the hash even when the body travels by value is deliberate: it makes a later move to by-*reference* transport (the body in a content-addressed store, the hash in the unit) an **additive change, not a breaking one** — which is what allows a federation of executors to pull one frozen bundle without a contract change. By-value is correct for a single executor; the hash keeps the door open.

**The verdict returns as a self-describing event.** A consumer that never observed the emission can reconcile the verdict from the event alone: it echoes the unit's identity and lineage, carries the **bundle hash it ran against** (closing the loop — every verdict is attributable to an immutable input, so provenance is reproducible), and states a **declared verdict** from a fixed vocabulary (accepted / rejected / escalate, §6.2) rather than a boolean in an exit code. The executor emits and is done — it does not wait, retry, or know who listens. **Temporal decoupling** follows for free: a verdict emitted while no one is listening is correctly reconciled whenever a consumer next reads the stream.

**What the framework fixes, and what it leaves open.** The framework fixes: the two schemas, the by-value-plus-hash identity, the verdict-as-event return, and the freeze-before-emit obligation (a unit that would require a callback mid-execution is not frozen and must not be emitted). The framework leaves entirely to the executor: scheduling, admission, batching, how the realisation is produced, and the transport of the event stream (log, bus, or file — the contract requires only that emit is fire-and-forget and events are self-describing). An executor is conformant to the seam if, given a frozen unit, it reaches a declared verdict with no further input, emits it as a self-describing event, and holds no knowledge of any consumer — nothing more is required, and nothing about its internals is specified.

**The two schemas are concrete and machine-checkable.** So that two independently built executors actually interoperate — the whole point of specifying the seam — the framework provides a **normative schema for each message** (JSON Schema is the reference encoding, paralleling the manifest schemas of §11–§12): [`schema/json/build-seam/workunit.schema.json`](../schema/json/build-seam/workunit.schema.json) and [`schema/json/build-seam/verdictevent.schema.json`](../schema/json/build-seam/verdictevent.schema.json), with a worked pair carried alongside them. Each schema has two parts:

- a **universal envelope** — the named, typed fields *every* conforming executor must understand: on the unit, `unit_ref`, `bundle_hash`, the SPMC `bundle` (with its four axes, §5), and `acceptance_class`; on the verdict, `event_id`, `emitted_at`, the echoed `unit_ref`/`bundle_hash`, the `verdict` enum (pinned to the §6.2 vocabulary), and an optional `next_consequence`. This envelope is the interoperable surface — write to it and any conforming consumer can reconcile your verdict, any conforming executor can run your unit.
- **exactly one opaque extension slot** — `executor_extension`, where a particular execution framework places structure the framework neither reads nor mandates (capability tiers, escalation-ladder position, sealed interior dependency graphs, per-cell results). The slot's existence is what keeps the universal schema closed while leaving executors unconstrained: a tiered, DAG-walking executor and a single-shot executor emit the *same envelope* and differ only inside the slot. Strip the slot and the messages are still complete and reconcilable — proof the contract lives in the envelope, not the extension.

This is the by-value-plus-hash discipline applied to the schema's *shape*: extensible by construction, additive not breaking. An executor never needs the envelope reopened to carry its own model, exactly as the hash means the bundle never needs the transport reopened to move by reference.

> **Why the work unit is the right cut.** The seam falls here because **both pillars already name the work unit as their edge** — it is the smallest thing specification bottoms out into (§5) *and* the unit execution consumes. That coincidence of boundary is structural, not a convenience: the seam goes where both sides already end, which is why a contract placed here binds neither side's internals. Build, in the everyday sense, runs *across* this seam — the specification side emits a unit, an executor returns a verdict — and the framework's responsibility ends at the emit.

---

## 6. Verification — the conformance bar

The framework is built on verification, but it deliberately ships **no verifications**. It defines what verification must *do*, and how strong it must be; the actual checks are the adopter's (and, typically, their most valuable proprietary asset).

### 6.1 Requirements on verification

- **No accepted output without a verdict.** Every artifact kind has versioned acceptance criteria; an artifact is accepted only when its verifications pass.
- **The coherence bar.** When realisation is split across work units, a verification must confirm the parts agree **at least as well as a single unsplit author would achieve from shared context.** If a split makes coherence weaker than the unsplit baseline, the split is not worth it. This is the framework's central quality guarantee.
- **Verifications are deterministic gates.** Conformance is established **by construction, not by instruction** — a check that fails the build, not a request to be careful.
- **Every verification names what it protects.** A verification cites the principle, contract, or model element it enforces — which is what makes the rationale trace ([§5](#5-work-units-and-the-rationale-trace)) honest and impact analysis possible.

### 6.2 How a verification runs — the anatomy of a check

The framework supplies no checks, but it does fix the **mechanism** every check obeys, so that a verdict means the same thing across instances and across kinds. A conformant verification is a function of declared inputs to a verdict, with no hidden state:

```
verify(artifact, oracle, criteria) -> Verdict { pass | fail, findings[] }
```

- **Inputs are frozen and declared.** A verification reads exactly three things: the **artifact** under test, the **oracle** it is judged against, and the **criteria** that define conformance. Each is a named element of the graph, pinned by version — nothing is fetched mid-run, exactly as a work unit's context is frozen (§5). The same inputs always produce the same verdict; this is what makes a verification a *deterministic gate* (§6.1) rather than an opinion.

- **The oracle is derived, never authored in the check.** What a verification compares against comes from the spec, not from the check's own body: behavioural conformance is judged against the **Decider's** flow-derived scenarios (§3.3); domain conformance against the **domain model**; layout conformance against the **repository layout model** (§4.3); contract and seam conformance against the **contracts** (§4.2). A check that embeds its own expected answers instead of deriving them from the model is non-conformant — it can pass while the model and the code disagree.

- **Criteria are explicit and versioned.** Each artifact kind carries its acceptance criteria as named, individually-evaluable conditions. A verification evaluates *each* criterion and records a **finding** per criterion — `pass`, `fail`, or `not-applicable` with the reason. A bare boolean is not a conformant verdict; the per-criterion findings are what make a failure diagnosable (which criterion, against which oracle element) and what let the rationale trace retract exactly the claims that failed (§5).

- **The verdict is the conjunction, and it gates.** An artifact is **accepted only if every applicable criterion passes**; one failing finding fails the verdict, and a failed verdict stops the build (§6.1). There is no partial acceptance and no override-by-assertion — conformance is established by the check passing, not by anyone declaring it passed.

- **Each verification names what it protects.** Every check cites the principle, contract, or model element it enforces (§6.1). This citation is not documentation: it is the edge (`enforces`, §9) that links a green verdict to the trace claim it justifies and to the impact-analysis graph. A check that protects nothing nameable should not exist (the earn-their-place rule, §4.1).

> **Why the oracle-derivation rule matters most.** The single property that separates this from "we have tests" is that **the thing a check compares against is computed from the spec, not written into the check.** That is what makes a passing verdict mean "the realisation computes what the model says," and it is why behavioural conformance can reuse the *same* scenarios the Decider was simulated against before any code existed (§3.3) — the oracle is authored once, in the What, and consumed twice.

### 6.3 The required verification kinds

A conformant instance must have verifications covering, at minimum:

| Kind | Confirms |
|---|---|
| **Layout conformance** | the file tree matches the declared repository layout model (§4.3) — the cheapest gate, run first |
| **Behavioural simulation** | a Decider (§3.3) or Projector (§3.4), simulated against flow-derived scenarios, is sound and complete — run *before* realisation, when defects are cheapest |
| **Pattern conformance** | every behavioural slice is a well-formed instance of one of the four patterns (§3.2.0): a Command pattern's command traces to a Trigger and produces ≥1 event; a View reads only declared events; an Automation's trigger reads a View and carries no business logic; a Translation reads from exactly one source system (write side may fan out). Malformed shapes — an event with no producing command, a View needing an unrecorded field, a cross-system read from two sources — are findings |
| **Journey conformance** | every crossing in a journey (§3.0.1) is a Translation (§3.2.0) — a journey that hops between systems by any other means (a direct reference to another system's screen, a shared table, an assumed call) is a finding. Confirms the journey composes only flows and Translations that exist and adds no new coupling; the cross-system path is therefore traversable for value-anchoring and blast-radius without the systems depending on each other's internals |
| **Internal coherence** | the parts of one work unit's output agree with each other |
| **Contract conformance** | realised code obeys the How's contracts |
| **Seam** | separately-described parts agree — e.g. application vs. runtime (§4.2), or a screen vs. its UI step: every datum shown is projected, every control maps to a valid command, every referenced AIO has a reifying CIO per context, and a UI step references only AIO-typed nodes (§3.2.2, §4.5) |
| **Domain conformance** | realised code matches the domain model; no structural drift |
| **Data conformance** | production data validates against the domain model's declared shapes (§3.1) — the structure checked against real data as an oracle. Unlike the other (build-time) kinds it also runs **continuously in production**, and its failures read **both ways**: data wrong *or* spec stale (the data-divergence rate, §3.1) |
| **State justification** | every field an aggregate evolves is read by some Decider, and every projected field is consumed by a UI step or report (§3.4); an unread field is a finding — dead state, or (for aggregates) an unmodelled invariant. Its mirror, **Decider justification**, fails any Decider with no reachable rejection (trivial behaviour mis-modelled, or a missing invariant). Both are model-gap detectors, peers to the intent-reliance and data-divergence signals |
| **Runtime-bound conformance** | a quality demand of the runtime kind (§3.6) — latency, throughput, capacity, uptime — holds when measured against production telemetry. The operational sibling of data conformance: continuous, with the same bidirectional reading (a breach means the system underperforms *or* the bound was unrealistic) and a divergence signal. Architectural quality demands (§3.6, Kind 2) are instead checked at **build time** as How-side seam contracts (residency, security posture), never observed after the fact |
| **Behavioural conformance** | realised behaviour matches the event model; where a Decider (§3.3) or Projector (§3.4) exists, the realised behaviour or projection produces identical outputs to it across the same scenarios |
| **Oracle conformance** | a named-algorithm primitive (§3.5) produces outputs matching its referenced algorithm/standard across its declared oracle pairs — the only check available where there is no Decider or Projector to simulate against |

*The framework specifies these kinds; the adopter supplies the checks. The content of those checks is out of scope by design — see the open/closed line in [§1](#1-purpose-and-scope).*

---

## 7. Delivery — bringing the What to a verifiable 'done'

Because the What is a graph, delivery is **partitioning that graph into shippable slices**, and "done" is a **verifiable predicate** rather than a judgement.

### 7.1 Units of delivery

- **A feature** is a **reference to a slice of the event model** — one or more behavioural flows — and is the smallest independently valuable and verifiable such slice (typically a single flow). A feature is identified by the flows it includes; it does **not** separately enumerate domain concepts.
- **A release** is a chosen, coherent set of features that ship together.

Both are **subgraphs of the What**, not free-floating tickets.

> **The domain footprint is derived, not declared.** Because the event model is built from domain-typed primitives (§3.2), a feature's set of concepts is *computed* from its flow slice by following the model's links: the entities its events `change`, the aggregates its commands `target`, the entities its read models `project` (§9). You never maintain a feature's concept list by hand — it is a graph traversal of the slice, and it cannot drift from the behaviour it serves. Behaviour is primary because value is delivered by a flow completing, not by a concept existing; the concepts are pulled in precisely because the flows need them.

### 7.2 'Done' as a predicate

```
footprint(f)   := the concepts reachable from f's flow slice —
                  entities its events `change`, aggregates its commands
                  `target`, entities its read models `project`   (derived, §3.2/§9)

feature_done(f) := every flow in f is realised & passes behavioural conformance
               and every concept in footprint(f) is realised & passes domain conformance
               and every verification citing an f-element is green
               and f's agreed acceptance criteria pass

release_done(r) := all member features done
               and the cut is closed: the transitive closure of every included
                  element's dependencies lies inside r — nothing included
                  depends on anything excluded
```

- **A feature is its flows; its concepts are derived.** The domain clause ranges over `footprint(f)` — computed from the flow slice — not a separately authored concept list. Behavioural conformance leads because the flow is the unit of value; domain conformance follows over exactly the concepts those flows reach.

- **Build order is read from the graph.** Dependency links give the valid orderings (topological); which valid ordering to take is a value/risk judgement.
- **Progress is computed, not estimated.** It is the fraction of in-scope elements that pass their verifications.
- **Done is exactly as honest as the verifications are strong** — delivery inherits the verification layer's credibility.

### 7.3 Versions and direction — where the product is going

Everything to this point describes the system **as it is**. But a specification also has a *time axis*: a product is moving *toward* something, and the framework should be able to say what, and how far away it is — without that answer being a roadmap document that drifts. It does this with three connected ideas, each derived rather than asserted.

**The What and the How carry independent semantic versions.** The *specified system* is versioned (distinct from the version of this specification *document*). Because one What can be realised by many Hows (§2, §4), the two version on **independent axes that reference each other**:

- a **What-version** advances when the *meaning* changes: **major** = behaviour added or changed (a new or altered Decider, a changed invariant — something a conformant realisation must now do differently); **minor** = additive behaviour that breaks nothing (a new flow or feature slice); **patch** = clarification that changes no behaviour (a renamed concept, a tightened wording).
- a **How-version** advances when the *realisation* changes: **major** = an architectural change (a new layout model, a swapped substrate); **minor** / **patch** = the *same What* realised differently (a refactor, a dependency bump, a perf fix). A How **declares which What-version it realises** (`realises What 2.1`), so a How can iterate — 2.1.0 → 2.1.1 → 2.1.2 — under a fixed What, and the cross-reference records exactly which meaning each realisation serves.

Crucially, each bump is **derivable from what changed in the graph**, not a human's guess: the verification layer already knows whether a change altered behaviour (a Decider/invariant diff), added a slice (a new subgraph), or only touched realisation (How-side only). So "is this a major or a minor?" is answered by *what the diff touched*, the same way `feature_done` is computed rather than declared. The independence is the §4.2 swappability made temporal: the How versions beneath a stable What exactly as a runtime satisfies a stable application contract.

**A target version is a declared future partition of the graph.** A goal is not roadmap prose ("we plan to improve checkout") — it is a **named set of feature-slices** (§7.1) that constitutes a future What-version, *some of which do not exist yet*. "What 2.0 = { refund-flow, loyalty-slice, multi-currency-slice }" is a declared partition, exactly like a release partition (§7.2) except that membership may include slices not yet realised. The discipline is identical to every other part of the framework: if a goal cannot be written as a named set of slices, it is **not yet a specified goal** — it is prose, and prose is debt. A target version is to direction what an invariant is to correctness: the checkable form of an intention.

**Direction is the computed gap.** Because a target version is a graph partition and `feature_done` is already a computed predicate (§7.2), the distance to a version is **a query, not a status report**: the unrealised slices in the target's partition.

```
distance(target) = { slice ∈ target.partition : not feature_done(slice) }
```

Progress toward the goal is `1 − |distance| / |partition|`, computed continuously from the same verifications that gate everything else. "Are we there yet?" stops being a meeting and becomes a traversal: which slices of What 2.0 are not yet done, and — via each slice's footprint (§7.1) — what remains to build. The roadmap is a **derived view of the graph against its declared targets**, never a separately maintained artifact that can disagree with reality. This is the §1 reproducibility→measurement chain applied to *teleology*: the product's direction is measured, not narrated, and the gap closes visibly as slices pass their verifications.

> **Why this is not a roadmap document.** A roadmap drifts because it is maintained apart from the thing it describes. Here, direction *is* the graph: a target version names slices that already have (or will have) real footprints, `done` is computed from real verifications, and the gap updates itself as work lands. You cannot have a stale roadmap because there is no roadmap — there is a query over declared targets. Versioning records where the product *has been*; the target partition records where it is *going*; the computed gap is *how far*.

### 7.4 Model, not practice

This section defines the delivery **model** — the partition and the predicate. It deliberately does **not** define delivery **practice** — cadence, ceremonies, who sequences work, release rhythm. Practice is organization-specific; two teams may run the same model very differently. Practice is out of scope.

> **A note for the parent standard.** The Two Pillars defines a verdict on an individual output. `release_done` generalises it to a verdict over a **composition** — all members pass, and the composition is closed (no dangling dependency). This composition-verdict is offered upward as a candidate refinement to the Two Pillars verification concept.

---

## 8. Conformance to the Two Pillars

This framework is a conformant instantiation of the Two Pillars Specification Framework. Each construct maps to a named Two Pillars concept; an instance is Two-Pillars-conformant when it satisfies the mapped requirements.

| Two Pillars concept | This framework's construct | Section |
|---|---|---|
| **What specification** | Product (owns domains + systems) → Domain(s) (structure + reference data + behaviour: events, Deciders, Projectors) + System(s) (surfaces referencing domains: page graph, flows, UI steps typed by AIOs + context-of-use + WCAG 2.2 criteria, target platforms) + named-algorithm primitives (the Polanyi floor) + quality demands (runtime bounds + architectural constraints) | [§3](#3-the-what--structure-and-behaviour) |
| **How specification** | Decisions/principles/patterns + contracts (incl. repository layout model) + interface standards + screen-composition contract (reification + content resolution) | [§4](#4-the-how--realising-the-what) |
| **SPMC (Schema, Prompt, Model, Context)** | Work unit: one bounded transformation, frozen input, one artifact — its frozen bundle *is* an SPMC bundle, with the four axes named in this framework's terms (§5). SPMC itself is defined by the parent Two Pillars framework; this row is the mapping, not the definition. | [§5](#5-work-units-and-the-rationale-trace) |
| **Derivation contract** | The typed links of [§9](#9-encoding-and-the-derivation-contract) (e.g. `derived_from`, `conforms_to`, `applies`, `realizes`, `enforces`) | [§9](#9-encoding-and-the-derivation-contract) |
| **Verification (criteria → judge → verdict)** | Verification — the required kinds and the coherence bar | [§6](#6-verification--the-conformance-bar) |
| **Verdict (extended to a composition)** | `release_done` — a verdict over a composition | [§7](#7-delivery--bringing-the-what-to-a-verifiable-done) |
| **The seam between the pillars** | The **Build seam** (§5.1) — the work-unit emission contract: a by-value unit with a hash identity out, a self-describing verdict event in, the producer holding no knowledge of any consumer. This is the Two Pillars boundary made concrete: the specification pillar (this framework) emits, an execution pillar consumes, and either is swappable without disturbing the other so long as both honour the seam. | [§5.1](#51-the-build-seam--the-work-unit-emission-contract) |

### 8.1 Conformance levels

- **Level 1 — Described.** A conformant What exists as a machine-readable graph: one or more **systems** (each with a name, kind, purpose, and target platform contexts, §3.2.5), a **domain model** with declared bounded contexts and mappings including any **reference data** the behaviour depends on (§3.1), and an **event model**. Where behaviour is interesting, a Decider (§3.3) makes it executable and is simulated sound and complete before any realisation; where a read model's projection is non-trivial, a Projector (§3.4) makes the fold executable to the same depth; genuinely irreducible computations are declared as named-algorithm primitives (§3.5), specified by reference and pinned by an oracle. Where a flow has screens, its UI steps (§3.2.1) declare their intent and derive their data and actions from the model, typed against Abstract Interaction Objects (§3.2.2).
- **Level 2 — Realised.** A conformant How exists (including a repository layout model); work units reference the What and How by pointer; interface contracts use standards and are generated from the domain model; where the event model has UI steps, screens are specified as design-system compositions (Atomic Design) with reification rules mapping each AIO, per context of use, to an on-system component.
- **Level 3 — Verified.** Verifications of all required kinds ([§6.3](#63-the-required-verification-kinds)) exist — including layout conformance — meet the coherence bar, gate acceptance, and back the rationale trace.
- **Level 4 — Delivered.** Features and releases are graph partitions; "done" is computed by predicate; progress is the fraction passing verification.

*Levels are cumulative. A claim of conformance states the highest level satisfied.*

> **One capability is adoptable alone: data conformance.** The levels above describe building a system *from* the framework. But **data conformance** (§3.1, §6.3) can be adopted by an existing system that uses none of the rest — no Deciders, no Projectors, no UI model, no work units. A team with only a domain structure and a production database can declare the shapes their data should satisfy and immediately get a continuous conformance signal — because every system already has data, and almost none can answer "is it valid against what we believe should be true?" This makes data conformance the lowest-commitment, highest-immediate-value **entry point** into the framework: a team starts here because it pays off on day one, and the rest of the model becomes the natural next step once the structure is declared. The Data Conformance Profile (§13) states this minimal adoption.

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
| `changes` / `projects` / `has_state` | this event changes this entity / this read model projects these / this projection can occupy this state (§3.2) |
| `triggers` / `triggered_by` / `watches` / `translates_from` | this trigger issues this command / this command is initiated by this trigger (source: user / external / automated) / this automated trigger watches this View / this Translation trigger reads events from this source system (§3.2.0) |
| `reference_data_for` / `conforms_to_shape` | this declared instance data is constitutive reference data for this concept / this production dataset is validated against this shape as an oracle (§3.1) |
| `decides_for` / `handles` / `emits_event` / `reads` / `rejects` | this Decider governs this aggregate / accepts these commands / may produce these events / reads this field to evaluate a rejection or emit an event / can reject on this invariant — a Decider with no `rejects` edge fails Decider justification (§3.3, §3.4) |
| `projects_for` / `folds` | this Projector builds this read model / folds these events into it (§3.4) |
| `specified_by_reference` / `pinned_by_oracle` | this named-algorithm primitive implements this named algorithm/standard / is checked against these reference I/O pairs (§3.5) |
| `surfaces` / `offers` / `transitions_to` | this UI step surfaces this projection / offers these commands / leads to this next page on an action or event — a `navigate` edge in the page graph (§3.2.1, §3.2.4) |
| `navigates_from_root` / `in_flow` | this page is a global destination (edge from the application root) / this page belongs to this named flow subgraph (§3.2.4) |
| `owns_domain` / `owns_system` / `references_domain` | this product owns this domain / this product owns this system / this system references (uses) this whole domain (§3.0, §3.2.5) |
| `journey_of` / `composes_flow` / `crosses_via` | this journey belongs to this product / this journey composes this single-system flow / this journey crosses between two flows via this Translation (§3.0.1) |
| `system_of` / `targets_platform` / `targets_class` / `roots_at` | this flow/page graph belongs to this system / this system targets this platform context / this system targets this interaction class (GUI/TUI, §3.2.2) / this system's page graph roots at this node (§3.2.5) |
| `bounded_by` / `measured_by` / `constrains` | this element (system, flow, step, Decider) carries this quality demand / this runtime bound is measured against this telemetry source / this architectural constraint binds this How-side contract (§3.6) |
| `versioned_as` / `realises_version` / `targets_version` / `in_target` | this What or How carries this semantic version / this How realises this What-version / this named target version is the product's goal / this feature-slice is a member of this target version's partition (§7.3) |
| `typed_as` / `reifies` / `in_context` / `unreifiable_in` | this UI-step interaction is typed as this AIO / this CIO reifies this AIO / this reification holds in this context of use / this AIO is declared unreifiable in this interaction class — a recorded coverage gap (§3.2.2, §4.5) |
| `must_satisfy` / `attests` | this AIO or UI step must satisfy this WCAG criterion / this dated, attributed attestation records that a non-machine criterion was evaluated and met (§3.2.3) |
| `realizes_step` / `composes` / `binds` | this page realises this UI step / composes these design-system components / binds this control to this command or this field to this projection (§4.5) |
| `references_content` / `resolves` / `in_locale` | this UI step or shell references this content key (with a role) / this content store resolves this key to a string / this resolution holds in this locale (§3.2.1, §4.6) |

These links are what make the framework **queryable**: impact analysis ("what depends on X?"), onboarding traces ("why is this shaped this way?"), and the verification-to-principle linkage all fall out of graph queries rather than hand-maintained documents.

---

## 10. Conformance rules (normative summary)

1. A product is described as three models — What, How, Delivery — in one machine-readable graph.
2. The What is organised by a three-level ownership hierarchy: a **product** (§3.0) owns **one or more domains** and **one or more systems**, and a **system references whole domains** rather than owning them — so domain concepts are shared across systems by being owned one level up. The dividing line is that **the domain owns meaning and rules** (entities, structure, data, invariants, *and* behaviour — events, Deciders, Projectors) while **a system owns surface and journey** (its page graph, flows, UI steps, target platform contexts §3.2.2 and interaction classes GUI/TUI); the test is *if two systems would disagree it is system-level, if they must agree it is domain-level*, so behaviour (which must agree) lives in the domain and is identical across every system referencing it. A flow arranges domain behaviour into a journey, referencing the domain's commands/events rather than containing them. Multiple systems over one domain (a customer app + an admin web) thus share the same Deciders, invariants, and events, differing only in surface. Product-wide concerns (direction §7.3, product-level quality demands) sit on the product; per-system ones on the system. A **journey** (§3.0.1) is a product-level, derived composition of single-system flows linked at crossings, where **every crossing is a Translation** (§3.2.0) or it is a finding (journey conformance, §6.3); it references flows and Translations that exist and owns nothing, so it gives end-to-end cross-system visibility (value-anchoring, blast-radius) without re-coupling the systems. System identity is What; concrete deployment identity (domain name, bundle id, runtime) is How (§4.2).
3. The domain model has two sides: **structure** (bounded contexts with explicit cross-context mappings — never one flat model; entities, relations, value objects, invariants in a constraint language) and **data**. Constitutive **reference data** (valid shipping methods, tax categories, taxonomies the behaviour depends on) is part of the What; **production data** is not specification but serves as the **oracle** the structure is validated against (data conformance, §6.3) — a structure no real data satisfies is suspect. The test: if changing a datum changes what the system means, it is reference data; if it is just more rows, it is production data. Data-conformance shapes are authored once but **asserted continuously** in production, and their failures read **both ways** (data defect *or* spec drift); the **data-divergence rate** is the resulting first-class signal of spec staleness, the over-confidence counterpart to the intent-reliance rate. Data conformance is adoptable standalone as the framework's minimal entry point (§13). **Quality demands** (non-functional requirements, §3.6) are admitted only as checkable artifacts, never prose, in exactly two kinds: a **runtime bound** measured continuously against telemetry (the data-conformance pattern over operational data — latency, capacity, uptime; runtime-bound conformance, §6.3) or an **architectural constraint** checked at build time as a How-side contract (residency, security posture). Each attaches to the element it scopes — system-wide on the system node (§3.2.5), scoped on the flow/step/Decider it bounds — and a demand that can be neither measured nor inspected is not yet specified.
4. The event model is built from four **building blocks** — Trigger (user / external / automated), Command, Event, View — wired into four named, checkable **patterns**: Command (`Trigger→Command→Event(s)`), View (`Event(s)→View`), Automation (`Event(s)→View→automated Trigger→Command→Event(s)`, the normative way to model self-acting behaviour, the trigger carrying no business logic), and Translation (the normative way sibling systems (§3.2.5) communicate — an automated trigger reads a View over **one** source system and issues a Command into a target, write side may fan out) (§3.2.0). Every event changes a real entity; a read model declares (or makes inferable) its state space (`present` plus any of `loading`/`empty`/`failed` it can exhibit); depth is proportional to behavioural complexity. A **UI step** is a flow primitive whose interactions are **typed against Abstract Interaction Objects** (§3.2.2) — a closed-core, extensible vocabulary of context-independent interaction kinds — never against concrete controls; this type boundary makes the What/How UI split structural, not advisory. Its buildable core is derived from the model (the projection it surfaces, the commands valid at it, its transitions, with input AIOs deriving their fields and validation from the relevant command's payload via the domain model); its modelled meaning — emphasis, the meaning of each state in the surfaced projection's declared **state space** (constrained to states the projection has and covering every one, or waived with reason — the UI analogue of command coverage), **accessibility obligations** (WCAG 2.2 success criteria, ingested as entities, inherited from its AIOs and extended per step; each tagged machine/assisted/manual, §3.2.3), and **content references** (standing authored words — headings, body, empty/error prose, help, legal — carried by content key with a declared role, never as literals, resolved by the How against a content store, §4.6) — is checkable specification. Its **intent** is *not* modelled specification but the marked residue of what the model does not yet determine: it is treated as specification debt, every use of it to settle a realisation choice must be promoted into a modelled element (AIO, context, reification rule, or a meaning declaration), and the intent-reliance rate is a measure of UI under-specification. The relevant **contexts of use** (form factor, modality, …) are declared as What-side elements, gated by an **interaction class** (GUI / TUI) that determines which sub-dimensions apply — GUI brings form factor and pointer/touch; TUI brings terminal capabilities (grid, colour depth, glyph set, keys) and rules form factor out (§3.2.2). Navigation is **one page graph** (§3.2.4): pages are nodes, `navigate` transitions are edges, a flow is a named connected subgraph with an entry page, and a declared **application root** whose out-edges are the global destinations the primary navigation renders; "top-level" is derived (an inbound edge from the root), not a hand-applied tag.
5. Where behaviour is interesting, a **Decider** makes it executable: its signature is derived from the event model (it handles exactly the commands targeting its aggregate, emits only events those commands sanction, evolves from the events that change it, rejects via the aggregate's invariants), and it is simulated sound and complete before realisation. Trivial behaviour needs no Decider. Symmetrically, where a read model's projection is non-trivial, a **Projector** makes the fold executable to the same depth — its signature derived (it folds exactly the events the read model projects, over the entities they change), only its fold logic authored, simulated sound and complete and serving as the projection's oracle; the `projects` link alone is a name, not a function. A computation is mechanically derivable only to the degree its variability is declared data over a generic interpreter; a genuinely irreducible computation (a named algorithm or standard) is declared as a **named-algorithm primitive** specified by reference plus an I/O contract plus an oracle, is exempt from derivation and simulation, and is checked by oracle conformance — never passed off as a mechanical projection. An aggregate carries **exactly its invariant-bearing state**: a field belongs on an aggregate **iff** some Decider reads it, so an unread aggregate field is a finding — dead state, or an unmodelled invariant (**state justification**, §6.3); the mirror holds for projected fields (consumed by a UI step or report, or a finding). Conversely, every Decider must **decide** — it must have at least one reachable rejection (enforce ≥1 invariant); a Decider that can never reject is a finding (trivial behaviour that should have no Decider, or a missing invariant). A Decider that reads no prior state is still valid if it rejects (a creation Decider); the substance is the rejection, not the fields.
6. The How captures decisions/principles/patterns (declared once, referenced by pointer, each decision carrying rationale), contracts (stated checkably, including that decision logic is kept in a pure, isolable core), interface contracts (industry standards, generated from the domain model), and — wherever the event model has UI steps — a **screen-composition contract** that binds each screen to a design system structured by Atomic Design (atoms → molecules → organisms → templates → pages), with the component set closed and styling via tokens not literals. Each page's data and controls are derived from its UI step (projected fields, valid commands), and each AIO the step references is **reified** to an on-system concrete control by a declared `reify(AIO, context) → CIO` rule that carries rationale and must cover every (AIO, context) pair its steps can encounter. The whole is checked by the seam verification. **Accessibility** is specified not as prose but as **WCAG 2.2 success criteria ingested as entities** (principle → guideline → criterion → level), each tagged by verification type: *machine* criteria are deterministic gates, *assisted* and *manual* criteria are discharged by recorded, attributed attestations entering through a frozen boundary. Obligations are inherited from a step's AIOs and extended per step; the accessibility verdict reports a conformance level and its basis, never a bare pass. **Content** (authored words) is carried by reference, not literal: the How resolves each content key against a **content store** (§4.6) parameterised by **locale** (`resolve(key, locale) → string`), and the store must cover every (key, locale) pair the application references.
7. The How includes a glob-based **repository layout model** stating which files must exist (with cardinality), may exist where, must co-exist, and must not exist, with **allowlist semantics** (every file matches an allow rule or fails). Two guards are normative: every rule — especially every prohibition — cites the principle it enforces, and explicit prohibitions are reserved for actively-dangerous cases, the allowlist handling the rest. The layout model is dual-read (it scaffolds and it verifies) and checks tree shape only, layered below the content audits.
8. A work unit is single-purpose with frozen input, references the What/How by pointer, and emits a rationale trace. The work unit is also the **Build seam** (§5.1): the boundary where specification ends and execution begins. The framework fixes the *contract* that crosses it — an emitted unit travelling **by value with a content-hash identity**, and a **self-describing verdict event** returning from the executor (the producer holding no knowledge of any consumer) — and fixes nothing on the far side: how a unit is scheduled, run, or batched is the executor's concern, out of scope, so that any executor honouring the seam is pluggable and either pillar is swappable. The freeze precedes the emit: a unit that would require a callback mid-execution is not frozen and must not be emitted.
9. No output is accepted without a verdict. A verification is a deterministic function of frozen, declared inputs — the artifact, an oracle **derived from the model** (Decider, domain model, layout model, or contracts — never authored inside the check), and versioned criteria — producing a per-criterion finding and a verdict that is the conjunction of those findings; one failure fails the build, with no override-by-assertion. Verifications meet the coherence bar and each names what it protects. Layout conformance is the cheapest verification and runs first; behavioural simulation runs before realisation; where a Decider exists, behavioural conformance checks the realised behaviour produces identical outputs to it.
10. The rationale trace must be true: every claimed principle is enforced by a passing verification or the claim is retracted.
11. Delivery units are subgraphs of the What: a **feature is a reference to a slice of the event model** (one or more flows) and does not separately enumerate concepts — its domain footprint is *derived* by traversing the slice's `change`/`target`/`project` links; a release is a coherent set of features. "Done" is a verifiable predicate (behavioural conformance over the flows, domain conformance over the derived footprint, all citing verifications green, acceptance criteria passing); a release cut must be closed under the transitive closure of dependencies. The What and the How carry **independent semantic versions** that reference each other (a How declares which What-version it realises; bumps are *derivable from what the graph diff touched* — behaviour, additive slice, or realisation only, §7.3). A **target version is a declared partition of feature-slices** (some not yet realised), and **direction is the computed gap** — the unrealised slices in the target — never roadmap prose; a goal that cannot be written as a named set of slices is not yet specified.
12. The delivery model is in scope; delivery practice (cadence, ceremonies) is not.
13. Conformance is claimed at the highest cumulative level satisfied (Described / Realised / Verified / Delivered).
14. Artifacts are authored in the **order the derivation contract forces** (§2.1): system & domain structure → reference data → events & flows → Deciders & Projectors → UI & page graph → the How. Verification is interleaved (simulation before realisation; data conformance continuous), not a final phase. The forward chain is a *cycle*: the intent-reliance rate (under-specification) and the data-divergence rate (over-confidence, post-deployment) send authoring back upstream. The framework is normative about this *order*; *practice* (cadence, ceremonies, who, when) is out of scope (§7.3).
15. The framework defines shapes and rules only. Specific products, archetypes, patterns, Decider logic, and the content of verifications are built on the framework, not part of it.

---

> **In one line.** Describe the What and How as one graph, realise it through referenced work units, gate every output with verifications that also keep the explanation honest, and deliver by partitioning the graph until a computable predicate says you are done.

---

## 11. Design System Conformance Profile

> **🅿 PREVIEW — not yet normative.** This section is a Preview profile, published ahead of a worked reference instance and subject to change before it is folded into the normative body. It introduces **no new requirements**: it is a derived view of §3.2.2, §3.2.3, and §4.5, reorganised from the design system's side of the seam. Where this section and the normative body appear to differ, the normative body governs. Feedback is explicitly invited (see [Contributing](#contributing)).

The framework's UI layering (Task & Concepts → Abstract UI → **Concrete UI** → Final UI, §3.2.2) leaves the Concrete UI layer to a **design system**. This profile states what a design system must provide to plug in as that layer — to be a *conforming design system* — so that screens specified against Abstract Interaction Objects can be realised against it and pass the seam verification (§6.3). The goal is decoupling: any design system meeting this profile can realise any conformant What, and a product can swap design systems without touching its What.

### 11.1 What a conforming design system must provide

A conforming design system is the supplier of **Concrete Interaction Objects** and the rules that map abstractions onto them. It must provide four things:

1. **A component catalog (the CIOs).** A declared set of components, each a Concrete Interaction Object, exposing a stable identifier. This is the closed vocabulary the allowlist (§4.5) checks against — a screen may compose only catalog components.
2. **Reification coverage.** For every Abstract Interaction Object the framework's core defines (§3.2.2), and for every context of use the design system claims to support, at least one reification rule `reify(AIO, context) → CIO` (§4.5). A design system that cannot reify a core AIO in a context it claims to support is **non-conforming for that context** — coverage is the central obligation, the design-system analogue of a Decider's command coverage. Because the **interaction class** (GUI / TUI, §3.2.2) gates which other context dimensions apply, a design system declares its class in `contexts_supported`; a GUI design system reifies AIOs to visual components, a TUI one reifies the *same* AIOs to terminal widgets (a box-drawing list, a reverse-video button, a tab-navigated form). Where an AIO cannot honestly cross into a class, the system declares it **unreifiable** for that class (a recorded gap with rationale, §4.5) rather than leaving coverage silently incomplete.
3. **Token surface.** A declared set of design tokens (colour, spacing, typography, …) such that every styling choice a component exposes is a token reference, never a literal — so that screens carry tokens, not values (§4.5).
4. **Accessibility guarantees.** For each component, a declared statement of which **WCAG 2.2 success criteria** (§3.2.3) it satisfies *by construction*, and by which verification type (machine / assisted / manual). This is what lets a screen *inherit* accessibility discharge from its components instead of re-attesting per screen: if a `text-entry` AIO reifies to a component the design system guarantees satisfies the labelling criteria, the screen's obligation for those criteria is discharged by the binding.

### 11.2 The coupling — how a What meets a conforming design system

The What references AIOs and contexts of use; the design system supplies CIOs and reification rules; they meet at the seam:

```
What (§3.2)         AIO  ×  context of use
                      |  reify(AIO, context) -> CIO        (design system, §4.5)
Design system       CIO  (+ tokens, + WCAG guarantees)
                      |  composes into
Screen (page)       a UI step realised — checked at the seam (§6.3)
```

The seam verification confirms, against the design system's declarations: every datum shown is projected, every control maps to a valid command, **every referenced AIO has a reifying CIO for each declared context** (resolved through the design system's reification rules), every projection state is covered, and every machine WCAG criterion the step requires is satisfied by a component that guarantees it (with attestations for the rest). A conforming design system is precisely one against which this verification *can* pass for any conformant What within its claimed contexts.

### 11.3 The design-system manifest (Preview schema)

A conforming design system ships a **manifest** — the machine-readable declaration of the four obligations above, the artifact tooling validates and the reifier reads. The Preview shape:

```yaml
# design-system.manifest.yaml  (PREVIEW schema)
design_system:
  id: "acme-ds"
  version: "3.2.0"
  wcag_target: "2.2-AA"               # the conformance level it claims

  contexts_supported:                 # which contexts of use it can reify into
    interaction_class: [gui]          # the gating dimension (§3.2.2): gui | tui | …
    form_factor: [phone, tablet, desktop]   # in scope because class = gui
    modality:    [pointer, touch]

  components:                         # the CIO catalog
    - id: searchable-list
      tokens: [color.fg, color.bg, space.inset, type.body]
      satisfies:                      # WCAG guarantees, by verification type
        - { criterion: "1.3.1", level: A,  via: machine }
        - { criterion: "4.1.2", level: A,  via: machine }
        - { criterion: "2.4.7", level: AA, via: assisted }
    - id: segmented-control
      tokens: [color.fg, color.bg, space.inset]
      satisfies:
        - { criterion: "1.3.1", level: A, via: machine }
    - id: primary-button
      tokens: [color.accent, color.on-accent, space.inset, type.label]
      satisfies:
        - { criterion: "2.5.8", level: AA, via: machine }   # target size
        - { criterion: "2.4.7", level: AA, via: assisted }

  reification:                        # reify(AIO, context) -> CIO, with rationale
    - aio: single-select
      when: { form_factor: phone, options: many }
      cio: searchable-list
      rationale: "a phone has no room for many side-by-side options"
    - aio: single-select
      when: { form_factor: tablet, options: few }
      cio: segmented-control
      rationale: "few options, ample width — direct choice beats a menu"
    - aio: trigger-action
      when: { emphasis: primary }
      cio: primary-button

  unreifiable:                        # recorded, deliberate coverage gaps (§4.5)
    - aio: display-collection
      class: tui
      rationale: "an image gallery has no faithful character-grid form"

  tokens:                             # the token surface
    - { id: color.accent,    type: color }
    - { id: space.inset,     type: dimension }
    - { id: type.label,      type: typography }
```

The manifest's three blocks correspond exactly to the obligations: `components` is the CIO catalog and its WCAG guarantees (obligations 1 and 4), `reification` is the coverage obligation (2), `tokens` is the token surface (3). A validator checks the manifest is internally whole — every `cio` named in `reification` exists in `components`; every `criterion` is a real WCAG 2.2 entity; every token referenced by a component is declared — and a coupling check confirms the manifest's `reification` covers every core AIO across every context in `contexts_supported`.

### 11.4 What this enables

- **Pluggability.** A What specified against AIOs is design-system-agnostic; pointing it at a different conforming manifest reifies the same screens through different components, with no change to the What. This is the entire reason the AUI layer exists.
- **Per-context adaptation as data.** The phone-vs-tablet decision lives in `reification`, declared and rationale-carrying, not in code.
- **Accessibility by inheritance.** A screen inherits its components' WCAG guarantees, so accessibility is discharged at the design-system level once, not re-litigated per screen — only the genuinely screen-specific criteria need step-level attestation.

> **Why Preview.** This profile is coherent with the normative body but has not yet been exercised against a real design system and a worked What. It is published to invite exactly that coupling experiment; the schema and the obligation set may change in response. It will be proposed for normative status once a reference instance has demonstrated a conforming design system realising a conformant What end to end.

---

## 12. Content Store Conformance Profile

> **🅿 PREVIEW — not yet normative.** Companion to §11, for the *content* provider rather than the *component* provider. It introduces **no new requirements**: it is a derived view of §3.2.1 (content references) and §4.6 (content resolution), reorganised from the content store's side of the seam. Where this section and the normative body appear to differ, the normative body governs.

The What references content by key with a role (§3.2.1); the How resolves those keys against a **content store** (§4.6). This profile states what a content store must provide to plug in — to be a *conforming content store* — so that screens referencing content keys can be realised against it and pass the seam verification (§6.3). The store is to words what a design system (§11) is to components: a swappable provider behind a conformance profile, with **locale** as its context dimension.

### 12.1 What a conforming content store must provide

1. **A keyed catalog.** A declared set of content entries, each with a stable **key** and a **role** (heading / body / empty-message / error-message / help / label-override / legal / …). The role is the contract between the What's reference and the store's value.
2. **Locale coverage.** For every key, a resolved string in **each locale the store claims to support**. A store that cannot resolve a key the application references, in a locale it claims, is **non-conforming for that locale** — coverage over (key, locale) is the central obligation, exactly parallel to a design system's reification coverage over (AIO, context).
3. **Role conformance.** Each resolved value must satisfy its role's constraints (an error-message is non-empty and actionable; a heading fits the length its role admits; legal text is present where required). This is what lets the seam verification check content for more than mere existence.

### 12.2 The content manifest (Preview schema)

```yaml
# content-store.manifest.yaml  (PREVIEW schema)
content_store:
  id: "acme-content"
  version: "1.4.0"
  locales_supported: [en, es, de]

  entries:
    - key: checkout.review.heading
      role: heading
      values: { en: "Review your order", es: "Revisa tu pedido", de: "Bestellung prüfen" }
    - key: cart.empty.message
      role: empty-message
      values:
        en: "Nothing to check out yet — add something to get started."
        es: "Aún no hay nada para pagar — añade algo para empezar."
        de: "Noch nichts zum Bezahlen — füge etwas hinzu."
    - key: cart.failed.message
      role: error-message
      values:
        en: "Couldn't load your cart. Check your connection and retry."
        es: "No se pudo cargar tu carrito. Revisa tu conexión e inténtalo de nuevo."
        de: "Warenkorb konnte nicht geladen werden. Verbindung prüfen und erneut versuchen."
```

A validator checks the manifest is whole — every key carries its role; every claimed locale has a value for every key; every error/empty role resolves to non-empty actionable text — and a coupling check confirms the manifest resolves every (content key, locale) the application's UI steps and shell reference (§3.2.1, §3.2.4).

### 12.3 What this enables

- **Localization without touching the What.** The same content keys resolve through a store covering different locales; the What does not change per language, just as it does not change per device. A localized application is a coupling, not a fork.
- **Editorial ownership.** Copy lives in the store, owned and revised by the people who own words, decoupled from the model and the components — the content analogue of the design system owning components.
- **Checkable copy.** Because keys carry roles, empty error messages, missing legal text, and overflowing headings are catchable at the seam, not discovered in production.

> **Why Preview.** Like §11, this profile is coherent with the normative body but unexercised against a real content store and a worked What. The two profiles are deliberately parallel; a reference instance should demonstrate one conforming design system *and* one conforming content store resolving the same What end to end, after which both are proposed for normative status together.

---

## 13. Data Conformance Profile

> **🅿 PREVIEW — not yet normative.** Unlike §11 and §12 (which describe *providers* that plug into a full instance), this profile describes the framework's **minimal standalone adoption**: data conformance used on its own, by a system that uses none of the rest of the framework. It introduces no new requirements — it is §3.1 (structure and data) and §6.3 (data conformance) packaged as an entry point. The normative body governs on any apparent difference.

### 13.1 What it is

The smallest useful slice of the framework is: **declare the shapes your data should satisfy, then assert them against real data — continuously.** A system needs only a domain *structure* (entities, relations, cardinalities, invariants as validatable shapes — §3.1) and a production dataset. It needs no event model, no Decider or Projector, no UI model, no work units. This is the lowest-commitment, highest-immediate-value way in: every system already has data, and almost none can answer *"is it valid against what we believe should be true?"*

### 13.2 What a conforming adoption provides

1. **A declared structure.** The entity types, relations, cardinalities, and invariants the data is expected to satisfy, expressed as validatable shapes (RDF + SHACL is the reference; an equivalent conforms). This is the structure side of §3.1, and nothing more is required.
2. **A bound dataset.** A declared production dataset (or several) that the shapes are evaluated against — the oracle (§3.1).
3. **Continuous assertion.** The shapes are evaluated against the data not once but on an ongoing basis — on write, or on a schedule — so conformance is a *standing* signal, not a one-time check (§3.1, "authored once, asserted continuously").
4. **Bidirectional triage.** A declared posture for reading failures: each violation is triaged as *data defect* (fix the data) or *spec drift* (fix the model), because a data-conformance failure is ambiguous by design (§3.1). An adoption that only ever blames the data is using half the capability.

### 13.3 The signal it produces

The **data-divergence rate** — the proportion of production data that fails the declared shapes, tracked over time (§3.1). It is the over-confidence counterpart to the intent-reliance rate: a rising divergence rate is a model falling behind reality, made visible as it happens. A conforming adoption surfaces this rate, not just pass/fail, because the *trend* is the early warning a single verdict hides.

### 13.4 Why it is the doorway

A team that adopts data conformance alone has, as a side effect, done the hardest part of Level 1 (§8.1): declared a real, validatable domain structure. From there, the rest of the framework is incremental — the event model attaches to entities that already exist, Deciders and Projectors make their behaviour executable, the UI model derives from projections. Data conformance pays for itself immediately *and* lays the foundation the rest builds on, which is why it is offered as the entry point rather than a final level.

> **Why Preview.** This profile names a packaging, not a new mechanism; it awaits a reference adoption that runs data conformance continuously against a real production system and reports a data-divergence trend, after which it is proposed for normative status as the framework's recognized minimal entry point.

---

## License

The **specification text** in this repository is licensed under [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/) — see [`LICENSE-docs`](../LICENSE-docs).

Accompanying **shapes, schemas, and code** (RDF vocabulary, SHACL shapes, examples, tooling) are licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) — see [`LICENSE`](../LICENSE).

## Contributing

Proposals, conformance reports, and reference tooling are welcome. See [`CONTRIBUTING.md`](../CONTRIBUTING.md). The specification is versioned ([`CHANGELOG.md`](../CHANGELOG.md)); breaking changes follow a documented deprecation policy so that existing conformant instances are never silently invalidated.