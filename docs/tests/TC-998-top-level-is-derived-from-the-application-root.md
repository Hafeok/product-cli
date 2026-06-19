---
id: TC-998
title: top-level is derived from the application root
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
runner-args: tc_998_top_level_is_derived_from_the_application_root
last-run: 2026-06-19T16:52:11.237587053+00:00
last-run-duration: 0.5s
---

## Scenario — "top-level" falls out of the graph's edges, not a hand tag

**Given** a captured What graph with an `ApplicationRoot`, a flow entry page
`BrowseCatalog` with an inbound `navigates_from_root` edge, and a page
`ReviewOrder` reachable only via `transitions_to` from `BrowseCatalog`,
**When** the user queries the page graph (the `pf::query` top-level / primary-
navigation queries),
**Then** the process exits 0 and stdout reports `BrowseCatalog` as **top-level**
and present in the computed primary-navigation set, while `ReviewOrder` is
reported **nested** — no page carries a hand-applied "top-level" tag; the
classification is derived purely from the presence of an inbound root edge.

## Validates

- FT-135 — The page graph — navigation, flows, and the application root
- ADR-079 — Navigation is one page graph with named flow subgraphs and a declared application root