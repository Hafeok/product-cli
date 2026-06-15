# `product author domain` — specification

**A facilitated, MCP-driven session that captures a product's *What* — the domain model
(structure) and event model (behaviour) — as a conformant graph, in about an hour.
Sibling to `product author feature`. Internal.**

This is the first build of the Product Framework toolchain, and it is deliberately the
cheapest, most visible, lowest-dependency piece: it proves that the knowledge already lives in
people's heads, and the tool simply *harvests* it fast into a machine-readable, validated What
graph. It touches only the What — no How, no cells, no audits, no delivery — so it ships small
and demos in the first hour.

> **Why this first.** It is the "Described" conformance level (framework §8.1) turned into a
> product: a conformant What graph exists. The SHACL shapes already written are its acceptance
> test. It produces the substrate every later build (archetype, cells, audits, feature
> authoring) needs as input. And it is the strongest possible customer demo — at the end of
> the hour the room has a model of *their own* system that they recognise as true.

> **What it does NOT prove.** It proves the framework can *harvest the What cheaply* — a real,
> sellable claim. It does **not** prove the 80%-at-platform-economics thesis, which lives
> downstream in the How/cells/audits. The What-capture is the front door; do not let its speed
> be sold as proof of the delivery economics. Different claims.

---

## 1. Relationship to `product author feature`

`product author domain` and `product author feature` are the same machinery pointed at
different layers of the same graph.

| | `product author domain` | `product author feature` |
|---|---|---|
| Captures | the What — domain + event model | a feature: a flow-shaped subgraph + acceptance |
| Runs | **first** (the What must exist before it can be partitioned) | after, against the captured What |
| Output | a conformant domain+event graph (Level 1: Described) | a `Feature`/`Release` subgraph (Level 4: Delivered) |
| Session model | identical: MCP server, structured ops, in-loop SHACL, hashed provenance | identical |
| Tool surface | the domain/event classes (this document) | the feature/release classes |

Both write to **one graph**. Feature authoring references the domain concepts and flows the
domain session produced — it cannot invent nouns the domain model doesn't define. The ordering
is the framework's seniority made operational: What first, delivery against it.

---

## 2. The session — 6 people, 60 minutes

The premise: a system's domain knowledge already exists, distributed across the people who
built or run it. The session is a facilitated harvest, with an LLM (holding the MCP server) as
the scribe that turns conversation into a validated graph in real time.

**The room (≈6):** product owner (owns meaning), UX (owns the language surface), two
engineers, a domain expert, a facilitator. The exact roster matters less than that the
knowledge is present.

**The choreography:**

| Phase | ~min | What happens | Tools used |
|---|---|---|---|
| **Contexts** | 0–10 | Name the major regions. "What are the big areas of this system?" The model proposes bounded contexts from the conversation and creates them. | `add_bounded_context` |
| **Structure** | 10–30 | Populate each context: the entities, their relations (with cardinality + rationale), value objects, invariants. The hard questions surface here — "is a User a Customer?" — and become explicit **context mappings**. | `add_entity`, `add_relation`, `add_value_object`, `add_invariant`, `add_context_mapping` |
| **Behaviour** | 30–50 | The interesting flows only (core/right-branch). "Walk me through what happens when someone subscribes." Each flow becomes commands → events → read models → wireframe steps. Trivial CRUD is *not* modelled. | `add_command`, `add_event`, `add_read_model`, `add_wireframe_step`, `add_flow` |
| **Close** | 50–60 | Review `open_questions`, fill the gaps it surfaces, run `validate`, `finalize`. | `open_questions`, `validate`, `session_finalize` |

The facilitation is driven by `open_questions` (§4): at any moment the model can ask the tool
"what's incomplete?" and get back the exact gaps — an entity with no context, an event that
changes nothing, a relation with no rationale — phrased as questions to put to the room. That
is what keeps 60 minutes structured rather than meandering.

---

## 3. Design principle: structured operations, never "emit Turtle"

The magic is the model writing the graph from conversation. The danger is that RDF is
unforgiving — a malformed triple or a dangling `changes` link corrupts the graph silently. So
the MCP tools are **structured graph operations**, not free-text emission:

- Each tool constructs a **valid graph fragment** from typed parameters.
- Each mutating tool **runs the relevant SHACL shape immediately** and returns any violation,
  with the framework-section message, so the model **self-corrects in the loop**.
- The model therefore *cannot* produce a non-conformant graph — an invalid `add_event` (no
  entity to change) is rejected at call time with "§3.2 every event must change a real
  entity," and the model fixes it before moving on.

This is what makes the hour **reliable**, not merely fast — the difference between a demo that
works once and a workshop you can run for a paying customer. The schema package
(`product-framework.ttl` + `shapes.shacl.ttl`) is exactly what the server validates against.

---

## 4. The full MCP tool surface

The complete tool schema is in `product-author-domain.tools.json` (every tool's
`inputSchema`, JSON-Schema-validated). Summary:

**Session**
- `session_start` — open/create a product's domain graph; optional seed from a prior session.
- `session_state` — current graph summary + conformance status + open questions.
- `session_finalize` — full validation, export the conformant graph (Turtle), return a
  provenance record (hash, participants, timestamp).

**Domain — structure (§3.1)**
- `add_bounded_context` — a region with one ubiquitous language.
- `add_entity` — a concept with identity; placed in exactly one context.
- `add_value_object` — a concept without identity.
- `add_relation` — a typed link with **cardinality and rationale** (both required).
- `add_invariant` — a checkable rule.
- `add_context_mapping` — an explicit cross-context correspondence (the "User = Customer"
  resolver).

**Event — behaviour (§3.2)**
- `add_command` — an intent; targets an aggregate, emits events.
- `add_event` — a past-tense fact; **changes a real entity** (enforced).
- `add_read_model` — a view; projects entities/events.
- `add_wireframe_step` — a UI step; triggers a command or displays a read model.
- `add_flow` — an ordered behaviour assembling the above into a timeline.

**Inspect & validate**
- `open_questions` — the facilitation driver: returns the SHACL gaps as questions for the room.
- `query` — read the graph (entity's relations, a context's contents, "what happens to X?").
- `validate` — run all shapes; return conformance + violations.

Every mutating tool returns `{ ok, node, violations[] }`. A non-empty `violations` means the
fragment was rejected (or accepted-with-warnings, for non-blocking gaps) — the model reads the
framework-section messages and corrects.

---

## 5. Output and provenance

`session_finalize` emits:

- **The conformant What graph** as Turtle, validated against `shapes.shacl.ttl` — entities,
  relations, value objects, invariants, contexts, mappings, commands, events, read models,
  flows, and the links between them.
- **A provenance record**: who was in the room, when, a content hash of the graph, and the
  tool-call log (the derivation of the model from the conversation). This is what lets the
  graph be trusted as an authored artifact, and what a later `product author feature` session
  reads as its starting point.

The graph is the substrate for everything downstream — the archetype is built to realise it,
cells are parameterised by its concepts, audits check code against it, features partition it.

---

## 6. The acceptance test — does it match reality?

The "6 people, 60 minutes" claim is a falsifiable experiment; treat it like the cell-dispatch
protocol. Run it on a system your team **already shipped and knows well**, then judge the
output not by "did we get a graph" but by "is the graph *true*":

| Measure | Threshold |
|---|---|
| **Coverage** | the major bounded contexts and the core flows are all present |
| **Accuracy** | relations, cardinalities, and context mappings match the real system; someone who knows the system recognises it as correct |
| **Time** | ≤ 60 minutes of room time to a finalized, conformant graph |
| **Conformance** | the output passes `shapes.shacl.ttl` with zero violations |

The failure mode to watch for: a graph that is **plausible but wrong** — confident fiction. A
wrong domain model that *looks* authoritative is more dangerous than no model, because
everything downstream inherits the error. So accuracy is judged by a person who knows the
system, not by the model's confidence. If the output is plausible-but-wrong, the fix is in the
facilitation and the `open_questions` prompts, not the model — the same "improve the
upstream, not the model" discipline as the funnel.

---

## 7. Scope boundaries (what this build is not)

- **No How.** It captures what the system is and does, not how code expresses it.
- **No cells, no audits, no delivery.** Those are later builds against this graph.
- **It models the interesting behaviour, not every CRUD triviality** (framework §3.2 depth
  rule) — the facilitator steers the event-model phase to core flows.
- **It does not prove the delivery economics** (§ the caution in the header).

---

## 8. Build checklist

1. The MCP server implementing the tools in `product-author-domain.tools.json`, each
   constructing graph fragments against the ontology and validating with `pyshacl` against
   `shapes.shacl.ttl` in-loop.
2. `product author domain` as the CLI entry that launches a session and hosts the MCP server
   for an LLM client.
3. `open_questions` wired to the SHACL report so incompleteness drives facilitation.
4. `session_finalize` emitting Turtle + provenance.
5. The acceptance test (§6) run on one known system before any customer session.
