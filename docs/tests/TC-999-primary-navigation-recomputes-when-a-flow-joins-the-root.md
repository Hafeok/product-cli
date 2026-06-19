---
id: TC-999
title: primary navigation recomputes when a flow joins the root
type: scenario
status: passing
validates:
  features:
  - FT-135
  adrs:
  - ADR-079
phase: 7
observes:
- graph
- stdout
runner: cargo-test
runner-args: tc_999_primary_navigation_recomputes_when_a_flow_joins_the_root
last-run: 2026-06-19T16:52:11.237587053+00:00
last-run-duration: 0.5s
---

## Scenario — primary navigation is the root's out-edges, computed not maintained

**Given** a captured What graph whose computed primary navigation lists one
global destination (the `Browse` flow's entry page),
**When** the user adds a `navigates_from_root` edge from the application root to
the `Checkout` flow's entry page,
**Then** the process exits 0 and the recomputed primary-navigation set now lists
both destinations — the set changed automatically because it *is* the root's
out-edges, not a separately maintained list; removing the edge would drop the
destination just as automatically.

## Validates

- FT-135 — The page graph — navigation, flows, and the application root
- ADR-079 — Navigation is one page graph with named flow subgraphs and a declared application root