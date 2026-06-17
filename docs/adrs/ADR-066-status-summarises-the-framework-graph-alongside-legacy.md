---
id: ADR-066
title: product status summarises the framework graph alongside the legacy spec
status: accepted
features:
- FT-125
supersedes: []
superseded-by: []
domains:
- api
scope: feature-specific
source-files:
- product-cli/src/commands/status.rs
---

## Context

The framework graph (What / How / delivery) is now the real product description,
but `product status` reported only the legacy FT/ADR/TC graph — the product-cli
tool's own spec. The two graphs are kept separate on purpose (the legacy system
remains the tool's spec; the framework graph is the product). Status should make
the framework graph visible without conflating the two.

## Decision

Append a **Framework graph** section to `product status`, additively:

- Gather What counts from the captured domain session, deciders/slices from
  `.product/deciders` and `.product/slices`, and How counts from
  `.product/how-contract.yaml` + layout rules from `.product/layout.yaml`.
- Render the section after the legacy summary in text, and add a `framework`
  object under `--format json`.
- Omit the section entirely when no framework artifacts exist, so legacy repos
  are unaffected.

The gathering + rendering live in the status CLI adapter: it is a read-only
display over artifacts other slices already own, so no new pure slice is
warranted (ADR-043's "trivial wrapper" judgement).

## Rationale

- Additive, not replacing: the legacy FT/ADR/TC summary is the tool's own status
  and stays; the framework section answers "what does the product graph contain"
  in the same view.
- Omitting the section when empty keeps the change invisible to repos that do not
  use the framework graph.
- Keeping it in the adapter avoids a pure slice for what is purely display
  assembly over existing typed artifacts.

## Rejected alternatives

- **Replace the FT/ADR/TC summary.** Rejected per the chosen model — the legacy
  graph stays as the tool's spec; both are shown.
- **A separate command/flag.** Rejected: the user wanted the framework graph in
  the main `status` view; a separate command hides it.

## Test coverage

- TC-966 — with a captured What graph and a slice, `product status` shows the
  Framework graph section with correct counts.
