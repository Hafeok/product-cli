# Live-Wire Completion Plan — the 1.7.0 explorer ⇄ the live graph

## Status (as implemented)

`GET /api/pf` projects the live graph into `window.PF`; `app.jsx` fetches it before
first render + on the SSE `changed` tick, merging the fields in `PF_LIVE_KEYS`.

**Live (verified headless against the real product-cli graph):**
- Graph: Everything
- What: Systems map · Domain ER · Flows · Deciders · Scenarios · Projectors
- UI §3.2: AIO catalog (live reification + WCAG) · Pages (page graph) · Steps (spec sheets) · Screens (render contract)
- How §4: Systems (blueprint / DeployableUnits / why-cascade) · Patterns · contracts · standards · **Layout rules**
- Build §5–6: Work units (live SPMC bundles)
- Delivery §7: Features (real §7.2 done) · Versions
- Per-node conformance dots computed from real verdicts (feature_done / decider conform / release)

**Still on demo data — the genuine data boundary (the graph carries no such data):**
- **Data** (refData / oracle) — 0 reference-sets / production-datasets in the graph.
- **Content** — 0 content-stores modelled (renders live-but-empty).
- **Composition** (§4.5 Atomic-Design narrative) · **Process** (H1–H6 companion doc) — fixed narrative, not graph-derived.
- **Reification** (design-system manifest) — derivable from design-systems + reification-rules; not yet projected.
- **Verifications** (§6.3) — the required kinds are framework-universal (demo is representative); per-product standings need real verification results.
- **Layout's repo-tree pane** — needs a live repository scan against the layout rules.

Every view renders (never crashes). Each remaining item becomes live the same way
the shipped ones did: project its field → add the key to `PF_LIVE_KEYS`.

## Per-product scoping + product picker
- **Per-product artifacts.** The projection resolves a per-product base
  `.product/products/<product>/` (falling back to the shared `.product/` for the
  self-hosted product-cli), so a product's How/Delivery/Build artifacts —
  blueprint, DeployableUnits, how-contract, features/deliverables/releases/
  targets, work-units, deciders — are its own. ACME ships its own
  `acme-storefront` blueprint + 3 DeployableUnits + why-cascade + deliverables
  (`scripts/showcase-acme-how.sh`), so its How/Delivery views are acme-specific
  while product-cli's stay unchanged.
- **Product picker.** `/api/pf` + `/api/graph` take `?product=`; the banner shows
  a picker whenever more than one product has a captured What graph, switching
  product-cli ⇄ acme (and any future product).

## Since then
- **Live repo-scan** for the §4.3 Layout tree (`pf_view/pf_repo.rs`) — walks the
  real repo files the blueprint's layout rules cover, attributes each to its rule,
  emits an indented tree with per-file verdicts. Layout is now fully live.
- **ACME Shop showcase/test product** (`scripts/showcase-acme.sh`) — authors a
  strict-conformant second product (51 nodes: 2 domains, 2 systems, entities/VOs/
  invariants, full event model, AIOs/WCAG/reification/reference-data, ui-steps/
  page-graph, a cross-system journey, quality demands) into
  `.product/author-domain/acme/` via the real CLI, exercising the whole authoring
  surface end-to-end. View it in the explorer at **`/?product=acme`** (the
  `/api/pf` + `/api/graph` handlers now take a `?product=` override). This also
  makes **Data** (reference-sets) and the UI reification views live for acme.

## North star (definition of done)
Every view renders the real `.product/` graph, refreshes over SSE, and shows honest
conformance — with **no bundled demo data consumed at runtime** (demo becomes an
offline-only fallback). `PF_LIVE_KEYS` contains every top-level field; a contract test
guards backend↔UI drift.

## The core finding
The new UI's views are **bespoke layouts hardcoded to the acme demo** (fixed pixel
coordinates, demo ids — e.g. `DomainGraph` uses a `PLACE[id]` map + literal "Ordering"
title; `SystemsMap` used `pos.acme`). They are *not* generic data-driven renderers.
"Complete live-wire" therefore = **rewrite each view to auto-layout live data**
(`SystemsMap` is the template) **+** project the remaining data fields.

## Guiding principles
- **Each view ships end-to-end in one slice**: project its data → rewrite the view
  data-driven → add its key to `PF_LIVE_KEYS` → verify headless (0 console errors +
  a live-data assertion) → commit. Never half-wire (that caused the Versions crash).
- **Backend stays under the fitness gates**: split `pf_view/` into per-section modules
  (`pf_flows`, `pf_how`, add `pf_ui`, `pf_build`, `pf_delivery`, `conformance`);
  functions ≤40 stmts, files ≤400 lines.
- **Layout once, reuse everywhere**: a shared auto-layout toolkit makes each view
  rewrite ~40 lines, not ~150.

---

## Phase 0 — Foundations (unblocks all rewrites)
| Item | Where | Notes |
|---|---|---|
| Layout toolkit | `assets/ui/shared.jsx` → `PFUI.layout` | `rowLayout`, `gridLayout`, `layeredColumns` (longest-path, already in `pf_flows`), `orthogonalEdges`/routing. Ported/generalized from the old `view.html`. |
| Conformance model | new `pf_view/conformance.rs` | `conformance_of(id) -> described\|realised\|verified\|delivered` from graph + `.product` verdicts (feature_done §7.2, decider `.conform.json`, deliverable acceptance, release membership). Replaces hardcoded `"realised"`. |
| PF contract test | `pf_view` tests | Golden test: `/api/pf` emits every key the UI reads (drift guard). |

## Phase 1 — The old-view trio (Domain, Flows, Graph)
| View | Backend | Frontend | Effort |
|---|---|---|---|
| Domain ER | `domain` ✅ (add all-contexts + selector) | rewrite `DomainGraph.jsx` auto-layout; read `contextId` for title | M |
| Flows | `flows` ✅ (computed lanes/cols) | rewrite `FlowsTimeline.jsx` to consume computed layout | M |
| Everything | ✅ (live systems/domains) | verify `GraphView`/`buildGraph`; add to allowlist | S |

## Phase 2 — Behaviour (Deciders, Scenarios)
| View | Backend | Frontend | Effort |
|---|---|---|---|
| Deciders | enrich `deciders`: per-command `handles` (from `logic`), real `stateRead`/`rejections`/`coverage` | rewrite `DecidersView` | M |
| Scenarios | new: `scenarios` from each Decider's `scenarios` (+ simulate verdicts) | rewrite `ScenariosView` | M |

## Phase 3 — Delivery (Features, Versions)
| View | Backend | Frontend | Effort |
|---|---|---|---|
| Features | enrich `delivery.features`: friendly name, real `done` (`pf::done::feature_done`), derived footprint closure, per-clause status | rewrite `FeaturesView` | M |
| Versions | enrich `delivery.versions` from product/how versions + targets; derive `bump`/`diff` | rewrite `VersionsView` | M |

## Phase 4 — The How (§4)
| View | Backend (`pf_how`) | Frontend | Effort |
|---|---|---|---|
| How · Systems | ✅ (enrich patterns.files/rules from layout; real blueprint→system) | rewrite `HowViews` blueprint/DU/cascade | M |
| Patterns | `patterns` ✅ | rewrite | S |
| Layout | new: `how.layout` from blueprint `layout.yaml` + `repoTree` (real repo scan vs rules) | rewrite | M |
| Composition / Reification | new: `how.contracts`, `how.standards`, `composition`, `manifest` from app/infra contracts + design-systems + reification-rules | rewrite | L |
| Process | new: `howProcess` (H1–H6 from binding state) | rewrite `HowProcessView` | M |

## Phase 5 — UI section (§3.2) — largest; new `pf_view/pf_ui.rs`
| View | Backend | Frontend | Effort |
|---|---|---|---|
| AIOs | `aios` + `aioUsage` | rewrite `AIOCatalogView` | M |
| Steps | `stepSpecs` from wireframe-steps | rewrite `UIStepsView` | M |
| Pages | `pageGraph` from application-roots + navigate edges | rewrite `PageGraphView` | M |
| Screens | `contract` (screen composition) from steps + reification + cios | rewrite `ScreenPreview` | L |
| Content | content store + `resolveContent` | rewrite `ContentView` | M |
| (data) | `wcag`, `refData`, `oracle` from wcag-criteria / reference-sets / primitives | consumed above | M |

## Phase 6 — Build (§5–6)
| View | Backend (`pf_view/pf_build.rs`) | Frontend | Effort |
|---|---|---|---|
| Work units | `workUnits` from `.product/work-units/` (bundle/hash) + build verdicts | rewrite `BuildViews` | M |
| Verifications | `verificationKinds` §6.3 + real standings | rewrite `BuildViews` | M |

## Phase 7 — Completeness & hardening
- Remove `PF_LIVE_KEYS` gate (all keys live); demo = offline fallback only.
- Empty-state polish for every view (e.g. 0 DeployableUnits).
- Model blueprint→system→DeployableUnit edges explicitly (graph can't currently say which blueprint realises which system).
- SSE reconnect/backoff + indicator.
- Contract-drift test in CI; `/api/pf` shape smoke test.

## Sequencing
`0 → 1 → 3 → 2 → 4 → 5 → 6 → 7` (Delivery before Behaviour: mostly ready + high-visibility).
Total ≈ 20 view rewrites + ~18 projection fields + conformance/layout foundations.

## Decisions (defaults, pending confirmation)
- **Execution:** Phase 0 with review, then autonomous run through Phases 1–6, check in at phase boundaries.
- **Conformance:** computed from real verdicts (Phase 0 `conformance.rs`).
- **Tracking:** this doc + a tracked task list.
