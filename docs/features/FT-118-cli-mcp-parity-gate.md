---
id: FT-118
title: CLI↔MCP parity gate — every command exposes an MCP tool or is classified
phase: 6
status: complete
depends-on: []
adrs:
- ADR-060
tests:
- TC-980
domains:
- api
domains-acknowledged:
  ADR-041: Additive feature — a new fitness gate; no CLI surface, MCP tool, or schema field is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: No new implementation pattern; this is a fitness test (ADR-029 family).
  ADR-049: No context bundle or template change.
  ADR-043: The gate is a self-contained test; there is no slice/adapter split to make.
  ADR-048: Reads source under product-cli/ and product-mcp/; writes nothing.
  ADR-051: TC-980 observes the exit-code surface of the fitness test and asserts on the reported command list.
  ADR-018: A single fitness test in the code_quality_tests binary (the ADR-029 family); no property/session dimension for a static-source gate.
  ADR-040: The gate is a structural check over the CLI and MCP source; it crosses no LLM boundary.
patterns:
- PAT-001
---

## Description

A recurring defect: a new CLI command family is added without exposing the same
functionality as an MCP tool, so agents driving the product through MCP lose
access to it. FT-118 makes the convention enforceable — a fitness gate that
fails when a command is neither MCP-exposed nor explicitly classified.

Dogfooding the framework surfaced the gap concretely: the new
What/How/cell/archetype/work-unit families (and the older `dep`) ship CLI-only.
The rule is also recorded as the `cli-mcp-parity` principle in product-cli's own
How archetype, `enforced_by` this gate.

## Functional Specification

### Inputs

- The top-level CLI command names (parsed from `commands/root_enum.rs`).
- The MCP tool names (parsed from the registry dispatch in
  `product-mcp/src/registry.rs`).

### Behaviour

- Each command is classified into exactly one of: **MCP-exposed** (a
  `product_<cmd>_*` tool exists), **CLI-only** (an explicit allowlist of
  process/launch/meta commands), or **pending-MCP** (an explicit documented
  debt list).
- A command in none of the three fails the gate, naming it and instructing the
  author to add a tool or classify it.
- The debt list is self-cleaning: an entry that has since gained a tool fails
  the gate (must be removed), and a stale non-command entry fails too.

### Error handling

- The test fails with the list of unclassified commands.
- The test fails if a pending-MCP entry already has a tool, or if any
  classification entry is not a real command.

## Out of scope

- It does not itself add the missing MCP tools — it drives them down via the
  debt list; exposing the pending families is follow-on work.
- It checks command-family-level parity by name, not per-subcommand verb
  coverage.

## Acceptance

- TC-980 — every command is MCP-exposed or classified; the debt list is
  self-cleaning; emptying the debt list flags exactly the unexposed families.
