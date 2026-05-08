---
id: TC-737
title: CLI feature depends-on mirrors MCP tool
type: scenario
status: passing
validates:
  features:
  - FT-062
phase: 5
runner: cargo-test
runner-args: tc_737_cli_feature_depends_on_mirrors_mcp_tool
last-run: 2026-05-08T08:03:32.829301623+00:00
last-run-duration: 0.8s
---

## Given

Three features `FT-001`, `FT-002`, `FT-003`. None linked.

## When

The CLI invocation
`product feature depends-on FT-001 --add FT-002 --add FT-003` runs.

## Then

- Exit code is `0`.
- `FT-001`'s `depends-on` array now contains both `FT-002` and `FT-003`.
- Running `product feature depends-on FT-001 --remove FT-002` removes
  `FT-002` while preserving `FT-003`.
- A subsequent attempt to add a non-existent target fails with
  non-zero exit and leaves `FT-001`'s file unchanged.
- A cycle-creating `--add` (e.g. self-loop) fails with non-zero exit and
  leaves the file unchanged.
- `product graph check` exits `0` after each successful call.