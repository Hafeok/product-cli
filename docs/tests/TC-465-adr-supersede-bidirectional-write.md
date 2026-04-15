---
id: TC-465
title: adr supersede bidirectional write
type: scenario
status: passing
validates:
  features:
  - FT-038
  adrs:
  - ADR-037
phase: 1
runner: cargo-test
runner-args: "tc_465_adr_supersede_bidirectional_write"
last-run: 2026-04-15T11:45:30.776758337+00:00
last-run-duration: 0.2s
---

Create ADR-A and ADR-B. Run `product adr supersede ADR-B --supersedes ADR-A`. Assert:
1. ADR-B front-matter contains `supersedes: [ADR-A]`
2. ADR-A front-matter contains `superseded-by: [ADR-B]`
3. ADR-A status changed to `superseded` (if it was `accepted`)

Then run `product adr supersede ADR-B --remove ADR-A`. Assert both links are removed from both files.