---
id: TC-211
title: sparql_select_feature_adrs
type: scenario
status: unimplemented
validates:
  features: 
  - FT-011
  - FT-016
  - FT-024
  adrs:
  - ADR-008
phase: 1
---

load a graph with FT-001 linked to ADR-001 and ADR-002. Execute `SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }`. Assert the result set contains exactly `adr:ADR-001` and `adr:ADR-002`.