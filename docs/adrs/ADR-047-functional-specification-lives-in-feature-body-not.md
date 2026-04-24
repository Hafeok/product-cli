---
id: ADR-047
title: Functional Specification Lives in Feature Body, Not a Separate Artifact
status: proposed
features:
- FT-055
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: cross-cutting
source-files:
- docs/product-functional-spec.md
- src/config.rs
- src/feature/body_sections.rs
- src/graph/functional_spec_validation.rs
- src/graph/validation.rs
---

**Status:** Proposed

**Context:** An LLM reading a feature context bundle needs more than a capability title and a list of decisions — it needs a precise, structural description of what the feature does, so that "implement FT-NNN" produces a result that matches what the team expects without guessing. Today the feature body carries a free-form `## Description` and some ad-hoc prose; the surrounding ADRs explain *why* the feature exists but do not pin the behavioural contract *what* the implementation must satisfy.

Three alternatives were considered for where the behavioural contract should live.

1. **New `SPEC-XXX` artifact type.** Every feature gets an accompanying spec node in the graph. Rejected because it creates an extra node the agent must traverse, introduces "ADR vs SPEC" confusion about where decisions live, and would require context bundles to expand to include SPEC artifacts. The feature is where agents already look first — adding a parallel artifact doubles the navigational cost for the same information.

2. **Spec expressed as a set of exit-criteria TCs.** TCs already verify specific behavioural points. Rejected because TCs verify *instances*; a functional specification defines the *space* of correct behaviour. A spec expressed as a hundred small TCs is harder for an LLM to reason about than one structured document with TCs pointing to specific clauses. The two shapes serve different purposes — one describes the contract, the other verifies points on it.

3. **Structured section on the feature.** Extend the feature body with a fixed-structure `## Functional Specification` section and a sibling `## Out of scope` section. Accepted. The feature body already carries prose description; adding a conventional structure uses existing infrastructure, keeps related information together (description + spec + out-of-scope all in one place), and appears in every context bundle for that feature without any extra graph traversal.

The decision to extend rather than proliferate is also consistent with ADR-002 (YAML front-matter as source of truth) and ADR-006 (context bundle as the primary LLM interface): both decisions push for fewer, self-describing artifacts over more, cross-referenced ones.

---

**Decision:** The functional specification for a feature lives inside that feature's markdown body, structured as an H2 section `## Functional Specification` with seven fixed H3 subsections, plus a sibling `## Out of scope` H2 section. There is no parallel `SPEC-XXX` artifact. `product graph check` validates structural completeness and emits a new warning code, W030, when required sections are absent.

---

### Required Structure

Every non-stub feature (phase ≥ `[features].required-from-phase`, default 1) must contain:

- `## Description` (H2) — one-paragraph summary of the feature. Already present.
- `## Functional Specification` (H2) — container for the contract.
  - `### Inputs` (H3) — every input the feature accepts.
  - `### Outputs` (H3) — every output path including success and failure.
  - `### State` (H3) — what the feature remembers between requests; "stateless" if it doesn't.
  - `### Behaviour` (H3) — numbered/ordered steps for the core algorithm.
  - `### Invariants` (H3) — properties that must hold at all times.
  - `### Error handling` (H3) — what happens on failure and how it's signalled.
  - `### Boundaries` (H3) — edge cases and boundary conditions.
- `## Out of scope` (H2) — what this feature explicitly does not do. Sibling of Functional Specification, not nested underneath, because it scopes the whole feature, not just its behaviour.

The subsection list is configurable via `[features].functional-spec-subsections` in `product.toml`, but the default structure is the one above and teams should only adjust it deliberately.

### W030 — Missing required section

`product graph check` parses each feature body looking for the configured H2 and H3 headings. A missing section emits:

```
warning[W030]: feature body missing required section
  --> docs/features/FT-009-rate-limiting.md
   |   FT-009: Rate Limiting
   |   Missing sections:
   |     - Functional Specification > Behaviour
   |     - Functional Specification > Boundaries
   |     - Out of scope
   = hint: add with `product request change`, op: set, field: body
```

W030 is advisory by default (warning tier, exit code 2 when no errors). Promotable to error-tier via `[features].completeness-severity = "error"`; when set, W030 becomes E-class and blocks the `planned → in-progress` status transition for the feature. Teams that want spec-before-implementation opt in via this flag; teams still migrating their existing features keep the default warning tier so CI stays green.

### Empty-meaning subsections are valid

A feature that is genuinely stateless may write:

```markdown
### State
  Stateless. No data is retained between requests.
```

This satisfies W030 — the section is present with content that explicitly declares the concept doesn't apply. What does *not* satisfy W030 is an absent section or a section containing only whitespace. The distinction matters: an LLM reading "Stateless" knows the feature designer considered state and declared it empty; an LLM reading no section at all doesn't know whether state was forgotten or deliberately omitted.

### Context bundle integration

No new bundle assembly logic is needed. `product context FT-NNN --depth 2` already includes the full feature body. The spec appears in every bundle because the spec is part of the body. This is the central design benefit: an LLM calling `product_context` via MCP receives the full implementation contract in the response it already requested, with no extra graph traversal and no extra tool call.

### Relationship to TCs and ADRs

TCs verify specific points on the functional specification. A TC body may reference the subsection it validates as a convention (not enforced): "Verifies Functional Specification / Behaviour step 7". ADRs explain *why* decisions were made; the functional spec explains *what* behaviour those decisions produce. They are orthogonal and an LLM implementing the feature needs both — the ADR for decision context, the spec for the behavioural contract, the TCs for verification points.

---

**Rationale:**

- **Cognitive locality.** The description, the contract, and the out-of-scope items all answer questions a reader asks in sequence: "what is this?", "what does it do?", "what does it deliberately not do?". Placing them adjacently in a single file matches the reading order and minimises context-switching.

- **No graph expansion.** Adding a `SPEC-XXX` artifact would introduce a new edge type, a new validator path, a new schema section, new MCP tools, and new context-bundle assembly rules. All of that is zero when the spec is a markdown section — it inherits every behaviour the feature already has.

- **LLM failure mode prevention.** LLMs pattern-match helpfully. Given "rate limiting", an LLM may implement endpoint-specific limits, allow-lists, and distributed state unless told not to. The `## Out of scope` section is explicitly for preventing this pattern — it lists what the LLM must *not* add. This is why it is a mandatory section, not an optional appendix.

- **Structural completeness is cheap to check.** W030 is a heading-presence check, not a semantic check. It runs in milliseconds against any feature body. No LLM is involved. This means the completeness gate can run in every `product graph check` invocation including CI, without cost.

- **Opt-in enforcement.** Defaulting W030 to warning (not error) lets teams adopt the structure incrementally. Teams writing specs before implementation flip `completeness-severity = "error"` and get enforcement; teams mid-migration leave the default and get advisory signals without blocking merges.

**Rejected alternatives:**

- **`SPEC-XXX` as a separate artifact type** — rejected above. Creates navigational cost for information that co-locates naturally with the feature.

- **Express the spec as a structured appendix inside ADRs** — would split the behavioural contract across every ADR linked to a feature, defeating the "one place to look" property. Also breaks cleanly-superseded ADRs: a spec clause inside a superseded ADR becomes confusing (is the clause still active?).

- **A single free-form "Functional Spec" subsection with no mandatory structure** — rejected because the structure is the point. The seven subsections (Inputs / Outputs / State / Behaviour / Invariants / Error handling / Boundaries) are the checklist an LLM needs to produce correct code. A free-form spec is indistinguishable from the existing `## Description` and does not close the "will the LLM remember to describe Boundaries?" failure mode.

- **Semantic checking via LLM** — rejected because it conflates structure with content. Structural completeness (a section is present) is a deterministic check and should be deterministic. Content quality (the section is *good*) is an LLM judgement and belongs in `product gap check`, not `product graph check`.

**Test coverage:** Session tests ST-340–ST-355 in `docs/product-functional-spec.md` cover: section detection (ST-340–ST-345), completeness severity (ST-346–ST-348), empty-section handling (ST-349–ST-351), context-bundle preservation (ST-352–ST-353), and configuration (ST-354–ST-355). These are tracked as TC-665 through TC-680 under FT-055.
