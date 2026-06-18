---
id: FT-129
title: CLI↔MCP parity for archetype, cell, how, work-unit, and dep — PENDING_MCP cleared
phase: 6
status: complete
depends-on:
- FT-128
adrs:
- ADR-070
tests:
- TC-978
- TC-979
domains:
- api
domains-acknowledged:
  ADR-041: Additive — new read tools for the remaining families; nothing removed, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Handlers call the same pure `product_core::pf` (and graph) functions the CLI adapters use; shared loading in `pf_mcp`.
  ADR-048: Read-only — they read `.product/` artifacts or the loaded graph; no writes.
  ADR-051: Every TC declares `observes:` and asserts on the tool result.
  ADR-018: call_tool parity tests in product-mcp exercise the families.
  ADR-040: The tools compose existing slices; no verification gate changes.
  ADR-020: Tools follow the ToolDef + handler + registry-dispatch contract.
patterns:
- PAT-001
---

## Description

The last `PENDING_MCP` entries — `archetype`, `cell`, `how`, `work-unit`, `dep` —
now expose `product_<family>_*` MCP tools, emptying the parity debt list. These
are read/inspection tools (the write surfaces for these families — `how add/set`,
`cell dispatch`, the various `init` scaffolds — remain CLI-driven for now).

## Functional Specification

### Tools (all read-only)

- **archetype** — `product_archetype_{list,show,validate,check}`.
- **cell** — `product_cell_{show,validate}` (the active `.product/cell.yaml`).
- **how** — `product_how_{show,validate,export}`.
- **work-unit** — `product_work_unit_{show,validate}`.
- **dep** — `product_dep_{list,show,features}` (over the legacy FT/ADR/TC graph).

The `archetype`/`cell`/`how`/`work-unit` handlers read `.product/` artifacts via
`product_core::pf` (shared loading in `pf_mcp`); the `dep` handlers query the
loaded knowledge graph. `PENDING_MCP` is now empty; the parity gate (TC-980)
holds with no documented debt.

### Behaviour

Each tool is read-only: it loads the relevant `.product/` artifact (or the
legacy FT/ADR/TC graph, for `dep`) via `product_core::pf` and returns its
structured view. No write surface is exposed for these families.

### Error handling

- A call against an absent artifact (no active cell, no how-contract, unknown id)
  returns a structured not-found error rather than failing the server.
- A malformed artifact surfaces a parse error naming the file.

## Out of scope

- Write/scaffold/dispatch tools for these families (`how add/set`,
  `cell dispatch`, `*_init`) — a later increment if agent-driven authoring of the
  How contract is wanted over MCP.

## Acceptance

- TC-978 — the framework read tools (how/archetype/cell/work-unit) answer via
  `call_tool`.
- TC-979 — the dependency tools list/show/features via `call_tool`.
