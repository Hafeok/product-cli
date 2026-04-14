---
id: TC-423
title: ADR amend records amendment and recomputes hash
type: scenario
status: passing
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_423_adr_amend_records_amendment_and_recomputes_hash"
last-run: 2026-04-14T14:44:11.097422144+00:00
---

## Description

Create an ADR, accept it (hash written). Modify the body (fix a typo). Run `product adr amend ADR-XXX --reason "Fix typo"`. Verify: (1) the `amendments` array now contains one entry with `date`, `reason`, and `previous-hash` fields; (2) `content-hash` is updated to match the new body; (3) `product graph check` passes with no E014. Also verify that `product adr amend` without `--reason` is rejected with an error.