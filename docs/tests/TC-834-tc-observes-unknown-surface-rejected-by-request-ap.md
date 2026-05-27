---
id: TC-834
title: tc_observes_unknown_surface_rejected_by_request_apply
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_834_tc_observes_unknown_surface_rejected_by_request_apply
observes:
- stdout
- exit-code
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.3s
---

## Description

Compose a temp repo. Submit a `product_request_apply` payload
containing a TC `changes:` entry that adds `observes:
[bogus_surface]` to an existing TC. Capture the response.

Assert:

1. The response exits with a non-zero exit-code and prints E026
   on stdout (JSON format) or stderr (text format).
2. The error text names `bogus_surface` as the offending value.
3. The error lists the allowed vocabulary in the hint or detail.
4. The on-disk TC file is **unchanged** — the write does not
   partially succeed.

## Formal specification

⟦Λ:Scenario⟧
Given a repository containing TC-X,
When a `product_request_apply` payload attempts to add
  `observes: [bogus_surface]` to TC-X,
Then the response carries E026 naming `bogus_surface`,
And the TC-X file on disk is byte-identical to its
  pre-invocation snapshot.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩