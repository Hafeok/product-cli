---
id: ADR-053
title: Domain authoring is a separate What graph with native in-loop conformance
status: accepted
features:
- FT-109
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:1ef0d32c2c645024af7059a5bea412791fe76d75279c9dc5c8961f2998abdd95
source-files:
- product-core/src/pf/mod.rs
- product-core/src/pf/model.rs
- product-core/src/pf/validate.rs
- product-core/src/pf/turtle.rs
- product-core/src/pf/seed.rs
- product-core/src/pf/session.rs
- product-core/src/pf/ops.rs
- product-mcp/src/domain/mod.rs
- product-mcp/src/domain/registry.rs
- product-cli/src/commands/author.rs
---

## Context

`product author domain` (FT-109) captures a product's *What* — the domain
model (§3.1 structure) and event model (§3.2 behaviour) of the Product
Framework — as a conformant RDF graph, validated against the framework's
SHACL shapes (`schema/shapes/shapes.shacl.ttl`). Two design forces collide
with the existing toolchain:

1. **It is a different graph.** The framework What graph (bounded contexts,
   entities, relations, events, commands, …) is *not* the FT/ADR/TC
   knowledge graph the rest of Product manages. Conflating the two would
   overload the slices, the parser, and the MCP surface.

2. **The spec mandates in-loop SHACL.** The reference encoding validates each
   fragment with `pyshacl` against the shapes at call time so the model
   self-corrects. Product is a Rust CLI with a zero-unwrap policy and no
   Python runtime; oxigraph (already a dependency) ships SPARQL but not
   SHACL.

## Decision

1. **A dedicated `pf` subsystem and a dedicated MCP server.** The What graph
   lives in a pure `product-core/src/pf/` slice (typed model, conformance
   checker, Turtle export + seed, open-questions, session container). The 17
   spec tools are served by a separate `product-mcp/src/domain/` server,
   launched by `product author domain --serve` and distinct from `product
   mcp`. The two graphs never mix.

2. **Native conformance mirror, not a SHACL engine.** `pf::validate` mirrors
   the nine "What" shapes of `shapes.shacl.ttl` exactly, emitting the same
   framework-section messages (e.g. "§3.2 Every event must change a real
   domain entity"). Each mutating tool runs the relevant shape on the
   fragment it just built; a blocking violation reverts the fragment and is
   returned as `{ ok: false, violations[] }`. The exported Turtle remains
   verifiable against the real shapes with `pyshacl` (the §6 acceptance
   test) — the native verdict and the `pyshacl` verdict agree on conformance.

3. **Stateful single session, persisted as JSON.** The server holds one
   active session per launch, persisted as `session.json` so a stdio server
   reloads it per call. Turtle is produced only at `session_finalize`, which
   also writes a provenance record (participants, content hash, tool-call
   count). Prior Turtle can seed a new session.

## Rationale

- A separate graph keeps each slice single-responsibility and lets the What
  capture ship small, exactly as the spec frames it (the cheapest, lowest
  dependency first build).
- A native checker avoids a Python runtime dependency in a Rust binary,
  validates in-loop in microseconds, and produces typed violations — while
  the vendored shapes remain the external source of truth, cross-checked in
  tests.
- JSON session state matches Product's "graph is derived, rebuilt per
  invocation" philosophy (ADR-003) and is testable without a live agent.

## Rejected alternatives

- **Shell out to `pyshacl` per mutation.** Rejected: adds a fragile Python
  runtime dependency, subprocess latency on every tool call, and a second
  language to the build.
- **Reuse the FT/ADR/TC graph and MCP surface.** Rejected: the framework
  What graph has different classes, links, and validation rules; overloading
  the existing slices would violate the single-responsibility convention.
- **Store the working graph as Turtle, re-parse each call.** Rejected:
  Turtle→model round-tripping every call is slower and lossier than a typed
  JSON session; Turtle is the export/seed format, not the working store.

## Test coverage

- TC-896 — `--print-prompt` emits the facilitation prompt.
- TC-897 — a full `--serve` session reaches a conformant finalize that writes
  Turtle + provenance.
- TC-898 — an event that changes a non-entity is rejected in-loop with the
  §3.2 message.
- TC-899 — calling a tool before `session_start` is a clear error.
