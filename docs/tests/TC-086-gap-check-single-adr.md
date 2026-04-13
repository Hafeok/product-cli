---
id: TC-086
title: gap_check_single_adr
type: scenario
status: unimplemented
validates:
  features:
  - FT-029
  adrs:
  - ADR-019
phase: 1
---

run `product gap check ADR-001` against a fixture where ADR-001 has a testable claim with no linked TC. Assert exit code 1 and a G001 finding in stdout JSON.