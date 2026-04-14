---
id: TC-424
title: W016 for accepted ADR without content-hash
type: scenario
status: unimplemented
validates:
  features: [FT-034]
  adrs: [ADR-032]
phase: 1
runner: cargo-test
runner-args: "tc_424_w016_for_accepted_adr_without_content_hash"
---

## Description

Create an ADR file manually with `status: accepted` but no `content-hash` field (simulating a pre-existing accepted ADR that predates this feature). Run `product graph check`. Verify the output contains `warning[W016]` naming the file and suggesting `product adr rehash`. Verify exit code is 2 (warning, not error) when no other errors are present.
