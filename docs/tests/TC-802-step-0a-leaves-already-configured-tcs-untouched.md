---
id: TC-802
title: Step 0a leaves already-configured TCs untouched
type: scenario
status: passing
validates:
  features:
  - FT-068
  adrs: []
phase: 5
runner: cargo-test
runner-args: tc_802_step_0a_skips_already_configured_tcs
runner-timeout: 120
last-run: 2026-05-26T12:31:19.240471574+00:00
last-run-duration: 0.2s
---

## Scenario

A feature `FT-XXX` is `planned` and has one linked TC whose
front-matter already contains `runner: cargo-test` and
`runner-args: tc_999_custom_name` (a name that differs from the
filename-derived slug).

## When

The user runs `product implement FT-XXX`.

## Then

Step 0a iterates the feature's TCs, observes that the TC already
has both runner fields populated, and **skips** it. No diagnostic
line is printed for this TC. No write occurs.

After Step 0a completes, the TC's front-matter still has
`runner-args: tc_999_custom_name` — the user's explicit override
is preserved verbatim. This proves Step 0a is non-destructive: once
the user or a previous agent declares the runner-args, the
auto-fill never overwrites it.