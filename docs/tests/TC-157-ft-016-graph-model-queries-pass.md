---
id: TC-157
title: FT-016 graph model queries pass
type: exit-criteria
status: passing
validates:
  features:
  - FT-006
  - FT-016
  - FT-024
  - FT-014
  adrs:
  - ADR-012
  - ADR-008
  - ADR-003
phase: 1
runner: cargo-test
runner-args: "tc_157_ft016_graph_model_queries_pass"
last-run: 2026-04-14T14:53:21.175394484+00:00
---

## Description

Validates the complete FT-016 graph model by exercising all graph capabilities end-to-end: graph rebuild produces valid TTL with centrality scores, SPARQL queries return correct results, topological sort respects feature dependencies, centrality ranking works, impact analysis reports affected artifacts, context depth-2 includes transitive artifacts, and graph check passes with no broken links or cycles.