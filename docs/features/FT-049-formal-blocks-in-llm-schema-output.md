---
id: FT-049
title: Formal Blocks in LLM Schema Output
phase: 5
status: complete
depends-on:
- FT-033
- FT-048
adrs:
- ADR-011
- ADR-016
- ADR-031
tests:
- TC-617
- TC-618
- TC-619
domains:
- api
domains-acknowledged:
  ADR-043: Implementation adds functions to the existing `src/agent_context/schema.rs` pure module; no new slice or command adapter is introduced.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  ADR-042: Consumed unchanged — the structural / built-in-descriptive / custom partition from ADR-042 is the source of the "required by" annotations; this feature documents but does not alter the partition.
  ADR-040: Schema render is a read-only documentation surface; it does not participate in the verify pipeline stages and adds no hooks to the LLM boundary beyond the existing agent-context bundle.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-041: The feature documents which tc-types require formal blocks; it does not interact with absence TCs or ADR removes/deprecates lifecycle.
---

## Description

`product schema` and the `agent-init` / `agent-context` bundles tell an LLM
how to write Product's artifact front-matter (feature, ADR, TC, dep), but
they say nothing about the **AISP formal block grammar** that `type:
invariant` and `type: chaos` TCs must carry (W004), or that exit-criteria
TCs use to record the `⟦Λ:ExitCriteria⟧` enumeration. The LLM is left to
infer the notation from examples, which is brittle.

This feature closes the loop: the schema output grows a fifth section —
**Formal Blocks** — that documents each AISP block (`⟦Σ:Types⟧`,
`⟦Γ:Invariants⟧`, `⟦Λ:Scenario⟧`, `⟦Λ:ExitCriteria⟧`, `⟦Ε⟧⟨…⟩`),
its fence syntax, its required fields, and which `tc-type` values require
which block. The existing TC schema cross-references the new section so a
reader hitting `type: invariant` immediately knows to look at the formal
block spec.

Originates from GitHub issue #4 ("Formal blocks not in schema for LLM").

---

## Depends on

- **FT-033** — Agent Context Generation. The schema render is the surface
  this feature extends; `agent-init` embeds it verbatim into AGENT.md.
- **FT-048** — TC Type System. The `invariant` / `chaos` / `exit-criteria`
  structural types are the ones that require formal blocks, so the new
  schema section must keep the same vocabulary.

---

## Scope of this feature

### In

1. **`generate_all_schemas` / `generate_all_schemas_with_config`** emit a
   new top-level `## Formal Blocks` section after `## Dependency`. Content:
   AISP block syntax (delimiters `⟦` / `⟧`), one sub-section per block type
   with a minimal example and field list, and a "required by" line stating
   which `tc-type` values mandate which block (W004 contract).
2. **`generate_schema("formal")`** returns just the formal-block section so
   tooling can fetch it in isolation (mirrors `generate_schema("feature")`).
3. **TC schema cross-reference.** `test_schema_with_config` gets one extra
   comment line after the `type:` line pointing at the formal block section
   ("see Formal Blocks for invariant / chaos / exit-criteria notation").
4. **AGENT.md regeneration.** `product agent-init` re-emits AGENT.md so the
   new section appears for projects that re-bootstrap.
5. **Unit tests** on `generate_all_schemas` asserting the formal block
   section is present with the five AISP block names.

### Out

- **Teaching the formal grammar beyond what Product enforces.** The schema
  documents the block delimiters, fence syntax, and which `tc-type` needs
  which block. It does not document optimisation heuristics, symbolic
  execution semantics, or the broader AISP paper. A link to the ADR-016
  glossary suffices.
- **Validating formal block *content*.** Parser-level validation is
  FT-011 territory; this feature only adds documentation.
- **A new `formal-block` artifact type.** Formal blocks remain embedded in
  TC bodies; they are not top-level artifacts with their own schema.

---

## Commands

No new CLI subcommands. The feature surfaces through:

- `product schema` (both the default "all" render and `--type formal`).
- `product agent-init` / `product agent-context` (AGENT.md includes the new
  section automatically).

---

## Implementation notes

- **`src/agent_context/schema.rs`** — add `formal_block_schema()` returning
  the section body. Extend `generate_schema` and `generate_schema_with_config`
  to accept `"formal"`. Extend `generate_all_schemas` and
  `generate_all_schemas_with_config` to append the section.
- **TC schema cross-reference.** Add one comment line inside
  `test_schema_with_config` after the custom-type line pointing at
  `## Formal Blocks`. Keep within the 400-line file-length budget.
- **No new dependencies.** Pure string templating, matches the existing
  style.
- **Source of truth for the block list.** The five AISP blocks
  (`Σ:Types`, `Γ:Invariants`, `Λ:Scenario`, `Ξ:Exit-Criteria`, `Φ:Evidence`)
  are already enumerated in `src/formal/blocks.rs::FormalBlock`. The schema
  text stays in sync with that enum by eyeball; a future refactor could
  `impl Display` on the variants to derive the text, but that is out of
  scope.
- **Tests.** One unit test per block name asserting it appears in the
  rendered schema, plus one integration test that invokes
  `product schema` and greps for `## Formal Blocks`.

---

## Acceptance criteria

A developer running on a clean repository can:

1. Run `product schema` and observe a `## Formal Blocks` section after
   `## Dependency`, listing the five AISP blocks with examples (TC-617).
2. Run `product schema --type formal` and get just the formal-block section
   (TC-618).
3. Read the TC schema block and see a cross-reference line pointing at the
   formal-block section (TC-617).
4. Author a new `type: invariant` TC guided solely by `product schema` +
   `product agent-init` output, and pass `product graph check` with no W004
   warnings — the schema taught the LLM how to write `⟦Γ:Invariants⟧`
   (TC-619, exit).
5. Run `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` and observe all pass.

See TC-619 (exit criteria) for the consolidated check-list.

---

## Follow-on work

- **Derive schema text from `FormalBlock` enum.** A `Display` or `schema()`
  method on each variant would keep the two in lock-step. Deferred; current
  manual sync is fine at the current block count.
- **Live examples from repo TCs.** The schema could pick one real TC per
  block and embed its body fragment as the example. Nice-to-have.

---

## Functional Specification

### Inputs

- `product schema` — invoked with no arguments (renders all schema sections) or with `--type formal` (renders the formal-block section only).
- `product schema --type <name>` — the `"formal"` type name is the new selector added by this feature; all existing type names (`"feature"`, `"adr"`, `"test"`, `"dep"`) continue to work unchanged.
- `product agent-init` — regenerates `AGENT.md`; the new schema section is included automatically.
- `product agent-context` — includes the updated schema in the bundle.
- No additional runtime inputs; the formal block catalogue is a static string derived from the five AISP block types defined in `src/formal/blocks.rs::FormalBlock`.

### Outputs

- **`product schema` (all):** adds a new top-level `## Formal Blocks` section after `## Dependency`. The section documents: AISP block delimiters (`⟦` / `⟧`), fence syntax, one sub-section per block type (`Σ:Types`, `Γ:Invariants`, `Λ:Scenario`, `Ξ:Exit-Criteria`, `Φ:Evidence`) with a minimal example and field list, and a "required by" line stating which `tc-type` values mandate which block (the W004 contract).
- **`product schema --type formal`:** returns just the formal-block section, matching the pattern of `product schema --type feature` for individual-section access.
- **TC schema block (within `product schema`):** one additional comment line after the `type:` field pointing at the `## Formal Blocks` section ("see Formal Blocks for invariant / chaos / exit-criteria notation").
- **`AGENT.md` (regenerated by `product agent-init`):** includes the new formal-block section, making it available to LLMs that read AGENT.md for schema guidance.

### State

Stateless. The schema output is a pure function of the hard-coded block catalogue in `src/agent_context/schema.rs` and the `FormalBlock` enum in `src/formal/blocks.rs`. No runtime state is read or written; the output is identical across invocations with the same binary.

### Behaviour

1. **`generate_all_schemas` / `generate_all_schemas_with_config`** append the formal-block section after the dependency schema section. The section is built by `formal_block_schema()`, a new pure function in `src/agent_context/schema.rs`.
2. **`generate_schema("formal")` / `generate_schema_with_config("formal", ...)`** return just the formal-block section. The `"formal"` key is added alongside the existing per-artifact-type keys.
3. **TC schema cross-reference.** `test_schema_with_config` is extended with one comment line after the custom-type description line: `# See '## Formal Blocks' for invariant / chaos / exit-criteria notation`. This line is within the 400-line file-length budget (ADR-029).
4. **`product agent-init` regeneration.** Because `agent-init` embeds the full schema output verbatim into `AGENT.md`, re-running `product agent-init` on any project automatically includes the formal-block section. No separate migration step is required.
5. **Block list source of truth.** The five AISP blocks (`Σ:Types`, `Γ:Invariants`, `Λ:Scenario`, `Ξ:Exit-Criteria`, `Φ:Evidence`) are catalogued in `src/formal/blocks.rs::FormalBlock`. The schema text in `formal_block_schema()` is kept in sync with that enum by eyeball; the schema text is not derived automatically from the enum variants.

### Invariants

- `product schema --type formal` returns a non-empty string containing all five AISP block names.
- `product schema` (all) contains `## Formal Blocks` as a top-level section appearing after `## Dependency`.
- The TC schema block contains a cross-reference line pointing at `## Formal Blocks`.
- The formal-block section is identical whether accessed via `product schema` (all) or `product schema --type formal`.
- No new dependencies are introduced; the implementation is pure string templating.
- All files in `src/agent_context/` remain under 400 lines (ADR-029 / code-quality fitness test).

### Error handling

- `product schema --type <unknown>` — the existing behaviour for unknown type names is unchanged (error message listing valid type names, now including `"formal"`).
- No runtime errors are introduced; all schema generation is pure string computation with no I/O beyond the existing `product schema` output path.

### Boundaries

- The formal-block schema documents the grammar that Product enforces (block delimiters, fence syntax, which `tc-type` mandates which block). It does not document AISP optimisation heuristics, symbolic execution semantics, or the broader AISP paper beyond a link to the ADR-016 glossary.
- Formal block *content* validation (checking that an `invariant` TC's `⟦Γ:Invariants⟧` block is syntactically valid) is FT-011 territory; this feature only adds documentation.
- A new `formal-block` artifact type is not introduced; formal blocks remain embedded in TC bodies.
- The schema text is not automatically derived from the `FormalBlock` enum; manual sync is required when new blocks are added.

## Out of scope

- Documentation of AISP optimisation heuristics, symbolic execution semantics, or the broader AISP theoretical framework: the schema covers only the grammar that Product enforces, with a link to ADR-016 for further reading.
- Validation of formal block content (syntactic correctness of `⟦Γ:Invariants⟧` etc.): this is FT-011 territory; this feature only adds schema documentation.
- A new `formal-block` artifact type with its own schema section: formal blocks remain embedded in TC bodies.
- Automatic derivation of schema text from the `FormalBlock` enum: the current manual sync is sufficient at the current block count. A future refactor may add `impl Display` or `schema()` to each variant.
