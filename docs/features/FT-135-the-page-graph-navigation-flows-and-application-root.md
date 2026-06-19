---
id: FT-135
title: The page graph — navigation, flows, and the application root
phase: 7
status: complete
depends-on:
- FT-134
adrs:
- ADR-079
tests:
- TC-997
- TC-998
- TC-999
domains:
- api
- data-model
domains-acknowledged:
  ADR-041: Additive — adds the ApplicationRoot node kind and the navigate/in_flow/navigates_from_root edges; Flow gains entry-page semantics; nothing is removed, so no absence TC is required this increment.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-042: TCs use the reserved `scenario` type only; no new TC type is introduced.
  ADR-040: The page graph is a What-side artifact at the What/How boundary; its queries compose with the existing event-model rules; the verify pipeline is untouched.
  ADR-051: Every TC declares `observes:` and asserts on those surfaces (graph, exit-code, stdout).
  ADR-043: The page-graph model and the derived top-level / primary-navigation queries live in the pure `pf` slice; the CLI is a thin adapter.
  ADR-050: PAT-001 (slice + adapter) governs the `pf` slice + CLI adapter; no new implementation pattern is introduced.
  ADR-048: Reads/writes the captured What graph only (the domain session); no other side effects.
  ADR-018: Scenario TCs drive the binary through assert_cmd; `pf::query`/`pf::rules_ui` carry unit tests. No property or session dimension for a graph model + queries.
patterns:
- PAT-001
---

## Description

§3.2.4 of the framework makes navigation **one page graph**: pages (UI steps)
are nodes and `navigate` transitions are edges. A flow is a named connected
subgraph with a declared entry page; a singleton **application root** holds the
global destinations; and "top-level" is *derived* from the root's edges, never a
hand-applied tag. This feature gives that model graph representation and the
queries that compute primary navigation (ADR-079), building on the typed UiStep
and `transitions_to` edge of FT-134.

## Functional Specification

### Inputs

- The captured What graph for a product (the domain session; `--product` to
  override the default).
- Flow declarations (entry page, member pages), `transitions_to`/navigate edges
  between UiSteps, and the application root's global-destination edges.

### Behaviour

- **Declare the application root and its global destinations.** A singleton
  `ApplicationRoot` node is recorded for the product; `navigates_from_root` edges
  mark the flows (entry pages) and global actions reachable from anywhere — the
  set a renderer draws as primary navigation.
- **Mark a flow's entry page and membership.** A `Flow` declares its entry page
  and its member pages via `in_flow`; flows partition one shared graph (pages may
  be shared, flows may link to each other). The legacy ordered step list is
  superseded by `transitions_to` edges between UiSteps.
- **Add navigate edges.** `transitions_to` edges (from FT-134) connect pages on
  an action or a resulting event, forming the connected subgraph of a flow.
- **Derived top-level and computed primary navigation.** `pf::query` reports a
  page as *top-level* iff it has an inbound `navigates_from_root` edge and
  *nested* otherwise; the primary-navigation set is computed as the root's
  out-edges and recomputes automatically as flows join or leave the root.
  Surfaced through the `product domain` read surface (e.g. `--kind flow`,
  application-root listing).

### Error handling

- Declaring a second `ApplicationRoot` for one product is a clear error (the
  root is a singleton).
- Marking an entry page that is not a UiStep in the graph, or an `in_flow` /
  `navigates_from_root` edge to a non-page node, is reported against the
  offending edge.

## Out of scope

- **Reification of root-navigation to chrome** (phone → drawer, tablet → rail,
  desktop → sidebar) is ADR-083 / FT-139; here the root edges are What-side facts
  that carry no realisation.
- **The seam verification** that checks a page's `surfaces`/`offers` against its
  projection and the Decider's commands is FT-140.
- **State meanings, accessibility, and content** on a page are FT-136 / FT-137 /
  FT-138.

## Acceptance

- TC-997 — mark a flow's entry page and navigate edges; the graph records
  `in_flow` and `transitions_to` forming a connected subgraph.
- TC-998 — a page with an inbound root edge is reported top-level and appears in
  primary navigation; a page reachable only from another is reported nested — no
  page is tagged by hand.
- TC-999 — adding a `navigates_from_root` edge to a flow's entry page changes the
  computed primary-navigation set automatically.
