---
id: ADR-061
title: The domain (What) graph is exposed as product_domain_* MCP tools
status: accepted
features:
- FT-119
supersedes: []
superseded-by: []
domains:
- api
- data-model
scope: feature-specific
content-hash: sha256:aa6a462c5ed13ed716d3b3681d6b67ef57c9eed96dc0bebdce0218af8a2d69d9
source-files:
- product-mcp/src/domain_handlers.rs
- product-mcp/src/tools/read.rs
- product-mcp/src/tools/write.rs
- product-mcp/src/registry.rs
---

## Context

The CLI↔MCP parity gate (ADR-060) flagged `domain` as the first piece of debt:
`product domain` (list/show/new/edit/rm/validate/export/context) had no
counterpart on the main `product mcp` server, so MCP-driven agents could not
read or edit the captured What graph the way a human can at the CLI.

(The `author domain --serve` authoring server exposes a *different*, in-session
tool surface — `session_start`, `add_entity`, … — on its own server; it is not
the main `product mcp` registry the parity gate inspects, and it is an
interactive capture flow rather than direct CRUD.)

## Decision

Expose the `domain` family on the main MCP registry as `product_domain_*`
tools, mirroring the CLI subcommands and calling the **same** `product_core::pf`
functions the CLI adapter uses (no logic duplicated):

| MCP tool | mirrors | write? |
|---|---|---|
| `product_domain_list` / `_show` / `_validate` / `_export` / `_context` | the read subcommands | no |
| `product_domain_new` / `_edit` / `_rm` | the CRUD subcommands | yes |

Handlers (`domain_handlers.rs`) resolve the product (the `product` arg, else the
repo's configured `name`), load/save the persisted session under
`.product/author-domain/<product>/`, and return the same shapes the CLI
produces (`{ ok, node, violations }` for mutations). Writes are gated by
`mcp.write` like every other mutating tool. `domain` is removed from the parity
gate's debt list; the gate confirms the tools exist and that the debt entry is
retired.

## Rationale

- Sharing the `pf` slice functions means the MCP and CLI surfaces cannot drift:
  one validator, one CRUD path, two entry points — exactly the slice+adapter
  principle the parity rule protects.
- Resolving the product + session the same way as the CLI keeps behaviour
  identical across surfaces (same default product, same session file).
- Paying the first debt entry proves the gate's loop closes: add tools → remove
  from `PENDING_MCP` → the gate goes green by detection, not assertion.

## Rejected alternatives

- **Point the parity gate at the `author domain --serve` server instead.**
  Rejected: that server is an interactive capture surface (session-scoped add_*
  tools), not direct CRUD over a stored graph; agents doing graph edits want the
  main-registry tools, and conflating the two would weaken the gate.

## Test coverage

- TC-981 — the `product_domain_*` tools work through `product mcp` and match the
  CLI result (create → validate → list).
- TC-982 — `product_domain_new` rejects a non-conformant fragment in-loop.
