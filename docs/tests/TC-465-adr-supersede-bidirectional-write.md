---
id: TC-465
title: adr supersede bidirectional write
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
---

Create ADR-A and ADR-B. Run `product adr supersede ADR-B --supersedes ADR-A`. Assert:
1. ADR-B front-matter contains `supersedes: [ADR-A]`
2. ADR-A front-matter contains `superseded-by: [ADR-B]`
3. ADR-A status changed to `superseded` (if it was `accepted`)

Then run `product adr supersede ADR-B --remove ADR-A`. Assert both links are removed from both files.