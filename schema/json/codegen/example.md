# Code-generation seam — worked manifest / file-plan pair

Example messages that validate against `manifest.schema.json` and
`file-plan.schema.json` — the local projections of the **code-generation seam** (spec
§5.2), the boundary between the deterministic oracle and a language backend. (Not
reification: `reify(AIO,context)→CIO` (§4.5) is a different boundary; this seam
turns the whole oracle into a target-language source tree.)

**Outbound** crosses the **codegen manifest**: the *whole oracle by value* —
payload schemas, Decider/Projector scenarios, flow chains with every step's
outcome pre-computed, and screen facts. **Inbound** returns a **file plan**: a
flat list of files to write. A backend is any process that reads the manifest on
**stdin** and answers the file plan on **stdout** — a pure function of the
manifest, calling nothing back.

## 1. Codegen manifest fragment (oracle → backend)

Trimmed to one aggregate for readability; a real manifest carries every
aggregate, flow, and screen.

```json
{
  "manifest_version": "1",
  "graph_hash": "sha256:81e5387c02c05219c6ecfd8f19a72d88793b15447b6ff5059af37bb61dccce5b",
  "payload_schemas": {
    "Order": {
      "events": {
        "PaymentTaken": { "paid_total": "number" },
        "RefundIssued": { "amount": "number" }
      },
      "commands": {
        "IssueRefund": { "amount": "number" }
      },
      "view": {
        "paid_total": "number",
        "refunded_total": "number",
        "fully_refunded": "boolean"
      }
    }
  },
  "decider_scenarios": [
    {
      "aggregate": "Order",
      "given": [
        { "event": "PaymentTaken", "with": { "paid_total": "100.00" } },
        { "event": "RefundIssued", "with": { "amount": "60.00" } }
      ],
      "when": { "command": "IssueRefund", "with": { "amount": "50.00" } },
      "then": { "reject": "ORDER-REFUND-1" }
    }
  ],
  "projector_scenarios": [
    {
      "projector": "Order",
      "given": [
        { "event": "PaymentTaken", "with": { "paid_total": "100.00" } },
        { "event": "RefundIssued", "with": { "amount": "60.00" } }
      ],
      "then": { "paid_total": "100.00", "refunded_total": "60.00", "fully_refunded": false }
    }
  ],
  "flows": [
    {
      "id": "flow-refund",
      "steps": [
        {
          "when": { "command": "IssueRefund", "with": { "amount": "40.00" } },
          "then": { "emit": [ { "event": "RefundIssued", "with": { "amount": "40.00" } } ] }
        }
      ]
    }
  ],
  "screens": [
    {
      "id": "screen-order-summary",
      "surfaces": ["Order"],
      "offers": [ { "command": "IssueRefund" } ],
      "degraded_states": [
        { "id": "refund-cap-reached", "description": "Paid total fully refunded; the refund offer is disabled." }
      ],
      "present_state": { "paid_total": "100.00", "refunded_total": "60.00", "fully_refunded": false }
    }
  ]
}
```

Every money field (`paid_total`, `amount`, `refunded_total`) is **declared**
`number` in `payload_schemas` (§3.2) but travels **as a string** in every
payload and view above — the wire scalar alphabet (§3.3) has no fractional-number
type. `fully_refunded` is a genuine wire `boolean`. The Decider outcome is
**pre-computed** by the oracle (`then` names the rejected invariant), as is the
Projector's full-state view and each flow step — the backend reproduces them, it
never runs the oracle. `graph_hash` is the graph state the whole manifest was
computed from.

## 2. Codegen file plan (backend → oracle/consumer)

```json
{
  "files": [
    {
      "path": "src/order/order.decider.ts",
      "content": "// @generated from graph sha256:81e5387c… — do not edit by hand\nexport function decide(state, cmd) { /* … */ }\n"
    },
    {
      "path": "src/order/index.ts",
      "content": "// realiser-owned entry point — wire your adapters here\nexport * from './order.decider';\n",
      "overwrite": false
    }
  ]
}
```

The first file is **generated**: it carries a graph-hash provenance header the
backend rendered from the manifest's `graph_hash`, and it omits `overwrite`, so
it defaults to overwritable and is regenerated every run. The second is a
**realiser-owned scaffold**: `overwrite: false` means the consumer writes it once
if absent and then **never regenerates** it, so a human's edits to the entry
point survive future runs.

Both paths are **relative and contained** — no leading `/`, no `..` segment — so
the plan can only write inside the output root the consumer owns; the `path`
pattern rejects anything else structurally.

## What the consumer does that the backend does not (spec §5.2, §7.3.1)

The backend returns *only* file bodies. The **consumer** appends the
**provenance manifest** (§7.3.1) to what it materialises — recording the
`graph_hash` the files were generated from — so the **drift gate covers every
backend identically**. A backend cannot opt out of provenance, forge it, or
implement it differently, because provenance is never the backend's to write. The
manifest is versioned (`manifest_version: "1"`); a backend that does not
understand the version rejects the manifest rather than guessing.
```
