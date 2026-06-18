---
id: FT-124
title: product slice — a saved pointer to a section of the event model
phase: 6
status: complete
depends-on:
- FT-110
adrs:
- ADR-065
tests:
- TC-958
- TC-959
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — a new `slice` subcommand family; nothing existing is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command for the FT/ADR graph; it assembles a What-graph bundle and changes no template surface.
  ADR-043: Validation + closure live in pure `pf::slice`/`pf::bundle`; the CLI is a thin BoxResult adapter.
  ADR-048: Reads the captured What graph; writes only the slice pointer file on `new`.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the binary through assert_cmd; `pf::slice`/`pf::bundle` carry unit tests over validation + closure. No property or session dimension.
  ADR-040: A delivery slice is a §7.1 subgraph of the What; it composes the existing bundle assembler and touches no verification gate.
patterns:
- PAT-001
---

## Description

§7.1 defines a delivery feature as "the smallest independently valuable and
verifiable slice — typically one behavioural flow over its concepts," and
insists it is "a subgraph of the What, not a free-floating ticket." This feature
adds exactly that: a **saved pointer** into the captured event model that
restates nothing. From the pointer, the concrete build-context an LLM needs is
*assembled from the model*, so the feature and the spec can never drift.

(The command is named `slice` rather than `feature` because `product feature`
already manages the FT-XXX specification graph; this is the framework's §7.1
delivery slice over the What.)

## Functional Specification

### The pointer (`.product/slices/<id>.yaml`)

```yaml
id: place-order
anchors: [PlaceOrderFlow]   # node ids — a flow (typical), a context, an aggregate…
depth: 2                    # optional traversal depth (default 2)
```

Nothing about the behaviour is restated — only where it lives in the graph.

### Behaviour

- `product slice new <id> --anchor <node> [--anchor <node>…] [--depth N]` —
  validate that every anchor resolves to a real node in the captured What graph,
  then save the pointer. A dangling anchor is rejected (exit 1); nothing is
  written.
- `product slice context <name> [--depth N]` — assemble the concrete context:
  the union of the What subgraph reachable from each anchor (the bundle
  closure), rendered as an LLM-ready markdown bundle. `--depth` overrides the
  saved depth.
- `product slice list` / `show <name>` — the slices present / a slice's pointer.

### Closure

The context is the existing What-graph bundle over a *set* of focus nodes: a
flow pulls in its steps, those commands pull their targeted entity and emitted
events, events pull the entity they change, and the surrounding contexts,
invariants, relations, and read models come in by graph adjacency to the
configured depth.

### Error handling

- `new` whose anchor does not resolve to a node in the captured What graph is
  rejected (exit 1) and nothing is written.
- `context`/`show` on an unknown slice id fails with a clear "no such slice"
  error; an empty closure yields an empty bundle rather than an error.

## Out of scope

- Releases (§7.1's other delivery unit — a coherent set of slices) and the
  `done` predicate (§7.2) are separate increments.

## Acceptance

- TC-958 — a slice's `context` assembles the reachable subgraph (commands,
  events, contexts) without restating anything.
- TC-959 — `new` rejects an anchor that is not a node in the What graph.
