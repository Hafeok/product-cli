---
id: ADR-069
title: The framework families expose MCP tools mirroring their CLI subcommands
status: accepted
features:
- FT-128
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
source-files:
- product-mcp/src/pf_mcp.rs
- product-mcp/src/decider_handlers.rs
- product-mcp/src/delivery_handlers.rs
- product-mcp/src/tools/decider.rs
- product-mcp/src/tools/delivery.rs
- product-mcp/src/registry.rs
---

## Context

FT-118's parity gate requires every CLI command family to expose equivalent MCP
tools or be listed in `PENDING_MCP`. The framework families (`decider`, `slice`,
`deliverable`, `release`) were added to `PENDING_MCP` as they were built. Driving
the §3.3/§7 surface from an agent (the framework's "an LLM does the work" goal)
needs those tools to exist.

## Decision

Expose each family as `product_<family>_*` tools following the ADR-020 contract
(ToolDef + handler + registry dispatch arm):

- Tool defs in `tools/decider.rs` + `tools/delivery.rs`, added to
  `build_tool_list`. Read tools set `requires_write: false`; create/derive/accept
  set `requires_write: true`.
- Handlers in `decider_handlers` + `delivery_handlers` call the **same**
  `product_core::pf` functions the CLI adapters use (derive/validate/simulate,
  bundle closure, feature_done/release_done, validation), against
  `repo_root/.product/…`. Shared loading — resolving the product, the What graph,
  and artifact id sets — lives in `pf_mcp` to avoid duplication.
- Dispatch arms added to `registry.rs`; the four families removed from
  `PENDING_MCP`.

`decider conform` is deliberately not exposed: it spawns an arbitrary runner
subprocess, so it stays a CLI-invoked gate rather than an MCP tool.

## Rationale

- Calling the same pure slice functions as the CLI is what makes "parity" real:
  the MCP path and the CLI path cannot diverge in behaviour.
- A shared `pf_mcp` helper keeps the handlers thin and the loading logic in one
  place.
- Gating writes via `requires_write` reuses the existing `mcp.write` safety
  switch; reads are always available.

## Rejected alternatives

- **Leave the families in PENDING_MCP.** Rejected: the agent-driven workflow is
  the point of the framework; the tools must exist.
- **Expose `decider conform` over MCP.** Rejected: arbitrary subprocess execution
  over MCP is a needless risk; the CLI is the right place for it.

## Test coverage

- TC-976 — decider tools (derive/validate/simulate) via `call_tool`.
- TC-977 — delivery tools build the chain + compute done via `call_tool`.
- The parity gate (TC-980) confirms each family now has a tool and is no longer
  in `PENDING_MCP`.
