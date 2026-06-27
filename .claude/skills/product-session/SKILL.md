---
name: product-session
description: >
  Launch, resume, and drive a phase-gated What→How→Build workshop session over the
  Product framework graph, and route to the right phase skill. Use when the user
  says "start a product session", "run a workshop session", "product session
  start", "product mcp --workflow", "resume the session", "advance the phase", or
  "finalize the session". Owns start / status / advance / finalize and the
  locked-session guardrails; hands the actual authoring to product-what,
  product-how, and product-build.
---

# Product Session — the launcher

A **session** is a phase-gated facilitation run over an *isolated copy* of
`.product/`. It starts in **What**, advances strictly forward **What → How →
Build** (optionally capped), and on **finalize** validates the draft and promotes
the workspace into the canonical `.product/`. Nothing touches canonical until
finalize. The phase **gates which tools exist** — an out-of-phase tool call errors.

## Starting a session

```bash
product session start [<product>] [--until what|how|build] [--cli claude|copilot]
product session start <product> --no-launch    # scaffold only; prints the run command
```

`--no-launch` prints: `product mcp --workflow --session <id> --repo <root>` — the
phase-gated MCP server the agent drives. Also: `product session list`,
`product session show <id>`, `product session resume <id>` (relaunch + reprint the
live-view URL).

## Always orient first

Before any authoring, call **`product_workflow_status`**. It returns `phase`,
`until`, `availableTools`, `finalized`, and a `hint`. Route on `phase`:

| phase | hand off to | authoring families |
|---|---|---|
| `what` | **product-what** | `product_domain_*`, `product_decider_*`, `product_projector_*`, `product_primitive_*` |
| `how` | **product-how** | `product_how_*`, `product_archetype_*`, `product_cell_*`, `product_work_unit_*`, `product_worker_*` |
| `build` | **product-build** | `product_slice_*`, `product_deliverable_*`, `product_release_*`, `product_build_run` |

## Advancing

`product_workflow_advance` moves to the next phase (or jump with `to`). **Only
advance when the current phase's gate is green** — the phase skill says how to
check. Advancing freezes earlier-phase *writes* (e.g. you cannot edit the How once
in Build), so finish a phase before leaving it.

## Finalizing

`product_session_finalize` validates the draft What and, if conformant, promotes
the isolated workspace into the canonical `.product/`. It writes canonical files —
treat it as the commit point. Build-phase artifacts (slices/deliverables created
in-session) may not promote; the What graph + How always do.

## Guardrails (apply in every phase)

- **Locked session.** Do **not** make source-code edits or hand-edit files while a
  session is running. Finish the graph through the MCP/CLI tools; do code changes
  after you exit/finalize.
- **User data.** Never `rm` or bulk-`git add` `.product/author-domain/` — it's the
  captured What and is the user's.
- **Phase gating is real.** If a tool "belongs to another phase", advance first;
  don't fight the gate.

## Reference

The standard is `docs/product-framework-open.md` (§3 What, §4 How, §5–7 Build).
The live Event-Modeling view is served by the workflow server (URL printed on
start / `session show`).
