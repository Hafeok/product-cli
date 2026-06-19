---
id: TC-997
title: mark flow entry page and navigate edges
type: scenario
status: unimplemented
validates:
  features:
  - FT-135
  adrs:
  - ADR-079
phase: 7
observes:
- graph
- exit-code
runner: cargo-test
runner-args: tc_997_mark_flow_entry_page_and_navigate_edges
---

## Scenario — a flow is a named connected subgraph with an entry page

**Given** a captured What graph with UiSteps `BrowseCatalog`, `ReviewOrder`, and
`OrderConfirmed`,
**When** the user declares a flow `Checkout` with entry page `ReviewOrder`,
marks `ReviewOrder` and `OrderConfirmed` as `in_flow` `Checkout`, and adds a
`transitions_to` (navigate) edge from `ReviewOrder` to `OrderConfirmed`,
**Then** the process exits 0 and the graph records the `in_flow` membership
edges and the `transitions_to` edge, forming a connected subgraph rooted at the
declared entry page — flows partition the one shared page graph rather than
owning a separate navigation model.

## Validates

- FT-135 — The page graph — navigation, flows, and the application root
- ADR-079 — Navigation is one page graph with named flow subgraphs and a declared application root
