# The Product Framework

**An open standard for specifying software as What, How, and Delivery.**

*A conformant instantiation of the Two Pillars Specification Framework. Version 1.1 — open specification.*

---

> **What this is.** An **open, catalog-agnostic standard** for describing a software product as three connected models — **What** it is, **How** it is built, and how it is **Delivered** — in a form that is reproducible, verifiable, and traceable. It defines the **shapes and rules**, not any particular product. Anyone can build catalogs, archetypes, and tooling against it.
>
> It is a conformant instantiation of the **Two Pillars Specification Framework** for software; every construct maps to a named Two Pillars concept ([§8](#8-conformance-to-the-two-pillars)).

*Published under a dual license: the specification text under CC BY 4.0, and accompanying shapes/code under Apache-2.0 ([License](#license)). This document describes the format; it ships no proprietary catalog content.*

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
  |  decisions / principles / patterns  +  contracts  +  interface & screen standards
  v
DELIVERY  how the What is brought to a verifiable 'done' — model only
          features / releases as partitions of the What; done as a predicate
```

**The split is the central idea.** Above it lives **meaning**, expressed in the language of the business and owned by the people who own the product. Below it lives **realisation**, owned by engineering. The What is authored and agreed **before** the How is written. The line between them is where business meaning becomes technical reality, and keeping it explicit is what lets non-engineers own the What and engineers own the How without either guessing the other's intent.

**Everything is one graph.** All three models, and the links between them, form a single machine-readable graph (RDF is the reference encoding; [§9](#9-encoding-and-the-derivation-contract)). "Describe this system" is therefore a query, not a stale document, and impact analysis ("what depends on this?") is a graph traversal.

---

## 3. The What — structure and behaviour

The What is the specification every role reads and agrees on. It has two halves, expressed in one graph: the **domain model** (what exists) and the **event model** (what happens) — and, where behaviour is interesting, a **Decider** that makes that behaviour executable (§3.3).

### 3.1 The domain model — structure

The shared definition of **what concepts mean** and how they relate. Conformance requirements:

- **Bounded contexts, not one flat model.** Within a context a term has exactly one meaning (the **ubiquitous language**); the same word may mean different things in different contexts. Cross-context correspondences are **explicit declared mappings**, never assumed. (This is what resolves "is a User a Customer?" rather than restating the confusion.)
- **Entities, relations, value objects, invariants.** Relations carry cardinality **and rationale**. Invariants are stated as machine-checkable constraints.
- **Machine-readable, with a constraint language.** The model must be expressible as a graph with validatable shapes (RDF + SHACL is the reference; an equivalent is conformant). A diagram alone is not a conformant domain model — it cannot generate or validate.

### 3.2 The event model — behaviour

The description of **what happens over time**, peer to the domain model, in the same graph. Conformance requirements:

- **Built from domain-typed primitives.** Events, commands, read models, and UI steps each reference domain concepts: an event **changes** an entity, a command **targets** an aggregate, a read model **projects** entities. Behaviour may not reference structure that does not exist. A read model also declares (or makes inferable from its query shape) its **state space** — at minimum `present`, plus any of `loading`, `empty`, or `failed` it can actually exhibit: a projection over a possibly-empty collection can be `empty`, one backed by something that can be pending can be `loading`, one whose delivery can fail can be `failed`; a guaranteed-singleton, synchronous projection has none of these. The state space is mostly *derived* from the projection's shape and declared only where the shape does not settle it.
- **Depth is proportional to behavioural complexity.** Concepts with rich or historical behaviour get full event models; simple create/read/update/delete concepts get a minimal one or none. The framework requires that the *interesting* behaviour be modelled, not that every triviality be ceremony.
- **Owned by product and design.** Its natural form — a timeline with interface steps — is readable and signable by non-engineers; it is the bridge between concepts and screens.

> **Why structure and behaviour are one graph.** An event is never free-floating: it always changes a domain entity. Modelling them in one graph means structure and behaviour cannot drift apart, and a single question — "what happens to this concept?" — returns both its shape and the flows that change it.

#### 3.2.1 The UI step — the What of a screen

A **UI step** is the point in a flow where a person sees something and acts. It is **What**, owned by product and design: it says what a screen is *for* and *means*, never how it looks. How it is composed and rendered is the screen-composition contract in the How (§4.5); the two meet at a verified seam. The split is not a matter of discipline but of **type**: a UI step may reference only **Abstract Interaction Objects** (§3.2.2) — `single-select`, `trigger-action`, `text-entry`, and the like — never a concrete control (a dropdown, a button) from a design system. An AIO is meaning; a concrete control is realisation. Because the two are distinct kinds of node in the graph, "a UI step naming a dropdown" is not a style lapse to be caught by review — it is a structural violation a verification rejects (§6.3), the same way the layout allowlist rejects a misplaced file. The fusion the framework forbids (§2) is made impossible by construction, not discouraged by instruction.

A conformant UI step declares two layers. The first is the **buildable core** — four facts, each already a node or edge in the model, so the step is an *annotation on the flow*, not a new artifact:

- **Intent** — one sentence, in the ubiquitous language, for what the user is trying to accomplish here. Intent is **not modelled specification** — it is the marked residue of what the model does not yet determine (see *Intent is specification debt* below). It is permitted, but it is a liability to be driven down, never the field that "gives the implementer good judgment."
- **Information shown** — the read-model **projection** this step surfaces (it `projects`, §3.2), presented through one or more **display AIOs** (`display-value`, `display-collection`, …). The What names *which projection* and *which abstract interaction*, never how it is laid out. ("Surfaces the order-summary projection as a `display-collection`" — not "in a table.")
- **Actions available** — the **commands** the user may issue here (exactly the commands valid at this step — the ones a Decider `handles`, §3.3), each offered through an **action or input AIO** (`trigger-action`, `single-select`, `text-entry`, …). The What names *which decision* and *which abstract interaction*, never the control. ("`single-select` of a shipping option, `trigger-action` to confirm" — not "a dropdown and a primary button.")
- **Transitions** — given an action or a resulting event, which UI step follows. This is the flow's own topology; the step merely names its position in it.

The second layer is **modelled meaning** — declarations that shape realisation without prescribing it. These are *not* residue: each is a checkable model element the seam verification (§6.3) enforces, and each is a thing intent gets *promoted into* (see below):

- **Emphasis** — what matters most at this step, so the How knows what to make prominent. ("The total owed is the decisive figure" — not "make it large and bold.")
- **Projection states as meaning** — a projection is not always a present value; it can occupy any state in its declared **state space** (§3.2), and what each state *means to the user* is a behavioural fact, not styling. A UI step's state annotations are both **constrained** and **covering**: it may annotate only states the projection actually has (no "empty" meaning for a projection that cannot be empty), and it must give a meaning to *every* state the projection can be in, or explicitly waive one with a reason. This is the UI analogue of the Decider's command-coverage rule (§3.3) — exhaustiveness over the projection's state alphabet — and the dangerous case it catches is the *forgotten* state: a projection that can fail whose screen never says what failure means. The waiver is the escape hatch a Decider doesn't get, because some states are legitimately ignorable (a load too fast to perceive); the waiver must say why. For `failed` specifically, the *meaning* is What ("the user must know it can't be shown and how to recover"); the failure *mechanism* is How — the same split as an accessibility criterion versus its discharge (§3.2.3).
- **Accessibility obligations** — *not* prose about what "must be perceivable," but a set of **WCAG 2.2 success criteria** the step must satisfy, referenced as ingested entities (§3.2.3). Most are inherited from the AIOs the step uses (a `text-entry` carries its labelling criteria, an image-bearing `display-value` carries 1.1.1 Non-text Content); a step may add screen-specific criteria. This replaces the old free-text "accessibility intent": a criterion is a checkable entity with a known verification type, where prose was an unverifiable wish.

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

**The context of use is a declared model.** Per the four-level model, context-of-use enters at the Abstract→Concrete boundary, not above it: the AUI is deliberately context-independent, and the *same* AIO is reified differently depending on context. A conformant instance declares its relevant **contexts of use** — at minimum **form factor** (e.g. phone, tablet, desktop) and **modality** (e.g. pointer, touch, voice), and optionally user type and physical environment — as named elements. Context-of-use is What-side knowledge (a fact about who uses the system and where), but it carries no realisation; it is the parameter the How's reification rules (§4.5) are written against. This is what makes "phone → searchable list, tablet → segmented control" a *declared, traceable rule* rather than an implicit decision buried in code.

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

### 3.3 The Decider — the executable form of behaviour

The event model says *what* happens; a **Decider** says it *executably*. For a consistency boundary (typically an aggregate), a Decider is a pair of pure functions:

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

### 3.5 Named-algorithm primitives — the Polanyi floor

Not every computation reduces to declared rules over a generic interpreter. Betweenness centrality, SHA-256, a YAML grammar, a geometric solver — these are **irreducible**: there is no declarative fold or rule-set that *is* the computation; the algorithm itself is the specification. The framework states the governing principle plainly:

> A computation is mechanically derivable from spec **exactly to the degree its variability is declared data over a generic interpreter.** Push a projection's logic into declared rules and it rejoins the mechanical region; what remains — genuinely novel algorithms — is the irreducible kernel.

The honest move for the kernel is to **name and bound it, not pretend it is derivable.** A **named-algorithm primitive** is a first-class What-side element specified by *reference*, never by derivation:

- **Reference** — the named algorithm or standard it implements (e.g. "Brandes' betweenness centrality"; "SHA-256, FIPS 180-4"; "YAML 1.2 grammar"). The name *is* the specification.
- **I/O contract** — its input and output types, expressed in the domain model's terms, so it composes with the rest of the graph.
- **Oracle** — reference input/output pairs (or a reference implementation to differential-test against) that pin the behaviour. This is the only check available, because there is no Decider or Projector to simulate against.

A named-algorithm primitive is explicitly **exempt from derivation and from behavioural simulation**, and is checked by **oracle conformance** (§6.3) — its realised output matches the reference algorithm's, across the oracle's pairs — rather than by behavioural conformance against a fold it does not have. This is the deliberate counterpart to the Decider/Projector: those say "here is the logic, derived and simulated"; a primitive says "here is the *name and the proof obligation*, referenced and test-pinned." A conformant instance may use primitives freely, but each must be *declared as such* — an undisclosed algorithm masquerading as a mechanical projection is the dishonesty the floor exists to prevent.

> **Where the boundary falls in practice.** A `rm-conformance-report` projection *looks* irreducible but is not: its engine (evaluate clauses, collect verdicts) is generic, and only the clause-set varies — so once the clauses are a declared rule-set, the Projector folds mechanically over them and the report leaves the kernel. Betweenness centrality genuinely stays: no rule-set *is* the algorithm. The discipline is to push everything that *can* become declared data into the Projector's rule-set, and to name-and-bound only what truly cannot — keeping the irreducible kernel as small as the problem honestly allows.

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
- **An infrastructure/runtime contract** — the concrete runtime choices for an instance. May vary per deployment; once chosen, frozen. It must **satisfy** the application contract, and the satisfaction is recorded.
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
- **The CIO is always an on-system component.** A reification rule may only target a component the design system defines; it cannot invent one (the closed-vocabulary rule above). Reification chooses *among* the system's components; it never escapes it.
- **Rules carry rationale.** Each rule states why this control suits this AIO in this context — the UX reasoning that is otherwise lost. This is what makes the choice teachable and reviewable, and it is the `enforces`-style citation that lets the rule participate in the rationale trace (§5).

> **The seam is verified.** A screen sits on the What→How seam, like application↔runtime (§4.2). A verification (the seam kind, §6.3) confirms the two halves agree: every datum a page displays is `project`ed by a read model in its flow (no view needs a field no projection supplies — the same completeness the Decider simulation proves for behaviour, §3.3), every control maps to a command valid at that step (no button issues a command the step cannot accept), **every AIO referenced by a UI step has a reifying CIO for each declared context** (reification coverage), every state in each surfaced projection's declared state space (§3.2) is either given a meaning by the UI step and composed, or explicitly waived (state coverage — the UI analogue of command coverage), and the step's **accessibility obligations are discharged**: every *machine* WCAG criterion (§3.2.3) passes as a deterministic gate, and every *assisted* or *manual* criterion has a recorded attestation. The verdict reports the conformance level and its basis, never a bare pass. A complementary, *cheaper* structural check runs first and belongs with the other by-construction gates: **a UI step may reference only AIO-typed nodes; if it references a CIO, the build fails** — the type boundary that makes the What/How split for UI structural rather than advisory (§3.2.1). This is what keeps the UI from drifting from the behaviour it serves, and it is *required* wherever screens are specified, because nothing else makes the screen and the flow agree.

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
| **Internal coherence** | the parts of one work unit's output agree with each other |
| **Contract conformance** | realised code obeys the How's contracts |
| **Seam** | separately-described parts agree — e.g. application vs. runtime (§4.2), or a screen vs. its UI step: every datum shown is projected, every control maps to a valid command, every referenced AIO has a reifying CIO per context, and a UI step references only AIO-typed nodes (§3.2.2, §4.5) |
| **Domain conformance** | realised code matches the domain model; no structural drift |
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

### 7.3 Model, not practice

This section defines the delivery **model** — the partition and the predicate. It deliberately does **not** define delivery **practice** — cadence, ceremonies, who sequences work, release rhythm. Practice is organization-specific; two teams may run the same model very differently. Practice is out of scope.

> **A note for the parent standard.** The Two Pillars defines a verdict on an individual output. `release_done` generalises it to a verdict over a **composition** — all members pass, and the composition is closed (no dangling dependency). This composition-verdict is offered upward as a candidate refinement to the Two Pillars verification concept.

---

## 8. Conformance to the Two Pillars

This framework is a conformant instantiation of the Two Pillars Specification Framework. Each construct maps to a named Two Pillars concept; an instance is Two-Pillars-conformant when it satisfies the mapped requirements.

| Two Pillars concept | This framework's construct | Section |
|---|---|---|
| **What specification** | Domain model (structure) + event model (behaviour, incl. UI steps typed by Abstract Interaction Objects + context-of-use + WCAG 2.2 accessibility criteria) + Decider (executable behaviour) + Projector (executable read model) + named-algorithm primitives (the Polanyi floor) | [§3](#3-the-what--structure-and-behaviour) |
| **How specification** | Decisions/principles/patterns + contracts (incl. repository layout model) + interface standards + screen-composition contract | [§4](#4-the-how--realising-the-what) |
| **SPMC (Schema, Prompt, Model, Context)** | Work unit: one bounded transformation, frozen input, one artifact | [§5](#5-work-units-and-the-rationale-trace) |
| **Derivation contract** | The typed links of [§9](#9-encoding-and-the-derivation-contract) (e.g. `derived_from`, `conforms_to`, `applies`, `realizes`, `enforces`) | [§9](#9-encoding-and-the-derivation-contract) |
| **Verification (criteria → judge → verdict)** | Verification — the required kinds and the coherence bar | [§6](#6-verification--the-conformance-bar) |
| **Verdict (extended to a composition)** | `release_done` — a verdict over a composition | [§7](#7-delivery--bringing-the-what-to-a-verifiable-done) |

### 8.1 Conformance levels

- **Level 1 — Described.** A conformant What (domain + event model) exists as a machine-readable graph with declared bounded contexts and mappings. Where behaviour is interesting, a Decider (§3.3) makes it executable and is simulated sound and complete before any realisation; where a read model's projection is non-trivial, a Projector (§3.4) makes the fold executable to the same depth; genuinely irreducible computations are declared as named-algorithm primitives (§3.5), specified by reference and pinned by an oracle. Where a flow has screens, its UI steps (§3.2.1) declare their intent and derive their data and actions from the model, typed against Abstract Interaction Objects (§3.2.2).
- **Level 2 — Realised.** A conformant How exists (including a repository layout model); work units reference the What and How by pointer; interface contracts use standards and are generated from the domain model; where the event model has UI steps, screens are specified as design-system compositions (Atomic Design) with reification rules mapping each AIO, per context of use, to an on-system component.
- **Level 3 — Verified.** Verifications of all required kinds ([§6.3](#63-the-required-verification-kinds)) exist — including layout conformance — meet the coherence bar, gate acceptance, and back the rationale trace.
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
| `changes` / `projects` / `has_state` | this event changes this entity / this read model projects these / this projection can occupy this state (§3.2) |
| `decides_for` / `handles` / `emits_event` | this Decider governs this aggregate / accepts these commands / may produce these events (§3.3) |
| `projects_for` / `folds` | this Projector builds this read model / folds these events into it (§3.4) |
| `specified_by_reference` / `pinned_by_oracle` | this named-algorithm primitive implements this named algorithm/standard / is checked against these reference I/O pairs (§3.5) |
| `surfaces` / `offers` / `transitions_to` | this UI step surfaces this projection / offers these commands / leads to this next step on an action or event (§3.2.1) |
| `typed_as` / `reifies` / `in_context` | this UI-step interaction is typed as this AIO / this CIO reifies this AIO / this reification holds in this context of use (§3.2.2, §4.5) |
| `must_satisfy` / `attests` | this AIO or UI step must satisfy this WCAG criterion / this dated, attributed attestation records that a non-machine criterion was evaluated and met (§3.2.3) |
| `realizes_step` / `composes` / `binds` | this page realises this UI step / composes these design-system components / binds this control to this command or this field to this projection (§4.5) |

These links are what make the framework **queryable**: impact analysis ("what depends on X?"), onboarding traces ("why is this shaped this way?"), and the verification-to-principle linkage all fall out of graph queries rather than hand-maintained documents.

---

## 10. Conformance rules (normative summary)

1. A product is described as three models — What, How, Delivery — in one machine-readable graph.
2. The What has two halves: a domain model (structure) and an event model (behaviour), authored and agreed before the How.
3. The domain model is bounded contexts with explicit cross-context mappings — never one flat model; with a constraint language for invariants.
4. The event model is built from domain-typed primitives; every event changes a real entity; a read model declares (or makes inferable) its state space (`present` plus any of `loading`/`empty`/`failed` it can exhibit); depth is proportional to behavioural complexity. A **UI step** is a flow primitive whose interactions are **typed against Abstract Interaction Objects** (§3.2.2) — a closed-core, extensible vocabulary of context-independent interaction kinds — never against concrete controls; this type boundary makes the What/How UI split structural, not advisory. Its buildable core is derived from the model (the projection it surfaces, the commands valid at it, its transitions, with input AIOs deriving their fields and validation from the relevant command's payload via the domain model); its modelled meaning — emphasis, the meaning of each state in the surfaced projection's declared **state space** (constrained to states the projection has and covering every one, or waived with reason — the UI analogue of command coverage), and **accessibility obligations** (WCAG 2.2 success criteria, ingested as entities, inherited from its AIOs and extended per step; each tagged machine/assisted/manual, §3.2.3) — is checkable specification. Its **intent** is *not* modelled specification but the marked residue of what the model does not yet determine: it is treated as specification debt, every use of it to settle a realisation choice must be promoted into a modelled element (AIO, context, reification rule, or a meaning declaration), and the intent-reliance rate is a measure of UI under-specification. The relevant **contexts of use** (form factor, modality, …) are declared as What-side elements.
5. Where behaviour is interesting, a **Decider** makes it executable: its signature is derived from the event model (it handles exactly the commands targeting its aggregate, emits only events those commands sanction, evolves from the events that change it, rejects via the aggregate's invariants), and it is simulated sound and complete before realisation. Trivial behaviour needs no Decider. Symmetrically, where a read model's projection is non-trivial, a **Projector** makes the fold executable to the same depth — its signature derived (it folds exactly the events the read model projects, over the entities they change), only its fold logic authored, simulated sound and complete and serving as the projection's oracle; the `projects` link alone is a name, not a function. A computation is mechanically derivable only to the degree its variability is declared data over a generic interpreter; a genuinely irreducible computation (a named algorithm or standard) is declared as a **named-algorithm primitive** specified by reference plus an I/O contract plus an oracle, is exempt from derivation and simulation, and is checked by oracle conformance — never passed off as a mechanical projection.
6. The How captures decisions/principles/patterns (declared once, referenced by pointer, each decision carrying rationale), contracts (stated checkably, including that decision logic is kept in a pure, isolable core), interface contracts (industry standards, generated from the domain model), and — wherever the event model has UI steps — a **screen-composition contract** that binds each screen to a design system structured by Atomic Design (atoms → molecules → organisms → templates → pages), with the component set closed and styling via tokens not literals. Each page's data and controls are derived from its UI step (projected fields, valid commands), and each AIO the step references is **reified** to an on-system concrete control by a declared `reify(AIO, context) → CIO` rule that carries rationale and must cover every (AIO, context) pair its steps can encounter. The whole is checked by the seam verification. **Accessibility** is specified not as prose but as **WCAG 2.2 success criteria ingested as entities** (principle → guideline → criterion → level), each tagged by verification type: *machine* criteria are deterministic gates, *assisted* and *manual* criteria are discharged by recorded, attributed attestations entering through a frozen boundary. Obligations are inherited from a step's AIOs and extended per step; the accessibility verdict reports a conformance level and its basis, never a bare pass.
7. The How includes a glob-based **repository layout model** stating which files must exist (with cardinality), may exist where, must co-exist, and must not exist, with **allowlist semantics** (every file matches an allow rule or fails). Two guards are normative: every rule — especially every prohibition — cites the principle it enforces, and explicit prohibitions are reserved for actively-dangerous cases, the allowlist handling the rest. The layout model is dual-read (it scaffolds and it verifies) and checks tree shape only, layered below the content audits.
8. A work unit is single-purpose with frozen input, references the What/How by pointer, and emits a rationale trace.
9. No output is accepted without a verdict. A verification is a deterministic function of frozen, declared inputs — the artifact, an oracle **derived from the model** (Decider, domain model, layout model, or contracts — never authored inside the check), and versioned criteria — producing a per-criterion finding and a verdict that is the conjunction of those findings; one failure fails the build, with no override-by-assertion. Verifications meet the coherence bar and each names what it protects. Layout conformance is the cheapest verification and runs first; behavioural simulation runs before realisation; where a Decider exists, behavioural conformance checks the realised behaviour produces identical outputs to it.
10. The rationale trace must be true: every claimed principle is enforced by a passing verification or the claim is retracted.
11. Delivery units are subgraphs of the What: a **feature is a reference to a slice of the event model** (one or more flows) and does not separately enumerate concepts — its domain footprint is *derived* by traversing the slice's `change`/`target`/`project` links; a release is a coherent set of features. "Done" is a verifiable predicate (behavioural conformance over the flows, domain conformance over the derived footprint, all citing verifications green, acceptance criteria passing); a release cut must be closed under the transitive closure of dependencies.
12. The delivery model is in scope; delivery practice (cadence, ceremonies) is not.
13. Conformance is claimed at the highest cumulative level satisfied (Described / Realised / Verified / Delivered).
14. The framework defines shapes and rules only. Specific products, archetypes, patterns, Decider logic, and the content of verifications are built on the framework, not part of it.

---

> **In one line.** Describe the What and How as one graph, realise it through referenced work units, gate every output with verifications that also keep the explanation honest, and deliver by partitioning the graph until a computable predicate says you are done.

---

## License

The **specification text** in this repository is licensed under [Creative Commons Attribution 4.0 International (CC BY 4.0)](https://creativecommons.org/licenses/by/4.0/) — see [`LICENSE-docs`](../LICENSE-docs).

Accompanying **shapes, schemas, and code** (RDF vocabulary, SHACL shapes, examples, tooling) are licensed under [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) — see [`LICENSE`](../LICENSE).

## Contributing

Proposals, conformance reports, and reference tooling are welcome. See [`CONTRIBUTING.md`](../CONTRIBUTING.md). The specification is versioned ([`CHANGELOG.md`](../CHANGELOG.md)); breaking changes follow a documented deprecation policy so that existing conformant instances are never silently invalidated.