---
id: TC-721
title: mcp preflight with missing tc runners returns E024
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_721_mcp_preflight_missing_tc_runners_returns_e024
---

## Given

A temp Product repository containing:
- One feature `FT-300` with `status: in-progress`.
- Two TCs `TC-X` and `TC-Y` both linked to FT-300 with neither `runner` nor `runner-args` set in front-matter.

`product` is launched in MCP stdio mode against the temp repo.

## When

The test sends a `tools/call` JSON-RPC request for `product_preflight` with `{ "id": "FT-300" }`.

## Then

- The response carries an error result.
- The error code is `E024` and the message contains `"health-check-tc-runner-missing"`.
- The error payload includes `tc_ids: ["TC-X", "TC-Y"]` and `tc_paths` matching the on-disk paths of the two TC files.
- No call to `domains::preflight` or the dep-availability scan happens (asserted by ensuring the CLI's `--format json` invocation on the same repo produces the same `E024` envelope and short-circuits in the same place).
