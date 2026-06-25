---
id: FT-148
title: System as a first-class What node
phase: 1
status: complete
depends-on: []
adrs:
- ADR-090
tests:
- TC-1030
- TC-1031
domains: []
domains-acknowledged: {}
---

## Description

Framework §3.2.5 introduces the **system** as a first-class What-side node: the
named thing a page graph and flows belong to. A system carries a `kind`
(application/website/service/cli/…), a one-sentence `purpose`, its target
**platform** contexts and target **interaction classes** (§3.2.2), and the
`ApplicationRoot` its page graph roots at. A What may declare several systems
over one shared domain model — a customer app and an admin website speak the
same `Order` but are distinct surfaces, each with its own root and flows. A
flow belongs to exactly one system.

This feature adds `System` as a captured domain node (the 25th `NodeKind`),
authorable through the generic `product domain new/edit/rm` path, serialized to
Turtle under `pf:System` with `pf:systemKind`/`pf:purpose`/`pf:targetsPlatform`/
`pf:targetsClass`/`pf:rootsAt`, round-tripped by the seed parser, and validated
for integrity. Flows gain an optional `system` ownership edge (`pf:systemOf`).

## Functional Specification

### Inputs

- `product domain new system <id> --label <name> --system-kind <kind> --purpose <text> [--target-platforms a,b] [--target-classes gui,tui] [--roots-at <application-root-id>]`
- `product domain new flow <id> --label <name> --system <system-id>` — the flow→system ownership edge.

### Behaviour

- A `System` node is captured with its kind, purpose, target platforms, target
  interaction classes, and (optional) root, and appears in `domain list system`,
  `domain show`, and the Turtle export.
- A flow may declare the system it belongs to; the edge is emitted as
  `pf:systemOf` and survives a Turtle round-trip (`seed::from_turtle`).
- `domain validate` accepts a system whose kind and purpose are present and
  whose root (if set) resolves to a declared `ApplicationRoot`.

### Error handling

- A system missing its `kind` or `purpose` is rejected (`§3.2.5`), no change made.
- A system whose `root` does not resolve to an `ApplicationRoot` is rejected.
- A flow whose `system` does not resolve to a declared `System` is rejected.

## Out of scope

- Enforcing that *every* flow is owned by a system (the completeness rule) — the
  ownership edge is validated when present, but unowned flows are not yet a hard
  finding, to avoid invalidating existing What graphs. Deferred to a later pass.
- Per-system page-graph scoping of navigation (one root per system at render
  time) beyond the `rootsAt` edge.
- Deployment identity (domain name, bundle id, runtime) — that is How, carried
  by the infrastructure/runtime contract (§4.2), not the system node.

## Acceptance

- TC-1030, TC-1031 pass.
- `cargo t` and clippy are green; the system round-trips through Turtle.
