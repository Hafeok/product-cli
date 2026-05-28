---
id: TC-866
title: dashboard features endpoint lists every feature
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_866_dashboard_features_endpoint_lists_every_feature
observes:
- mcp-response
---

**observes:** [mcp-response]

Seed a fixture with five features across two phases and two statuses.
`GET /features` and assert every feature ID appears in the rendered
HTML in an anchor pointing at `/features/FT-XXX`. `GET
/features?phase=2` and assert only phase-2 features appear. `GET
/features?status=planned` filters by status. The filter semantics must
match `product feature list --phase 2 --status planned` exactly.

Surface:
- **mcp-response:** filtered HTML feature index matches the shared
  `feature::list` rule.
