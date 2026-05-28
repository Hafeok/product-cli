---
id: TC-874
title: dashboard reloads graph on each request
type: invariant
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_874_dashboard_reloads_graph_on_each_request
observes:
- mcp-response
- file
---

**observes:** [mcp-response, file]

Boot the server. `GET /api/features.json` and parse the array length
(`N1`). Out of band — i.e. using `product feature new "freshness probe"` —
create a new feature on disk. Without restarting the server, `GET
/api/features.json` again and parse the array length (`N2`). Assert
`N2 == N1 + 1` and that the new feature ID appears in the response.

This proves the per-request graph reload contract: there is no in-memory
cache that could shadow on-disk edits.

Surfaces:
- **file:** a new feature file appears on disk between requests.
- **mcp-response:** the second response reflects the new file.
