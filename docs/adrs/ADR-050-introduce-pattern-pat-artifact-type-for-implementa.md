---
id: ADR-050
title: Introduce Pattern (PAT) Artifact Type for Implementation Knowledge
status: accepted
features:
- FT-070
- FT-071
- FT-072
- FT-073
- FT-074
- FT-075
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: cross-cutting
content-hash: sha256:1866c18be0da62755e1020aad5c78143b197ad55fd5950d61475c3ffee42206a
---

**Status:** Proposed

**Context:** The product-cli graph stores four kinds of artifact today —
features (FT), architectural decisions (ADR), test criteria (TC), and
dependencies (DEP). Together they answer *what* is being built, *why* it
is being built that way, *whether* it works, and *what external systems*
it relies on. None of them answers the question every implementing agent
asks next: *how* should I build this, in this codebase?

That "how" knowledge exists. It is scattered across:

- `CLAUDE.md` sections such as "Architecture Pattern — Slice + Adapter",
  "TC Runner Configuration", and "Adding a New Command".
- Oral tradition embedded in past features — for example, the
  FT-046 → FT-066 series quietly established that an MCP tool which
  advertises a write must dispatch through the same slice the CLI uses,
  not return a placeholder envelope.
- The `conventions/` directory, which captures *enforced* policies but
  is not the right home for *guidance* (a convention is something
  checked; a pattern is something taught).
- Reviewer judgement, which is by definition not in the graph.

The cost of this scatter is observable. Every feature in the FT-046
through FT-069 sequence rediscovered the slice + adapter shape from first
principles. Several authored MCP handler stubs that returned the right
envelope shape against a no-op, because the prevailing pattern ("MCP tool
with disk side-effect") was not declared anywhere a planning agent would
read. FT-066 had to fix the same class of bug FT-046 had already named in
its "Out of scope" list — twice.

The fix is to make the "how" a first-class node in the graph: a Pattern
artifact (PAT-XXX) that captures reusable implementation guidance, cites
the ADRs it operationalises, declares prerequisite patterns, and is
linked from features the same way ADRs and TCs are.

Three alternative homes for the "how" knowledge were considered before
deciding on a new artifact type.

1. **Extend ADRs.** Add a "Pattern" status or a parallel ADR sub-type.
   Rejected on three grounds:
   - **Lifecycle mismatch.** ADRs evolve by supersession (audit trail,
     bidirectional `superseded-by` links, content-hash immutability per
     ADR-032). Patterns evolve by accretion — a slice + adapter pattern
     gains a worked example with each new feature without superseding the
     previous version. Folding two lifecycles into one artifact muddies
     both.
   - **Centrality muddying.** Betweenness centrality over a homogeneous
     ADR population surfaces architecturally pivotal decisions (ADR-013,
     ADR-020, ADR-043 today). Pattern citations are much higher in
     volume than ADR citations — every implemented feature would cite
     the same three or four patterns. The centrality signal would
     collapse onto the patterns and lose its diagnostic value for ADRs.
   - **Different content shape.** ADRs explain *why a path was chosen*
     and what was rejected. Patterns demonstrate *concrete code* with
     anti-patterns. The two content shapes co-exist in one file only by
     compromising both.

2. **Use a new TC type.** Make patterns a flavour of `descriptive` TC
   (per ADR-042). Rejected because TCs verify *instances of correct
   behaviour*; patterns are *templates for producing correct behaviour*.
   A TC declaring "the slice + adapter pattern is used here" is a
   category error — there is nothing to assert against the test runner.
   Reusing the TC type for templates also breaks the verify pipeline's
   assumption that every TC has a runner (FT-058, ADR-021).

3. **Keep patterns in `conventions/`, treating them as enforced policies.**
   The `conventions/` directory is the home for things checked by
   `tests/code_quality_tests.rs` (file length, doc-comment SRP). A
   pattern is *not* enforced — it is *taught*. The two have different
   semantics: a convention fails the build when violated; a pattern is
   ignored when a feature genuinely doesn't apply it. Eventually the two
   may converge (a pattern with an `enforced-by:` link becomes a
   convention), but that unification is a separate design and does not
   block introducing PAT.

4. **Skill-only (in `~/.claude/skills/` or similar).** Rejected because
   it excludes non-Claude agents and, more fundamentally, fails the
   product-cli dogfood test: "knowledge that an LLM needs to implement
   features belongs in the graph". Putting patterns in skills makes
   them invisible to `product context`, `product impact`, and the
   centrality view that surfaces architecturally important nodes.

---

**Decision:** Introduce **PAT-XXX** as a new artifact type, peer to
FT / ADR / TC / DEP. Patterns capture reusable implementation knowledge
("how to build a thing of this shape in this codebase") and integrate
with every existing graph operation: parse, link, context bundle,
impact, centrality, drift check.

### Front-matter schema

```yaml
---
id: PAT-001                        # Required. Format: PAT-NNN
title: String                      # Required
status: Enum                       # Default: live. Values: live | deprecated
domains: [String]                  # Default: []. Concern domains
adrs: [String]                     # Default: []. ADRs this pattern operationalises
requires: [String]                 # Default: []. Prerequisite PAT IDs (DAG)
examples: [String]                 # Default: []. Feature IDs that exemplify it
deprecated-by: String              # Optional. Only when status: deprecated
---
```

### Required body sections (validated structurally, mirroring FT-055 / W030)

- `## When to use` — one-sentence trigger.
- `## Prerequisites` — environmental or skill prerequisites in prose.
- `## The pattern` — concrete code or structural sketch.
- `## Anti-patterns` — named cases of what not to do.
- `## Worked example` — references to real features (the `examples:`
  list, expanded).

A new validator code (introduced by F2 in the implementing feature
cluster) emits a warning when a live PAT is missing a required section,
mirroring W030 for features.

### Lifecycle

- **`live`** — the pattern is current. New features may cite it.
- **`deprecated`** — the pattern is superseded by another approach.
  The optional `deprecated-by:` field points at the replacement. A
  warning fires when a live (non-abandoned) feature cites a deprecated
  pattern.
- There is **no supersession audit trail** (no `supersedes:` /
  `superseded-by:` chain, no content-hash immutability, no amendments).
  Patterns accrete: the body grows worked examples and the
  anti-patterns list lengthens as the codebase learns. The atomic-write
  invariants of ADR-038 still apply for any single update.

### Graph operations

- **`requires:`** forms a DAG. `product graph check` validates acyclicity
  (a new error code, allocated by F2). Topo order over `requires:` is
  the canonical traversal order when assembling a context bundle that
  includes multiple patterns.
- **`examples:`** is bidirectionally materialised against
  `feature.patterns:` by the request-apply pipeline (ADR-038 batching),
  mirroring the FT-066 fix for feature ↔ TC reciprocation.
- **`product context FT-XXX --depth N`** walks `feature.patterns` and
  emits each PAT in topo order following `requires:`. The bundle layout
  remains feature → ADRs → TCs → patterns (or feature → patterns →
  TCs → ADRs — the precise ordering is a presentation decision left to
  F5).
- **`product impact PAT-XXX`** traverses reverse edges to report every
  feature, ADR, and downstream pattern affected by a change to this
  pattern.
- **Patterns participate in Brandes centrality.** Highly-cited patterns
  surface as architecturally pivotal nodes alongside high-centrality
  ADRs.

### CLI and MCP surface

The new CRUD surface follows the established conventions:

- CLI: `product pattern new|show|list|link|status` (and the request
  interface accepts `type: pattern` records).
- MCP: `product_pattern_new`, `product_pattern_show`, `product_pattern_list`,
  `product_pattern_link`, `product_pattern_status`. All write tools
  route through the slice (per the FT-066 lesson — no envelope-only
  stubs).

### Repository layout

- `[paths].patterns = "docs/patterns"` (new entry in `product.toml`).
- `[prefixes].pattern = "PAT"` (new entry).

---

⟦Γ:Invariants⟧{
  every_pattern_carries_a_unique_PAT_id_in_front_matter
  no_pattern_requires_itself_directly_or_transitively
  every_pattern_examples_reference_resolves_to_a_real_feature_id
  every_feature_patterns_reference_resolves_to_a_real_pattern_id
  for_every_live_feature_citing_a_pattern_the_pattern_is_not_deprecated_or_a_warning_is_emitted
  pattern_status_is_one_of_live_or_deprecated_no_other_values_accepted
  no_pattern_has_both_status_deprecated_and_an_empty_deprecated_by_field_warned
  patterns_appear_in_product_context_output_for_every_feature_that_cites_them
  patterns_appear_in_product_impact_output_for_every_artifact_they_govern
  the_pattern_artifact_lifecycle_does_not_use_supersedes_or_superseded_by_fields
}

⟦Ε⟧⟨δ≜1.0;φ≜100;τ≜◊⁺⟩

---

**Rationale:**

- **Co-location matches the actual reading order.** An agent given a
  feature to implement asks, in sequence: what is this? (FT body), why
  was it decided this way? (ADRs), how should I write it? (patterns),
  how do I know I'm done? (TCs). Patterns slot into the existing chain
  at the position where the question is already being asked.

- **Topo-sorted `requires:` falls out of the existing toolkit.** The
  graph already does cycle detection (ADR-012, FT-016) and topo sort
  (the feature `depends-on` chain). Patterns reuse this infrastructure
  rather than inventing a per-feature ordering.

- **Accretion is the correct lifecycle.** A pattern improves as more
  features exemplify it. Supersession is the wrong shape — the previous
  worked examples don't become wrong just because a new one is added.
  When the pattern itself genuinely changes (e.g. slice + adapter
  becomes slice + adapter + something), the simpler `deprecated` →
  `live` flip with a `deprecated-by:` pointer captures the change
  without an audit trail nobody reads.

- **Centrality stays honest.** Keeping PAT a separate population means
  Brandes still surfaces foundational ADRs unmolested. PAT centrality
  is computed in the same algorithm but reported on its own axis.

- **Dogfood-able.** The seed catalog (PAT-001 slice + adapter; PAT-002
  MCP tool with disk side-effect; PAT-003 TC authoring observability)
  proves the schema can express the patterns already in oral tradition.
  If any of the three needs a field this ADR doesn't grant, the schema
  is wrong and gets fixed before the seed catalog lands.

**Rejected alternatives:**

- **Extend ADRs** — rejected above (lifecycle, centrality, content
  shape mismatch).
- **New TC type** — rejected above (templates ≠ verifications).
- **Conventions-only** — rejected above (enforcement vs. guidance).
- **Skills-only** — rejected above (excludes non-Claude agents and
  fails the in-graph dogfood test).
- **Single "knowledge" artifact unifying ADR + PAT + convention.**
  Rejected as premature. Each of the three has a distinct lifecycle
  today; collapsing them would require defining the union of all three
  lifecycles in a single state machine. Revisit only if a fourth
  artifact wants to join the group; until then, three nodes with three
  clear roles beats one node with three sub-modes.
- **Pattern versioning beyond `live | deprecated`.** Rejected as
  premature. Supersession audit trails exist on ADRs because
  accountability for decisions matters; patterns are guidance, and
  guidance does not need a paper trail. Add only if a concrete need
  surfaces.
- **Cross-project pattern sharing (publishing patterns as packages).**
  Out of scope. Each project's patterns reflect its own codebase
  shape; cross-pollination is a separate, speculative design.

**Test coverage:** Validated by the feature cluster that implements
this decision. F1 (Pattern artifact: parse, schema, CRUD) ships
scenario TCs for file creation with required sections, request-apply
roundtrip, and bidirectional `examples:` ↔ `feature.patterns:`
materialisation. F2 (Pattern in graph algorithms) ships scenario TCs
for context bundle ordering, impact traversal, and centrality
participation. F6 (seed catalog) is the dogfood test — three real
patterns expressed against the schema. Every TC in the cluster
declares its `observes:` surface explicitly per ADR-051, so the
verification chain itself demonstrates the second half of the lesson
that motivated this decision.
