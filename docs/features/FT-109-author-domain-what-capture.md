---
id: FT-109
title: product author domain — facilitated What-capture MCP session
phase: 6
status: complete
depends-on: []
adrs:
- ADR-053
tests:
- TC-896
- TC-897
- TC-898
- TC-899
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new subcommand and a new MCP server; no existing CLI surface, MCP tool, schema field, or behaviour is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: No context bundle or template change; the command does not render bundles.
  ADR-043: Followed — a pure `pf` slice in product-core (model, validate, turtle, seed, session, ops) with thin MCP/CLI adapters that own the I/O and process spawn.
  ADR-048: The domain session writes only under its own session directory (`.product/author-domain/<product>/`); no FT/ADR/TC state files are touched.
  ADR-051: All four TCs declare `observes:` (exit-code, stdout) and their bodies assert on those named surfaces.
  ADR-018: Four scenario TCs drive the binary through the assert_cmd harness; the slice's pure functions and the MCP registry carry unit tests. No property or session dimension for a workshop reporter.
  ADR-040: The captured graph is a What artifact; the in-loop checker is structural (SHACL mirror), and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product author domain` is a facilitated, MCP-driven session that captures a
product's *What* — the domain model (structure) and event model (behaviour)
of the Product Framework — as a conformant RDF graph, in about an hour. It is
the sibling of `product author feature`: the same machinery (MCP server,
structured ops, in-loop SHACL, hashed provenance) pointed at the What layer,
which must exist before delivery can be partitioned against it.

The session is a facilitated harvest. An LLM holds the MCP server as scribe
and turns conversation into a validated graph in real time. The magic is the
model writing the graph from conversation; the discipline is that every fact
enters through a **structured graph operation**, never raw Turtle, and each
mutating tool validates its fragment in-loop against the framework's SHACL
shapes — so the model cannot produce a non-conformant graph.

The build is deliberately the cheapest, lowest-dependency piece of the
framework toolchain: it proves the What can be harvested cheaply and produces
the substrate every later build needs. The full design rationale (separate
graph, native conformance mirror, JSON session) is ADR-053.

## Functional Specification

### Inputs

- A product identifier (positional) whose What is being captured.
- Optionally, a prior session's Turtle export to seed from (`--seed`).
- The 17-tool MCP surface defined by `product-author-domain.tools.json`:
  session (`session_start`, `session_state`, `session_finalize`); structure
  (`add_bounded_context`, `add_entity`, `add_value_object`, `add_relation`,
  `add_invariant`, `add_context_mapping`); behaviour (`add_command`,
  `add_event`, `add_read_model`, `add_wireframe_step`, `add_flow`); and
  inspect (`open_questions`, `query`, `validate`).

### Behaviour

- `product author domain <product>` launches the configured agent CLI with
  the facilitation prompt and the domain MCP server wired in.
- `product author domain <product> --serve --session-dir <dir>` hosts the
  domain MCP server over stdio (the form the agent's MCP config invokes).
- `product author domain <product> --print-prompt` emits the facilitation
  prompt and exits (the deterministic, scriptable path).
- Each mutating tool returns `{ ok, node, violations[] }`. A blocking
  violation reverts the fragment; the message names the framework section
  (e.g. "§3.2 Every event must change a real domain entity") so the model
  self-corrects in the loop.
- `open_questions` returns the current SHACL gaps plus softer completeness
  prompts (an empty context, an aggregate with no commands, contexts that
  should map but don't), optionally limited to `structure` or `behaviour`.
- `session_finalize` runs full validation; if conformant it exports the What
  graph as Turtle (validatable against `schema/shapes/shapes.shacl.ttl`) and
  writes a provenance record (participants, content hash, tool-call count).
  If non-conformant it returns the blocking violations and does not finalize.

### Error handling

- Calling any tool before `session_start` returns a clear "call
  session_start first" error.
- A malformed or duplicate node id, or an invalid cardinality, is rejected as
  a violation on the offending fragment, not a crash.
- A non-conformant seed graph blocks `session_finalize` with the framework
  violations.

## Out of scope

- No How, no cells, no audits, no delivery — those are later builds against
  this graph.
- It models the interesting (core) behaviour, not every CRUD triviality.
- It does not prove the delivery economics; What-capture is the front door.

## Acceptance

- TC-896 — `--print-prompt` emits the facilitation prompt for the product.
- TC-897 — a full `--serve` session (start → contexts → structure →
  behaviour → finalize) reaches a conformant finalize that writes Turtle +
  provenance.
- TC-898 — an event that changes a non-entity is rejected in-loop with the
  §3.2 framework message.
- TC-899 — calling a tool before `session_start` is a clear error.
