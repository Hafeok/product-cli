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
   `"ws-copies-canonical: ws/.product is a copy of canonical after start"`, not
   "the start works well". Non-predicate criteria are rejected.
3. **Work units** — make sure the slice actually has work units. If none target
   it, `product_build_run` falls back to *unrelated* units. Produce the right ones
   with `product_cell_dispatch` (bind the cell's slots to the slice's entities).
4. **Dry-run the build first** — `product_build_run deliverable=<d> dry_run=true`.
   It returns the assembled SPMC context, the worker + parallel run plan, the
   verify plan, and the gate status — **with no worker dispatched**. Review it
   before spending a real run.
5. **Real build** — `product_build_run deliverable=<d>`. Dispatches the worker per
   the role bindings, writes artifacts, runs the gates.
6. **Acceptance verdicts** — `product_deliverable_accept id=<d> criterion=<c>
   status=passing|failing`; `product_deliverable_done` computes whether the
   deliverable is done (§7.2). Criteria with a runner are auto-checked; others are
   recorded manually.
7. **Release (optional)** — `product_release_new` groups deliverables;
   `product_release_done` checks the cut is closed (no dangling dependency).
8. **Finalize** — `product_session_finalize` validates the What and promotes the
   workspace into canonical `.product/`. The What graph + How promote; in-session
   slices/deliverables may not — recreate canonical delivery artifacts outside the
   session if you need them persisted.

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
spends tokens). Plus the usual: locked session, `author-domain` is user data — see
**product-session**.
