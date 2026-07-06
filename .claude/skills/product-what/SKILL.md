---
name: product-what
description: >
  Guide authoring the What — the domain model (§3.1) and event model (§3.2) —
  inside a product session's What phase: bounded contexts, entities, value
  objects, commands, events, read-models, triggers, systems, flows, plus Deciders
  (§3.3) and Projectors (§3.4). Use when the session is in the What phase or the
  user says "model the domain", "add an entity/command/event", "add a system",
  "derive the decider", or "validate the what".
---

# Product Session — the What phase

Author the **What**: one graph with two lanes — **domain** (structure, §3.1) and
**event** (behaviour, §3.2) — with bridge edges crossing between them.

**Author in dependency order: domains → systems → flows.** The domain is the
hardest part and everything references it, so get it right first; then name the
**systems** that reference those domains; only then the **flows** — because a
flow belongs to exactly one system (§3.2.5) and cannot exist without it.

**Precondition:** call `product_workflow_status`; `phase` must be `what`. If not,
use **product-session** to advance/route.

## The question script (author in this order)

1. **Bounded contexts** — what areas of meaning are there?
   → `product_domain_new kind=context`
2. **Aggregates / entities** — what are the aggregate roots? their identity?
   → `kind=entity` (`is_aggregate_root`, `identity`, `context`); describe them with
   `kind=value-object`.
3. **Commands** — per aggregate, what commands target it, and what does each emit?
   → `kind=command` (`targets`, `emits`).
4. **Events** — what does each command emit, and what entity does each event
   *change*? Events are thin (just `changes` + `context`).
   → `kind=event` (`changes`).
5. **Read-models** — what views project which entities/events?
   → `kind=read-model` (`projects`).
6. **Triggers (§3.2.0)** — what's the *source* (user | external | automated)
   issuing each command? → `kind=trigger` (`source`, `issues`).
7. **Systems (§3.2.5)** — name the surfaces that reference the domains (an app,
   website, service, or CLI), *before* any flow — a flow belongs to exactly one
   system. → `kind=system` (`system_kind`, `purpose`, `references_domain`).
8. **Flows (§3.2.4)** — chain trigger → command → event → read-model into named
   flows and assign **system ownership** to a system from step 7. → `kind=flow`
   (`steps`, `system`).
9. **Deciders (§3.3)** — make behaviour executable:
   `product_decider_derive <aggregate>` → `product_decider_validate <id>` →
   `product_decider_simulate`.
10. **Projectors (§3.4)** — the read-model peer:
   `product_projector_derive` → `product_projector_validate`.

Inspect anytime: `product_domain_list`, `product_domain_show <id>`,
`product_domain_context <id>` (assembles a focused bundle). Fix with
`product_domain_edit` / `product_domain_rm`. Relations (`kind=relation`) need a
`rationale`.

## The gate

`product_domain_validate` runs the per-node §3.1/§3.2 shapes → `{ ok, violations }`.
The **strict** graph-level checks (flow ownership §3.2.5, the Command pattern
§3.2.0, view consumption §3.4, the unreifiable seam §4.5) run on the CLI as
`product domain validate --strict` — run that before finalize for full coverage.

When `validate` is green, advance: `product_workflow_advance` → **How** (use
**product-how**).

## Worked micro-example

A one-flow slice, in order — domain first, then the system, then the flow:
`entity e-thing (is_aggregate_root)` → `command cmd-x (targets=e-thing,
emits=ev-x)` → `event ev-x (changes=e-thing)` → `trigger t-x (source=user,
issues=cmd-x)` → `system sys-a (system_kind=service)` → `flow f-x (steps=[cmd-x,
ev-x], system=sys-a)`. Six `product_domain_new` calls, then
`product_domain_validate`.

Guardrails: see **product-session** (locked session; `.product/products/` is user data).
