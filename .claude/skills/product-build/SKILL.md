---
name: product-build
description: >
  Guide the Build phase of a product session — partition the What into shippable
  slices, define deliverables with checkable acceptance (§7), dispatch cells into
  work units, run the autonomous build, and finalize. Use when the session is in
  the Build phase or the user says "create a slice", "add acceptance", "build the
  deliverable", "dispatch", "run the build", or "finalize the session".
---

# Product Session — the Build phase

Delivery is **partitioning the What graph into shippable slices**; "done" is a
**computed predicate**, not a judgement (§7). Realise each slice through its work
units, gate it with verifications, and finalize.

**Precondition:** `product_workflow_status` → `phase` must be `build`. If not, use
**product-session**.

## The question script

1. **Slice** — which section of the event model ships? Anchor it on the relevant
   nodes. → `product_slice_new id=<s> anchors=[…] depth=N`; inspect with
   `product_slice_show` / `product_slice_context`.
2. **Deliverable** — `product_deliverable_new id=<d> slice=<s>
   acceptance=["<id>: <statement>", …]`.
   **§7.2 (the gotcha):** each criterion is the literal form `id: statement`, and
   the statement must be a **checkable predicate, not a judgement** — e.g.
   `"journal-created: .product/sessions/<id>/workflow.json exists after start"`, not
   "the start works well". Non-predicate criteria are rejected.
3. **Work units must already exist.** `product_build_run` consumes the §5 work
   units for the slice; if none target it, it falls back to *unrelated* units.
   Work units are produced in the **How phase** by `product_cell_dispatch` (bind
   the cell's slots to the slice's entities) — and that tool **freezes once you're
   in Build** (phases are forward-only). So dispatch every unit your deliverables
   will need *before* advancing from How. If you reach Build and one is missing,
   you cannot dispatch here — finalize and fix in a fresh pass.
4. **Dry-run the build first** — `product_build_run deliverable=<d> dry_run=true`.
   It returns the assembled SPMC context, the worker + parallel run plan, the
   verify plan, and the gate status — **with no worker dispatched**. Review it
   before spending a real run.
5. **Build it** — two ways:
   - **In-process worker:** `product_build_run deliverable=<d>` dispatches the
     worker per the role bindings, writes artifacts, runs the gates.
   - **Hand it to a Claude Code session:** `product_build_emit deliverable=<d>`
     returns a self-contained SPMC prompt (frozen What/How/Behaviour/Acceptance +
     the work-unit build plan in order + the verify commands). Save it and run
     `claude -p "$(cat <file>)"` from the repo root, or `product build <d>
     --emit-spmc` to write `.product/build/<d>.spmc.md` directly. The agent builds
     every artifact at its declared path and makes the verify commands pass.
6. **Acceptance verdicts** — bind a runner so the build auto-verifies a criterion:
   `product_deliverable_runner id=<d> criterion=<c> runner=cargo-test args="<test
   filter>"` (or `runner=shell args="<command>"`). Then the §6 verify step runs it
   and records the verdict. Without a runner, record manually with
   `product_deliverable_accept id=<d> criterion=<c> status=passing|failing`.
   `product_deliverable_done` computes whether the deliverable is done (§7.2).
7. **Release (optional)** — `product_release_new` groups deliverables;
   `product_release_done` checks the cut is closed (no dangling dependency).
8. **Finalize** — `product_session_finalize` validates the What, stamps
   provenance, and closes the session. Everything authored in the session (the
   What graph, How, and delivery artifacts alike) is already in canonical
   `.product/` — finalize is the conformance gate, not a promotion.

## The gate

The deliverable's **done predicate** (§7.2): every concept in the slice's footprint
is conformant, every cited verification is green, and the agreed acceptance passes.
The dry-run's gate status shows domain conformance + each acceptance criterion's
state (pending / passing).

## Worked micro-example

`slice_new id=session-start anchors=[cmd-start-session, ev-session-started,
e-session] depth=1` → `deliverable_new id=session-start slice=session-start
acceptance=["starts-in-what: a started session is in phase what immediately after
start"]` → `build_run deliverable=session-start dry_run=true` → review → real run →
`session_finalize`.

Guardrails: **always dry-run before a real build** (it dispatches a worker and
spends tokens). Plus the usual: locked session, `.product/products/` is user data — see
**product-session**.
