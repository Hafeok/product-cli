# AI Development Foundations — Conformance Statement

> This statement asserts that **The Product Framework** conforms to the
> [AI Development Foundations](https://github.com/Hafeok/ai-development-foundations)
> **Pillar One — Specification Framework**. Conformance is self-declared: every
> claim below cites the section of [`docs/product-framework.md`](docs/product-framework.md)
> that satisfies it, and anyone may verify it by reading the framework against the
> published [specification checklist](https://github.com/Hafeok/ai-development-foundations/blob/main/conformance/specification-conformance.md).

## Framework

| Field | Value |
|-------|-------|
| **Name** | The Product Framework |
| **Version** | 1.6.0 |
| **Repository** | https://github.com/Hafeok/product-framework |
| **License** | Apache-2.0 (shapes, schemas, code) · CC BY 4.0 (specification text) |
| **Maintainer** | Emil ([@Hafeok](https://github.com/Hafeok)) |
| **Conforms to** | Specification Framework ☑ / Execution Contract ☐ |
| **Foundation version targeted** | `ai-development-foundations` @ `13b0cb4` (2026-06-26) |
| **Autonomy level** | n/a (specification framework; does not execute) |

## Statement

> We assert that any artifact produced under **The Product Framework**, following
> its defined process, satisfies every requirement checked below. Each checked
> item cites where in the framework the requirement is met. Requirements that are
> only partially met, or deliberately out of scope, are declared honestly in
> [Known gaps](#known-gaps) rather than overclaimed.

The Product Framework is a catalog-agnostic standard for describing software as
three connected, machine-readable models — **What**, **How**, and **Delivery** —
in one typed graph. It is, by construction, a Pillar One framework: it defines the
shape of a specification, the derivation contract that keeps it traceable, and the
seam at which a frozen work unit is handed to an executor. It stops at that seam
and specifies nothing on the far side of it (§5.1), so it makes **no** Pillar Two
claim.

## Pillar One — Specification

**Coverage:** What layer ☑ · How layer ☑ · SPMC ◐ (partial — see §5 below and
[Known gaps](#known-gaps)).

### Summary

| Requirement | Met? | Where in the framework |
|-------------|------|------------------------|
| Premise (LLM input artifact, 5 properties) | ☑ | §1, §2, §9, §5, §6 |
| Layer separation (What / How / SPMC) | ☑ | §2, §8 (mapping table) |
| What completeness (9 sections) | ◐ | §3, §10 — 7 of 9 fully; 2 partial |
| How completeness (8 sections) | ◐ | §4, §9 — 7 of 8 fully; 1 partial |
| Derivation contract (5 rules) | ☑ | §9 (typed links), §4.1, §10 |
| SPMC (4 axes, frozen context) | ◐ | §5, §5.1, §6.2 — Schema & Context met; Model axis out of scope |
| Scale discipline | ☑ | §10 (rules 1, 13–15), §8.1 |
| The seam (Section 7) | ☑ | §5.1 — defined, frozen, independently versioned, gated before emit |

### Section 0 — Premise (spec is an LLM input artifact)

| Property | Met? | Where |
|---|---|---|
| Unambiguous — every term defined, nothing left to inference | ☑ | One typed graph with defined node/link kinds (§9); "intent" (unmodelled residue) is explicitly marked as specification debt to be promoted, not left to the model (§3.2.1, rule 4) |
| Complete at the boundary | ☑ | Work unit carries a **frozen, fully-resolved input bundle**; freeze precedes emit; a unit needing a mid-execution callback must not be emitted (§5, §5.1) |
| Internally consistent — contradictions resolved in the spec | ☑ | The derivation contract and verification kinds (state/Decider justification, coherence bar) surface contradictions as findings (§6.2, §6.3, §9) |
| Machine-readable — structured, not prose | ☑ | A product is one machine-readable graph; reference encoding RDF + SHACL, any typed-node/typed-link/validatable-shape encoding conformant (§9, §10 rule 1) |
| Explicitly bounded — out-of-scope declared | ◐ | Framework-level open/closed line is explicit (§1); bounded contexts carry explicit cross-context mappings (§3.1). A required *per-What* non-empty out-of-scope list is not enforced — see Known gaps |

### Section 1 — Layer separation

| Requirement | Met? | Where |
|---|---|---|
| What / How / SPMC kept distinct, not collapsed | ☑ | §2 "the three models and the split"; §8 mapping table |
| What ownable/validatable by a non-technical product role | ☑ | What (systems, domain model, event model) owned by product and design; authored and agreed before the How (§2.1, §3, §10 rule 2) |
| How authored by the team and **derived** from the What, not restating it | ☑ | §4; "references, never re-derives" (§5); `derived_from` link (§9) |
| Each unit of work decomposes into an SPMC bundle before a model consumes it | ☑ | Work unit = one bounded transformation with frozen input (§5); §8 maps it to SPMC |

### Section 2 — What completeness

| Required element | Met? | Where |
|---|---|---|
| Identity — single unique name | ☑ | Each **system** is a first-class node with a name, used consistently downstream (§3.2.5, §10 rule 2) |
| Purpose — problem / who / why | ☑ | §1; each system carries a purpose (§3.2.5) |
| Actors — every person/role/system, what each may and may not do | ☑ | Triggers typed by source (user / external / automated) (§3.2.0); what an actor *may not* do is the Decider's reachable rejections (§3.3, Decider justification §6.3) |
| Behaviors — initiator, precondition, postcondition, **and unhappy path** | ☑ | Event model patterns (§3.2.0); the Decider derives commands/events/evolution and **must reject** ≥1 invariant — the unhappy path (§3.3) |
| Boundaries — in/out scope; out-of-scope **non-empty** | ◐ | In-scope is derived (feature footprints, allowlist); framework-level scope explicit (§1). A mandatory non-empty out-of-scope list per What is not enforced — see Known gaps |
| Data — every entity CRUD, with owner and lifecycle | ☑ | Domain model structure + data (§3.1); events `change` entities (§3.2, §9); reference vs production data distinguished (§10 rule 3) |
| Constraints — every non-derivable rule, with source and a way to verify | ☑ | Invariants in a constraint language (§3.1); constraints map to NFRs and are checked (§6, §9) |
| Acceptance Criteria — ≥1 testable criterion per behavior | ☑ | Versioned criteria per artifact kind; "done" is a predicate over behavioural conformance and acceptance criteria (§6.2, §7.2) |
| Open Questions — every known unknown, with owner and what it blocks | ◐ | Under-specification is a **measured signal** (intent-reliance rate, §3.2.1; data-divergence rate, §3.1) and undeclared decisions are surfaced (derivation Rule 5). A dedicated owner-tagged Open-Questions artifact is not a required construct — see Known gaps |

### Section 3 — How completeness

| Required element | Met? | Where |
|---|---|---|
| What Reference — identity and version derived from | ☑ | Work units reference the What/How by pointer; `derived_from` carries the upstream identity (§5, §9); Level 2 (§8.1) |
| Components — single-responsibility, dependencies directed and acyclic | ☑ | Contracts fix layering and isolation; decision logic kept in a pure isolable core (§4.2); structure enforced by the repository layout model (§4.3) |
| Data Model — a technical rep for every What entity, no orphans | ☑ | Derivation Rule "every data model entry maps to a What data entity"; domain conformance forbids structural drift (§6.3, §9) |
| Integrations — external system, direction, protocol, contract, **and failure** | ☑ | Interface contracts use industry standards generated from the domain model (§4.4); cross-system Translation pattern (§3.2.0); failure handled by error categories and `failed` read-model state (§3.2, §6.3) |
| Decisions — question, decision, rationale, rejected alternatives, driving What | ◐ | Decisions carry what/why/when-it-applies/when-it-does-not, by pointer (§4.1); driving What via `derived_from` (§9). "Rejected alternatives" is implied by "when it does not apply", not a mandated field — see Known gaps |
| Error Handling — a declared behavior per error category | ☑ | Decider rejections (§3.3); `failed` projection state (§3.2); error/seam verification kinds (§6.3) |
| Non-Functional Requirements — measurable threshold per What constraint | ☑ | Derivation Rule "every NFR maps to a What constraint"; verifications are deterministic gates (§6, §9) |
| Open Questions — propagated and structured as in the What | ◐ | Same mechanism and same gap as the What layer — see Known gaps |

### Section 4 — The derivation contract (5 rules)

| Rule | Met? | Where |
|---|---|---|
| Every component traces to ≥1 What behavior or data entity | ☑ | `derived_from` typed links (§9); rationale trace per work unit (§5) |
| Every data model entry maps to a What data entity | ☑ | §9; domain conformance verification (§6.3) |
| Every NFR maps to a What constraint | ☑ | §9; NFRs derive from constraints (§4, §6) |
| Every decision traces to a What element that required it | ☑ | Decisions linked to their driving What element (§4.1, §9) |
| Every How element with no What anchor is flagged as an undeclared product decision and escalated | ☑ | Surfaced, not silently blocked: the `escalate` verdict (§6.2) and the **earn-their-place rule** (§4.1) flag elements with no anchor; the intent-reliance rate measures the residue (§3.2.1) |

### Section 5 — SPMC

The work unit (§5) is the framework's SPMC bundle (§8 mapping). Coverage is partial
by design — the framework owns the **specification** axes and treats the
**Model** axis as the executor's concern beyond the Build seam (§1 open/closed,
§5.1).

| Axis / property | Met? | Where |
|---|---|---|
| **Schema** — output shape; non-conforming output rejected | ☑ | One unit produces one artifact against versioned acceptance criteria; a failed verdict rejects it (§5, §6.2); the acceptance schema is dual-read like the layout model (§4.3) |
| Schema pins the shape language, recorded alongside | ☑ | Encoding fixes the shape language (SHACL or any validatable-shape language), recorded in the graph (§9) |
| **Prompt** — derived from What behaviors/criteria, filtered through How responsibility | ◐ | A unit reads its What concepts and How principles/patterns by pointer and re-derives nothing (§5); the framework does not name "prompt" as a separate pinned axis — see Known gaps |
| **Model** — selected by complexity remaining after the How | ☐ | **Out of scope by design.** Choosing/running the model is the executor's concern across the seam (§1, §5.1) — see Known gaps |
| Model pinned as a binding (precision/quantization/params) | ☐ | Out of scope — same reason |
| **Context** — assembled from the How, **frozen and bounded** at execution; no mid-execution retrieval | ☑ | Frozen, fully-inlined input bundle travelling by value with a content-hash identity; nothing fetched mid-run; freeze precedes emit (§5, §5.1, §6.2) |
| Each axis pinned to the precision at which it affects output | ◐ | The frozen bundle's **content hash is its identity**, so any change to schema/context is a distinct unit (§5.1); Model-axis pinning is out of scope |
| Axes independently attributable (a quality failure points to which axis) | ◐ | Per-criterion findings localize failures to a model element (§6.2) and the rationale trace retracts exactly the failed claims (§5); attribution is by graph element rather than by named SPMC axis |

### Section 6 — Scale discipline

| Requirement | Met? | Where |
|---|---|---|
| Same floor for a one-page feature as for a large project (same structure, smaller content) | ☑ | The framework defines shapes and rules only and applies them uniformly; authoring follows the order the derivation contract forces regardless of size (§10 rules 1 & 14–15) |
| A spec missing a required section or holding placeholder content is a draft and does not proceed | ☑ | Conformance is cumulative and claimed at the highest level **satisfied**; an incomplete What does not reach the How (§8.1, §10 rule 13); verification is by construction, not assertion (§6.1) |

### Section 7 — The seam (framework hands work to an execution pillar)

The Product Framework emits frozen work units to an executor, so the seam section applies — and is a first-class part of the design (§5.1).

| Requirement | Met? | Where |
|---|---|---|
| The framework **defines** the handoff artifact's shape (contents, identity, return event) | ☑ | The Build seam fixes two schemas: an emitted work unit (by value, content-hash identity) and a self-describing verdict event (§5.1) |
| The unit is **frozen and self-contained** at emit; no callback to resolve | ☑ | "The freeze is the decoupling"; a unit needing a mid-execution callback is not frozen and must not be emitted (§5.1) |
| The interface **versions independently** of the framework (executor swap needs no framework change) | ☑ | A data contract with no live surface; either side can be rebuilt against the contract alone; by-value→by-reference is additive, not breaking (§5.1) |
| Any spec-side conformance gate the seam depends on runs **before emit** | ☑ | Layout conformance runs first and behavioural simulation runs before realisation; the freeze (and its gates) precede the emit (§6.3, §5.1) |

## Pillar Two — Execution

**Not claimed.** The Product Framework is a specification pillar with a published
socket (§5.1): it defines the Build seam and the verdict-event vocabulary, but it
deliberately specifies nothing about how a unit is scheduled, admitted, batched,
or run — that is the executor's concern, out of scope by the open/closed line
(§1, §5.1). It declares none of the eight Execution building blocks (workers,
capabilities, environment, tools, credentials, transition contract) and asserts no
closure contract. An execution framework conforming to Pillar Two can plug in
behind this seam without either side changing.

## Known gaps

Honest partial conformance, with intent for each item.

| Gap | Status / plan |
|-----|---------------|
| **What/How Open Questions** — no dedicated owner-tagged "Open Questions" artifact with an explicit "what it blocks" field. | Functionally covered by measured under-specification signals (intent-reliance rate §3.2.1, data-divergence rate §3.1) and by Rule 5 / the `escalate` verdict surfacing undeclared decisions. Considering promoting an explicit Open-Question node kind in a future cut so each known unknown carries an owner and a blocked-set as graph data. |
| **Out-of-scope list per What** — the framework does not require every What instance to carry a non-empty explicit out-of-scope list (it declares scope at the framework level and derives in-scope footprints). | Under consideration as a What-side completeness rule; today the boundary is expressed structurally (bounded-context mappings, derived feature footprints, allowlist layout) rather than as a prose exclusion list. |
| **Decision "rejected alternatives"** — §4.1 mandates what/why/when-it-applies/when-it-does-not but not a discrete rejected-alternatives field. | "When it does not apply" carries the same information for traceability; a dedicated `rejected_alternatives` attribute is a candidate addition to the decision shape. |
| **SPMC Model axis** — the framework does not select, pin, or bind the model (precision/quantization/invocation params). | **Deliberate scope boundary**, not a defect: model choice and runtime live across the Build seam and belong to a Pillar Two executor (§1, §5.1). A Pillar One + Pillar Two pairing built on these foundations closes this axis. |
| **SPMC "Prompt" as a named axis** — the unit's instruction derives from What+How by pointer but is not pinned as a distinct labelled axis. | Coverage is by derivation and reference rather than by an explicit per-axis label; revisiting whether to name the prompt axis explicitly when (or if) the framework moves closer to the execution boundary. |

---

*Statement last verified: 2026-06-26 against ai-development-foundations `13b0cb4`.*
