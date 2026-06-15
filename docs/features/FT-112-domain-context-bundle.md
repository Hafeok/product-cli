---
id: FT-112
title: product domain context — assemble an LLM context bundle from the What graph
phase: 6
status: complete
depends-on:
- FT-110
adrs:
- ADR-053
tests:
- TC-920
- TC-921
- TC-922
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `domain context` verb; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: This IS a bundle command, but over the What graph (a new bundle source), not the FT/ADR/TC context templates; no template surface changes.
  ADR-043: Followed — assembly is a pure `pf::bundle` slice in product-core; the CLI is a thin BoxResult adapter.
  ADR-048: Reads the domain session under `.product/author-domain/<product>/`; writes nothing.
  ADR-051: Every TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: Three scenario TCs drive the binary through the assert_cmd harness; the pf::bundle slice carries unit tests. No property or session dimension for a read-only assembler.
  ADR-040: The bundle is assembled structurally from the captured What graph; no LLM boundary is crossed at assembly time and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

Once a product's *What* is captured (FT-109/110), the next use is to **build
context bundles from those artifacts** — focused, LLM-ready slices of the
domain model. `product domain context <id>` is the domain analog of
`product context <feature>`: it takes a focus node (an entity, bounded
context, flow, command, …), traverses the What graph outward to a depth, and
renders a markdown bundle — the focus node in full, then its neighbourhood
grouped by kind.

This turns the captured graph from a static record into a working input: hand
an implementer (human or agent) exactly the domain knowledge around the
concept they are touching — its definition, the events that change it, the
commands that target it, the read models that project it, its relations,
invariants, and bounded context — without dumping the whole graph.

## Functional Specification

### Inputs

- A focus node id (any of the eleven What-graph kinds).
- `--depth N` (default 2): traversal hops from the focus over the undirected
  edge set (inContext, relation from/to, changes, targets, emits, projects,
  triggers/displays, flow steps, mapping correspondences, applies_to).
- The target product, defaulting to the repo's configured name
  (`--product` to override).

### Behaviour

- Emits a markdown bundle on stdout: a `# Domain Context Bundle` header, an
  `⟦Ω:WhatBundle⟧` summary block (product, focus + kind, depth, node count),
  the focus node rendered in full, then a section per kind for the reachable
  neighbourhood (Bounded contexts, Entities, …, Flows).
- `--depth` bounds reach: a node N hops from the focus appears only at
  `--depth ≥ N`.

### Error handling

- An unknown focus id exits 1 with a clear "no node with id" message.
- A missing domain graph exits 1 pointing at `domain new` / `author domain`
  (shared with the other `domain` read commands).

## Out of scope

- It does not inject domain artifacts into the FT/ADR/TC feature bundles
  (`product context`/`implement`) — that cross-graph linkage is future work.
- It does not author or mutate the graph (FT-109/110 cover that).
- It does not call an LLM; it assembles the bundle for one.

## Acceptance

- TC-920 — context emits a bundle with the focus node and its direct neighbours.
- TC-921 — `--depth` controls reach (a 2-hop node appears only at depth ≥ 2).
- TC-922 — an unknown focus id is a clear error (exit 1).
