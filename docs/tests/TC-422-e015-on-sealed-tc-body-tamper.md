---
id: TC-422
title: E015 on sealed TC body tamper
type: scenario
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_422_e015_on_sealed_tc_body_tamper"
---

## Description

Create a TC via `product test new`, seal it via `product hash seal TC-XXX`. Verify `content-hash` is written. Modify the TC body text. Run `product graph check`. Verify `error[E015]` is emitted with file path and hash mismatch. Repeat for changes to `type` and `validates.adrs` — both must trigger E015.
