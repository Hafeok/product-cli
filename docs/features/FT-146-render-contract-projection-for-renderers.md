---
id: FT-146
title: Render contract projection for renderers
phase: 7
status: complete
depends-on:
- FT-135
- FT-141
adrs:
- ADR-085
tests:
- TC-1027
- TC-1028
domains: []
domains-acknowledged: {}
---

## Description

The **render contract** (PREVIEW, `preview/render-contract.schema.md`) is the
*missing half* of the §11 design-system coupling: §11's manifest says **what a
design system can render**; the render contract says **what there is to render**.
They meet at the renderer: `render(render_contract, manifest?) → surface`. This
feature derives the render contract from the captured What page-graph so a
renderer can put a flow's screens on a surface — with a §11 manifest for concrete
components, or at generic wireframe fidelity without one.

## Functional Specification

### Inputs

- The captured What graph for a product (its application root, flows, UI steps,
  read models, content references), and a **flow id** to project.
- Optional `--context` (a declared context of use) and `--locale`.

### Behaviour

- **Pure projection.** The render contract is a read-only projection of the
  What-graph — derived, never hand-authored, regenerated on every change (the
  same discipline as derived diagrams). It cannot drift.
- **Emit the page graph + Abstract UI.** A render contract is the application
  `root` (its destinations = the global navigation), the requested `flow`
  (entry + pages), and one `screen` per UI step in the flow. Each screen carries
  its `projection`, `state_space` + `state_meanings`, and an `elements` list:
  each surface becomes a `display`/binding element, each offer a control that
  `issues` a command and `transitions_to` the next screen, every element
  carrying the WCAG obligations it inherits from its AIO (§3.2.3).
- **Resolve content.** Content keys a screen references resolve through the
  content store for the chosen locale (§4.6); none are emitted as literals.
- **Generic-renderer sufficiency.** The contract is design-system-agnostic — it
  names AIOs, not components — so a generic renderer can draw it at wireframe
  fidelity, and plugging a §11 manifest in swaps the defaults for real
  components without changing the contract.

### Outputs

- The render contract as JSON on stdout (`contract_version: "preview-0"`),
  matching the Preview schema's structure.

### Error handling

- An unknown flow id, or a graph with no application root, is a clear, named
  failure pointing at the missing element.

## Out of scope

- The **scenario** block (simulated Projector output embedded as real data) — the
  page-graph + Abstract-UI projection is emitted; populating screens with
  projected data is a later increment.
- The **renderer** itself (`preview/renderer.html`) — this feature emits the
  contract a renderer consumes, it does not render.
- The §11 **manifest** (FT-141) and the seam verification (FT-140).

## Acceptance

- TC-1027 — `product preview render-contract <flow>` emits a `preview-0` contract
  with the application root's destinations, the flow's entry + pages, and one
  screen per UI step carrying its projection, state space, and AIO-typed elements
  with inherited WCAG obligations.
- TC-1028 — an unknown flow id exits non-zero naming the missing flow; a screen's
  content keys resolve against the content store for the chosen `--locale`.
