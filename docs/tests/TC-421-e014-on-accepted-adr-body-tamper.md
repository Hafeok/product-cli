---
id: TC-421
title: E014 on accepted ADR body tamper
type: scenario
status: passing
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_421_e014_on_accepted_adr_body_tamper"
last-run: 2026-04-14T14:44:11.097422144+00:00
---

## Description

Create an ADR, accept it (writing the content-hash), then modify its body text directly (append a sentence to the Rationale section). Run `product graph check`. Verify the output contains `error[E014]` with the file path, expected hash, and actual hash. Verify exit code is 1. Repeat with a title change — verify E014 is also emitted for title mutations.