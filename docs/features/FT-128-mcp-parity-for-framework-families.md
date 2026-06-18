---
id: FT-128
title: CLI↔MCP parity for the decider, slice, deliverable, and release families
phase: 6
status: complete
depends-on:
- FT-118
- FT-127
adrs:
- ADR-069
tests:
- TC-976
- TC-977
domains:
- api
domains-acknowledged:
  ADR-041: Additive — new `product_decider_*`/`product_slice_*`/`product_deliverable_*`/`product_release_*` tools; nothing removed, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-043: Handlers call the same pure `product_core::pf` functions the CLI adapters use; shared loading lives in `pf_mcp`.
  ADR-048: Read the captured What graph + `.product/{deciders,slices,deliverables,releases}`; write tools are gated by `mcp.write`.
  ADR-051: Every TC declares `observes:` and asserts on the tool result.
  ADR-018: call_tool parity tests in product-mcp exercise each family end-to-end.
  ADR-040: The tools compose the existing slices; no verification gate changes.
  ADR-020: New tools follow the ToolDef + handler + registry-dispatch contract; write tools set requires_write.
patterns:
- PAT-001
---

## Description

FT-118 established the CLI↔MCP parity gate; the framework families built since
(`decider`, `slice`, `deliverable`, `release`) were carried as documented debt in
`PENDING_MCP`. This feature clears that debt: each family now exposes
`product_<family>_*` MCP tools mirroring its CLI subcommands, so an agent can
drive the whole §3.3/§7 surface over MCP — derive/validate/simulate a Decider,
assemble a slice's build-context, and build + check the delivery hierarchy.

## Functional Specification

### Tools

- **decider** — `product_decider_{list,show,validate,simulate}` (read) +
  `product_decider_derive` (write).
- **slice** — `product_slice_{list,show,context}` (read) + `product_slice_new`
  (write).
- **deliverable** — `product_deliverable_{list,show,done}` (read) +
  `product_deliverable_{new,accept}` (write).
- **release** — `product_release_{list,show,done}` (read) + `product_release_new`
  (write).

Each handler calls the same `product_core::pf` functions as the CLI adapter,
against `repo_root/.product/…`; shared loading (`product`, the What graph,
artifact id sets) lives in `pf_mcp`. Write tools set `requires_write` and are
refused unless `mcp.write` is enabled. The four families are removed from
`PENDING_MCP`; the parity gate (TC-980) confirms each now has a tool.

### Behaviour

Each `product_<family>_*` tool mirrors its CLI subcommand: it loads the same
`.product/…` artifacts via `product_core::pf`, performs the read or write, and
returns the same structured result. Read tools are always available; write tools
set `requires_write` and run only when `mcp.write` is enabled.

### Error handling

- A tool call against a missing or malformed artifact returns a structured MCP
  error mirroring the CLI's exit-code semantics, never crashing the server.
- A write tool invoked while write tools are disabled is refused with a clear
  capability error and makes no change.

## Out of scope

- `decider conform` is not exposed over MCP (it spawns an arbitrary runner
  subprocess); it remains a CLI gate.

## Acceptance

- TC-976 — the decider tools derive/validate/simulate via `call_tool`.
- TC-977 — the delivery tools build the slice→deliverable→release chain and
  compute `done` via `call_tool`.
