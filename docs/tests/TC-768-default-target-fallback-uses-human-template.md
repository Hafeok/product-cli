---
id: TC-768
title: default-target-fallback-uses-human-template
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_768_default_target_fallback_uses_human_template
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `default-target-fallback-uses-human-template`

**Given** `product.toml` without a `[context]` section (no `default-target` set)
and no `--target` flag passed,
**When** the user runs `product context FT-XXX`,
**Then** the bundle is rendered through the **`human`** template — byte-equal
to what `product context FT-XXX --target human` produces.

This is the strict version of TC-758. TC-758 only asserts the output is "not
XML" — which is technically true even when the legacy AISP-framed renderer
runs. TC-768 closes the loophole: per the FT-063 selection spec, omitted
target *must* fall back to `human`, not to a parallel legacy renderer.

The drift this test prevents was found during the FT-063 e2e shake-out: with
no flag and no `[context].default-target`, the binary emitted the legacy
`Ω:Bundle` AISP-framed Markdown instead of the clean `human` template
output, even though `product context templates` reported `Default target:
human (fallback)`.

## Validates

- FT-063 — Per-Model Context Bundle Templates (selection: omitted target falls back to `human`)
- ADR-049 — Per-Model Context Bundle Templates as Data Files