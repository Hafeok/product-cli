---
id: TC-868
title: dashboard JSON endpoint matches product status --format json
type: invariant
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_868_dashboard_json_endpoint_matches_product_status_format_json
observes:
- mcp-response
---

**observes:** [mcp-response]

**Parity invariant** (per ADR-020 amendment and ADR-052). Run
`product status --format json` against a temp repo and capture stdout
bytes. Boot `product serve` against the same repo and `GET
/api/status.json`. Assert the response body bytes are byte-for-byte
equal to the CLI capture.

Repeat for `/api/features.json` vs `product feature list --format
json` and `/api/adrs.json` vs `product adr list --format json`.

Any drift fails the test and signals a new entry in the
FT-046/059/062/066/069 parity series.

Surface:
- **mcp-response:** JSON bodies are byte-equal to the CLI output.
