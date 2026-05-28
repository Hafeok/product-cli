---
id: TC-865
title: dashboard root renders phase summary HTML
type: scenario
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_865_dashboard_root_renders_phase_summary_html
observes:
- mcp-response
---

**observes:** [mcp-response]

With a temp repo seeded with three features in three phases, boot the
server and `GET /`. Assert the response body contains:

- A heading per phase that exists in the fixture.
- The total counts of features/ADRs/TCs.
- A `<meta http-equiv="refresh" content="30">` tag.

The page must render without JavaScript (assert no `<script>` tags).

Surface:
- **mcp-response:** HTML body matches the phase rollup contract.
