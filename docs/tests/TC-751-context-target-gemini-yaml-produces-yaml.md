---
id: TC-751
title: context-target-gemini-yaml-produces-yaml
type: scenario
status: passing
validates:
  features:
  - FT-063
  adrs:
  - ADR-049
phase: 1
runner: cargo-test
runner-args: tc_751_context_target_gemini_yaml_produces_yaml
last-run: 2026-06-10T19:41:51.052986067+00:00
last-run-duration: 0.3s
---

## Scenario — `context-target-gemini-yaml-produces-yaml`

**Given** the built-in `gemini-yaml` template (`[format].structure = "yaml"`),
**When** the user runs `product context FT-XXX --target gemini-yaml`,
**Then** stdout is a valid YAML mapping. The top-level keys are exactly the entries of `[ordering].sections`, in declared order.

`yaml::from_str` parses the output without error.

## Validates

- FT-063 — Per-Model Context Bundle Templates (YAML rendering)
- ADR-049 — Per-Model Context Bundle Templates as Data Files