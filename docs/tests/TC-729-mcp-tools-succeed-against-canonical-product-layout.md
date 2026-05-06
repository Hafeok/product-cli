---
id: TC-729
title: mcp tools succeed against canonical .product layout
type: session
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_729_mcp_tools_succeed_against_canonical_product_layout
---

## Description

Session-level regression test guarding FT-061. Sets up a fresh
canonical-layout repo (config at `.product/config.toml`) and verifies
that representative MCP read/write tools succeed against it.

## Setup

1. `tempdir` with no Product state.
2. `product init -y` from inside the tempdir.
3. Confirm `.product/config.toml` exists and root `product.toml` does
   not.
4. Spawn `product mcp` (stdio transport) with cwd set to the tempdir.

## Steps

For each of the following MCP tools, send a `tools/call` JSON-RPC
request and assert the response carries `result` (not `error`):

- `product_responsibility` — empty args.
- `product_graph_check` — empty args.
- `product_feature_list` — empty args.
- `product_feature_new` — `{title: "Smoke", phase: 1}`.
- `product_drift_check` — empty args.
- `product_preflight` — `{id: "<smoke-feature-id>"}` for the feature
  just created.

## Expected

- Every call returns success.
- After the session, `.product/features/FT-001-*.md` exists.
- `mcp.write` defaults to `false` in `product init -y`, so
  `product_feature_new` initially errors with the canned
  `Write tools are disabled` message — to test the success path,
  re-init with `--write-tools` (or override the config beforehand)
  and re-run.

## Negative path

A second tempdir uses the legacy layout (root `product.toml` only).
Repeat the calls above and assert the same set succeeds — guarding
against regressions to current test fixtures.

## Failure criteria

Any tool returning a JSON-RPC error referencing
`Failed to read .../product.toml` against a canonical-layout repo
fails this test. That error string is the FT-061 fingerprint.
