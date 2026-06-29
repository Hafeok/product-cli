# Build-seam — worked message pair

Two example messages that validate against `workunit.schema.json` and
`verdictevent.schema.json`. They show the **universal envelope** (what every
executor must understand) and the **one opaque slot** (`executor_extension`) where a
particular execution framework — here a tiered, cell-DAG executor — puts structure
the framework neither reads nor mandates.

The point to take away: strip `executor_extension` from both messages and they are
still complete, reconcilable, and conformant. The framework's contract lives
entirely in the envelope; the slot is where one executor's model lives without the
framework knowing about it.

## 1. Emitted WorkUnit (specification → execution)

```json
{
  "unit_ref": "wu-checkout-refund-decider-0007",
  "parent_lineage": "deliverable-refunds",
  "bundle_hash": "b3d1f2a9c7e84416a0f5d2c8e1b6079a4d33f8e2c1a90b7765e4d3c2b1a09f8e7",
  "bundle": {
    "schema": {
      "artifact_type": "module",
      "language": "typescript",
      "acceptance_criteria": [
        "exports a pure decide(order, cmd) function",
        "rejects ORDER-REFUND-1 when refund_total + amount > paid_total",
        "passes the frozen simulation scenarios in context.simulation"
      ]
    },
    "prompt": {
      "transformation": "Realise the Order refund Decider from its specification.",
      "produces": "the decide() implementation only; no I/O, no persistence"
    },
    "model": {
      "capability": "code-implementation",
      "note": "abstract capability label; the executor maps this to a resident producer"
    },
    "context": {
      "decider_spec": { "ref": "Order#decide", "resolved": "...inlined decider spec..." },
      "invariants": [ { "id": "ORDER-REFUND-1", "rule": "refund_total <= paid_total" } ],
      "domain_elements": [ "Order", "Money" ],
      "simulation": [
        { "given": ["paid 100", "refunded 60"], "when": "refund 50", "then": "Rejected(ORDER-REFUND-1)" },
        { "given": ["paid 100", "refunded 60"], "when": "refund 40", "then": "Accepted[ev-refund-issued]" }
      ]
    }
  },
  "acceptance_class": "needs-verdict",

  "executor_extension": {
    "kind": "spark-substrate",
    "tier": "coder-medium",
    "ladder_position": 0,
    "cell_graph": {
      "cells": ["write-test", "implement", "gate"],
      "edges": [["write-test", "implement"], ["implement", "gate"]]
    }
  }
}
```

Everything above `executor_extension` is universal. The `tier`, `ladder_position`,
and `cell_graph` are one executor's model of *how* it will run the unit — capability
tiers, an escalation ladder, a sealed interior dependency graph. A different executor
with none of those concepts ignores the slot entirely and still runs the unit from
the envelope alone.

## 2. Returned VerdictEvent (execution → stream)

```json
{
  "event_id": "ev-9f81c0a4-7b22-4e6d-9b3a-1c2d3e4f5a6b",
  "emitted_at": "2026-06-26T02:14:08Z",
  "unit_ref": "wu-checkout-refund-decider-0007",
  "parent_lineage": "deliverable-refunds",
  "bundle_hash": "b3d1f2a9c7e84416a0f5d2c8e1b6079a4d33f8e2c1a90b7765e4d3c2b1a09f8e7",
  "verdict": "accepted",
  "next_consequence": "advance",

  "executor_extension": {
    "kind": "spark-substrate",
    "tier_ran": "coder-medium",
    "cell_results": [
      { "cell": "write-test", "verdict": "accepted" },
      { "cell": "implement",  "verdict": "accepted" },
      { "cell": "gate",       "verdict": "accepted" }
    ]
  }
}
```

The `emitted_at` of `02:14:08Z` is a night emission — it sat in the stream until the
consumer next read it, with no executor-side wait (temporal decoupling). The
`bundle_hash` matches the unit exactly, so the `accepted` verdict is attributable to
the immutable input it ran against. The `cell_results` are this executor's evidence;
a consumer may act on `verdict` alone and ignore them.

## What an execution framework must do to be conformant (spec §5.1)

1. Given a WorkUnit, reach a declared `verdict` with **no further input** — resolve
   nothing by calling back.
2. Emit the verdict as a **self-describing** VerdictEvent — its `unit_ref`,
   `bundle_hash`, and `verdict` are sufficient for a consumer that never saw the
   emission to reconcile it.
3. Hold **no knowledge of any consumer** — emit to the stream and be done.
4. Keep the `bundle_hash` it ran against, so every verdict is **attributable to an
   immutable input**.

Anything beyond these four — scheduling, batching, tiers, ladders, how the artifact
is actually produced — is the executor's own concern and goes in
`executor_extension` if it crosses the seam at all.
