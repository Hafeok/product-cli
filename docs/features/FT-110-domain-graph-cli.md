---
id: FT-110
title: product domain — CLI list, show, and CRUD over the captured What graph
phase: 6
status: complete
depends-on:
- FT-109
adrs:
- ADR-053
tests:
- TC-901
- TC-902
- TC-903
- TC-904
- TC-905
- TC-906
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — a new `domain` subcommand family; no existing CLI surface, MCP tool, schema field, or behaviour is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: No context bundle or template change; the command does not render bundles.
  ADR-043: Followed — CRUD logic is a pure `pf::edit` slice in product-core; the CLI is a thin BoxResult adapter that loads/saves the session.
  ADR-048: Operates only on the domain session under `.product/author-domain/<product>/`; the FT/ADR/TC graph is untouched.
  ADR-051: Every TC declares `observes:` (exit-code, stdout, stderr) and its body asserts on those named surfaces.
  ADR-018: Six scenario TCs drive the binary through the assert_cmd harness; the `pf::edit` slice carries unit tests. No property or session dimension for a CRUD adapter.
  ADR-040: The What graph is a structural artifact; CRUD writes run the same in-loop conformance checker, and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

`product author domain` (FT-109) captures a What graph through an LLM-driven
MCP session. FT-110 adds a direct, human CLI over the same persisted graph so
you can **see and interact with the captured artifacts** without launching an
agent: list them, show one with its links, and create/update/delete them.

The artifacts are the eleven Product-Framework node kinds — bounded context,
entity, value object, relation, invariant, context mapping, command, event,
read model, wireframe step, flow. The command family reads and writes the
session store (`.product/author-domain/<product>/session.json`) that the
author session and MCP server share, so the CLI and the agent operate on one
graph. Every write goes through the same in-loop conformance checker as the
MCP `add_*` tools (ADR-053), so a CLI edit can never commit a non-conformant
fragment.

## Functional Specification

### Inputs

- A verb: `list`, `show`, `new`, `edit`, `rm`, `validate`, or `export`.
- The target product, defaulting to the repo's configured `name` (overridable
  with `--product`), matching `author domain`.
- For `new`/`edit`: a node kind plus `--field` flags (`--label`, `--context`,
  `--definition`, `--cardinality`, `--rationale`, `--changes`, `--emits`, …)
  that map onto the node's schema fields.

### Behaviour

- `product domain list [<kind>]` — print every node (or one kind) as
  `kind  id  label` rows.
- `product domain show <id>` — print the node's fields plus its links (what
  changes/targets/projects it; its relations) as JSON on stdout.
- `product domain new <kind> <id> [--field …]` — create a node, validating the
  fragment in-loop; auto-initialises the graph on first write.
- `product domain edit <id> [--field …]` — patch fields, re-validating.
- `product domain rm <id>` — delete a node; warn on stderr about any reference
  the deletion leaves dangling.
- `product domain validate` — run the conformance checker over the whole graph.
- `product domain export` — print the conformant What graph as Turtle.

### Error handling

- A non-conformant `new`/`edit` is rejected: the fragment is reverted, the
  framework-section violations print, and the process exits 1.
- A malformed id, duplicate id, or unknown kind exits non-zero with a clear
  message.
- `validate` exits 1 when the graph has violations.
- Reading a graph that does not exist yet exits 1 with a "no domain graph"
  message pointing at `domain new` / `author domain`.

## Out of scope

- It does not author through an agent (that is FT-109) — it is the direct CLI.
- It does not edit the finalized Turtle export in place; it operates on the
  working session store, from which `export` regenerates Turtle.
- It does not manage the FT/ADR/TC knowledge graph — a separate surface.

## Acceptance

- TC-901 — `new` → `list`/`list <kind>` → `show` round-trips a built graph.
- TC-902 — a non-conformant `new` is rejected (exit 1, §-message, not committed).
- TC-903 — a rejected `edit` is reverted; a valid one persists.
- TC-904 — `rm` warns on dangling refs; `validate` exit code tracks conformance.
- TC-905 — `export` emits Turtle for the captured graph.
- TC-906 — reading a non-existent graph is a clear error (exit 1).
