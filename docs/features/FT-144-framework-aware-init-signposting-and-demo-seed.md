---
id: FT-144
title: framework-aware init — signposting and a --demo bookstore seed
phase: 8
status: complete
depends-on:
- FT-035
- FT-143
adrs:
- ADR-088
- ADR-048
tests:
- TC-1020
domains:
- api
- data-model
domains-acknowledged:
  ADR-049: Not a context-bundle/template command; no template surface changes.
  ADR-047: The functional specification lives in this feature's body, not a separate artifact.
  ADR-043: The seed lives in the pure `product_core::demo` slice; the init handler is a thin adapter.
  ADR-041: Additive — a new `--demo` flag and a richer next-steps message; the prior behaviour (config + dirs) is unchanged, so no removal/absence TC is required.
  ADR-040: The seed reuses the validated `pf::edit::create` path and writes only the domain session; the verify pipeline is untouched.
  ADR-050: PAT-001 (slice + adapter) is cited via `patterns:`; no new implementation pattern is introduced.
  ADR-051: The TC declares `observes:` (stdout, exit-code) and asserts on those surfaces, and on the seeded graph via `domain validate`.
  ADR-018: A scenario TC drives the binary through assert_cmd; `product_core::demo` carries a unit test that reloads + asserts conformance. No property or session dimension.
  ADR-042: The TC uses the reserved `scenario` type only; no new TC type is introduced.
patterns:
- PAT-001
---

## Description

`product init` previously dead-ended a new user at the meta graph — it scaffolded
`features/adrs/tests` and said `Run product feature new …`, with the framework
graph (What/How/Delivery) unmentioned and undiscoverable. This feature makes init
framework-aware: it prints a **Next steps** block pointing at `product guide` and
`product author domain` alongside `product feature new`, and adds a **`--demo`**
flag that seeds a small, conformant "bookstore" What model so a workshop
participant has a real model to explore in seconds.

## Functional Specification

### Inputs

- The existing `product init` options, plus `--demo` (a boolean flag).

### Behaviour

- After scaffolding, print a **Next steps** block:
  - default: `product guide`, `product author domain <name>`, `product feature new …`.
  - `--demo`: `product status`, `product guide`, `product domain show Order`.
- `--demo` seeds the bookstore What model via `product_core::demo::seed_bookstore`
  (a Catalog context, Book/Order entities, an OrderPlaced event, a PlaceOrder
  command, an OrderSummary read model) — six conformant nodes, reusing the
  validated `pf::edit::create` authoring path.

### Error handling

- A node the seed cannot create surfaces a clear error naming the broken rule
  (defensive — the curated demo is conformant by construction).

## Out of scope

- The `guide` command itself (FT-143) and the framework lifecycle commands.

## Acceptance

- TC-1020 — `product init --yes --name bookstore --demo` exits 0, reports the
  seeded demo, and the seeded graph passes `product domain validate`; `product
  guide` then shows the What captured and conformant.
