---
id: FT-119
title: product_domain_* MCP tools — CLI↔MCP parity for the What graph
phase: 6
status: complete
depends-on:
- FT-110
- FT-118
adrs:
- ADR-087
tests:
- TC-981
- TC-982
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive feature — new MCP tools; no existing CLI surface, MCP tool, or schema field is removed or deprecated, so no absence TC is required.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; the MCP handlers are a second adapter over the same pf slices.
  ADR-049: No context-bundle/template change; product_domain_context reuses the existing What-graph bundler.
  ADR-043: Followed — the handlers call the pure pf slice functions the CLI adapter uses; no logic is duplicated.
  ADR-048: Reads/writes the domain session under `.product/author-domain/<product>/`; the FT/ADR/TC graph is untouched.
  ADR-051: Every TC declares `observes:` (exit-code, stdout) and asserts on those surfaces.
  ADR-018: Two scenario TCs drive the MCP server through the assert_cmd harness; the handlers carry unit tests in product-mcp. No property or session dimension for thin handlers.
  ADR-040: The tools are structured graph operations validated in-loop; they cross no LLM boundary at call time and the verify pipeline is untouched.
patterns:
- PAT-001
---

## Description

Pays down the first entry in the CLI↔MCP parity debt (FT-118): the `domain`
family is now exposed on the main `product mcp` server as `product_domain_*`
tools, so an MCP-driven agent can read and edit the captured What graph exactly
as a human does with `product domain`. The handlers call the same
`product_core::pf` functions the CLI adapter uses, so the two surfaces cannot
drift.

## Functional Specification

### Inputs / tools

- Read: `product_domain_list` (`kind?`), `product_domain_show` (`id`),
  `product_domain_validate`, `product_domain_export`,
  `product_domain_context` (`id`, `depth?`).
- Write (gated by `mcp.write`): `product_domain_new` (`kind`, `id`, + fields),
  `product_domain_edit` (`id`, + fields), `product_domain_rm` (`id`).
- All accept an optional `product`, defaulting to the repo's configured `name`;
  they operate on the session under `.product/author-domain/<product>/`.

### Behaviour

- Each tool mirrors its CLI subcommand and returns the same data: mutations
  return `{ ok, node, violations[] }` (validated in-loop against the framework
  shapes); reads return the node/links, the conformance report, the Turtle, or
  the context bundle.
- `domain` is removed from the parity gate's `PENDING_MCP` list; the gate
  (TC-980) confirms the tools exist and the debt entry is retired.

### Error handling

- A non-conformant `new`/`edit` returns `ok: false` with the framework
  violations; the fragment is not committed.
- Reading a product with no captured graph yet is a clear error.

## Out of scope

- It does not change the `author domain --serve` authoring server (a separate,
  session-scoped tool surface).
- The remaining parity debt (`archetype`, `cell`, `dep`, `how`, `work-unit`) is
  follow-on work the gate continues to track.

## Acceptance

- TC-981 — the `product_domain_*` tools work through `product mcp` and match the
  CLI result.
- TC-982 — `product_domain_new` rejects a non-conformant fragment in-loop.
