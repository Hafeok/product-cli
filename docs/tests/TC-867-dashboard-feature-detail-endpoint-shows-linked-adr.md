---
id: TC-867
title: dashboard feature detail endpoint shows linked ADRs and TCs
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_867_dashboard_feature_detail_endpoint_shows_linked_adrs_and_tcs
observes:
- mcp-response
---

**observes:** [mcp-response]

Seed a feature `FT-001` linked to `ADR-001` and `TC-001`. `GET
/features/FT-001` and assert the HTML body contains:

- The feature title.
- The Description and Functional Specification headings.
- A link to `/adrs/ADR-001` (with the ADR title as text).
- A link to `/tests/#TC-001` or `/tests` showing TC-001 status.
- The `depends-on` list rendered as anchors.

Unknown feature ID `GET /features/FT-999` returns `404` with the
ADR-013 error envelope.

Surface:
- **mcp-response:** detail page renders the graph neighbourhood at
  depth 1.
