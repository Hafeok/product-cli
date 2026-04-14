---
id: TC-421
title: E014 on accepted ADR body tamper
type: scenario
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_421_e014_on_accepted_adr_body_tamper"
---

## Description

Create an ADR, accept it (writing the content-hash), then modify its body text directly (append a sentence to the Rationale section). Run `product graph check`. Verify the output contains `error[E014]` with the file path, expected hash, and actual hash. Verify exit code is 1. Repeat with a title change — verify E014 is also emitted for title mutations.
