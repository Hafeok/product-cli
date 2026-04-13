---
id: TC-047
title: context_bundle_adr_order_centrality
type: scenario
status: passing
validates:
  features:
  - FT-006
  - FT-011
  - FT-016
  - FT-014
  adrs:
  - ADR-012
phase: 1
runner: cargo-test
runner-args: "tc_047_context_bundle_adr_order_centrality"
---

feature linked to ADR-001 (high centrality) and ADR-007 (low centrality). Assert ADR-001 appears before ADR-007 in the default bundle output.