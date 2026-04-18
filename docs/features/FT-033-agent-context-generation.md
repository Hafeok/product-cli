---
id: FT-033
title: Agent Context Generation
phase: 3
status: complete
depends-on: []
adrs:
- ADR-031
- ADR-002
- ADR-006
- ADR-020
- ADR-025
tests:
- TC-404
- TC-405
- TC-406
- TC-407
- TC-408
- TC-409
- TC-410
- TC-411
- TC-412
- TC-413
- TC-414
- TC-415
- TC-416
- TC-417
- TC-418
- TC-419
domains:
- api
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
---

## Description

Generated `AGENTS.md` and `product schema` command (ADR-031). `product agent-init` generates a repo-root file from actual repo state containing: working protocol, current front-matter schemas, domain vocabulary, repository state summary, and MCP tool usage guide. `product schema` returns the complete front-matter schema for any artifact type. Both are exposed as MCP read tools (`product_schema`, `product_agent_context`). Configurable via `[agent-context]` in `product.toml`.
