---
id: TC-786
title: mcp_feature_link_reciprocates_adr_features
type: scenario
status: passing
validates:
  features:
  - FT-066
  adrs:
  - ADR-020
phase: 5
runner: cargo-test
runner-args: tc_786_mcp_feature_link_reciprocates_adr_features
last-run: 2026-05-22T07:23:52.891705159+00:00
last-run-duration: 0.2s
---

## Description

Compose a temp repo with FT-X (status `planned`, no ADR links) and
ADR-Z (status `proposed`, `features: []`, no TCs referencing it).
Invoke `product_feature_link` over MCP with `id: FT-X, adr: ADR-Z`.
Assert:

1. `FT-X.adrs` array contains `ADR-Z` on disk.
2. `ADR-Z.features` array contains `FT-X` on disk.
3. The MCP response includes `ADR-Z` in the `reciprocated` array
   with field `features`.
4. The TC-inference path is not triggered (no transitive TCs in the
   repo to infer); `reciprocated` does not gain extra TC entries.

The equivalent `product feature link FT-X --adr ADR-Z` against a
sibling temp repo produces byte-identical feature and ADR files.