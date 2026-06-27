---
name: product-how
description: >
  Guide authoring the How — the §4 architecture contract — inside a product
  session's How phase: the Why cascade (decisions → principles → patterns), the
  application/infrastructure contracts, interfaces, plus archetypes, cells
  (task-types), and work units. Use when the session is in the How phase or the
  user says "specify the how", "add a principle/pattern/decision", "set the app
  contract", "do we need a new archetype", "scaffold a cell/work unit", or
  "dispatch a cell".
---

# Product Session — the How phase

The How is where the engineer takes the frozen What and decides the **realisation**:
the Why cascade, the two contracts, the layout, interfaces — realised through
**archetypes** (a reusable pre-filled How), **cells** (task-types), and **work
units** (§4–§5). This is authoring, not inspection — drive the `*_add` / `*_init`
tools, not just `*_show`.

**Precondition:** `product_workflow_status` → `phase` must be `how`. If not, use
**product-session**.

## The question script

1. **Scaffold** (if no contract yet) — `product_how_init` (keyed to an archetype).
2. **Foundational decisions (§4.1)** — what choices shape everything? Each carries
   rationale and *licenses* principles. → `product_how_add element=decision`
   (`decision`, `rationale`, `licenses[]`).
3. **Principles** — what checkable rules do those decisions license?
   → `product_how_add element=principle` (`statement`, `licensed_by[]`).
4. **Patterns** — what concrete code shapes realise the principles? (a work unit
   emits a pattern) → `product_how_add element=pattern` (`shape`, `realizes[]`).
5. **Application contract (§4.2)** — language, layering, cross-cutting; plus
   *checkable* statements. → `product_how_set target=app-contract`, then
   `product_how_add element=app-statement`.
6. **Infrastructure contract** — concrete frozen resources that satisfy the app
   contract. → `product_how_set target=infra-contract`, `element=resource`.
7. **Interfaces (§4.4)** — published surfaces derived from the domain.
   → `element=interface`.
8. **Refine** — `product_how_edit element=<kind> id=<id> …` patches a Why-cascade
   element (keeps unmentioned fields); `product_how_rm id=<id>` removes one.

### Archetype, cells, work units (§4.3 / §5)

9. **Archetype decision** — does an existing one fit? `product_archetype_list`,
   `product_archetype_show <name>`, `product_archetype_validate`,
   `product_archetype_check` (layout vs the actual tree). If none fits:
   `product_archetype_init <name>` (scaffolds How + layout + an example cell).
10. **Cells** — `product_cell_init <id> [archetype]`; inspect with
    `product_cell_show` / `product_cell_validate`.
11. **Work units** — `product_work_unit_init` then `product_work_unit_edit` (patch
    prompt/model/applies/…); **or generate real ones** with
    `product_cell_dispatch` binding slots to entities (e.g. `binds={entity: Order}`)
    — this is how delivery work units are produced from a cell + the What graph.
12. **Workers** — `product_worker_list` / `product_worker_resolve` confirm the
    capability + role bindings a dispatch will target (read-only).

## The gate

`product_how_validate` → structure + conformance + trace-truth `{ ok, violations }`.
For an archetype, `product_archetype_check` confirms the layout holds against the
repo tree. Every `*_add` / `*_set` / `*_edit` / `*_rm` also re-validates the whole
contract in-loop and returns its `violations`.

When set, advance: `product_workflow_advance` → **Build** (use **product-build**).
**Note:** How *writes freeze* once you enter Build — finish the architecture here.
In particular, **`product_cell_dispatch` is a How tool**: dispatch the work units
your Build deliverables will consume *before* advancing — you cannot dispatch in
Build, and `product_build_run` falls back to unrelated units if a slice has none.

## Worked micro-example

`how_init archetype=demo` → `add decision (id=d-lang, decision="Use Rust",
rationale="safety", licenses=[zero-unwrap])` → `add principle (id=zero-unwrap,
statement="no unwrap in non-test code", licensed_by=[d-lang])` → `add pattern
(id=slice-adapter, shape="pure slice + thin adapter", realizes=[zero-unwrap])` →
`set app-contract (id=demo-app, language=Rust)` → `how_validate`.

Guardrails: see **product-session**.
