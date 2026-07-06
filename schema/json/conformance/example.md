# Conformance seam — worked request/response arrays

Example messages that validate against the four conformance schemas —
`decision-request.schema.json`, `decision-response.schema.json`,
`projection-request.schema.json`, and `projection-response.schema.json` — the
local projections of the **behavioural-conformance wire protocol** (spec
§6.3.1).

A conformance runner is a process that reads a **JSON array of requests on
stdin**, writes a **JSON array of outcomes on stdout — one per request, in the
same order** — and **exits 0**. The array position is the *only* correlation
between a request and its outcome: there is no per-request id, so **ordering is
load-bearing**. Every payload value on this wire is drawn from the **wire scalar
alphabet** (§3.3): `boolean`, 64-bit signed `integer`, and `string` only. A
field a payload schema declares as `number` or `date` (§3.2) travels **as a
string** here — the wire has no native fractional-number or date type.

An **EventRef** / **CommandRef** is either a bare id string (the event/command
with the *empty* payload) or an object `{ "event"|"command": "<id>", "with": {
field: scalar, … } }`; a missing `with` is the empty payload.

## 1. Decider scenarios — request array (in)

Each element is one Decider scenario: the runner folds `given` into **fresh**
aggregate state, then decides the single `when` command against it.

```json
[
  {
    "given": [
      { "event": "PaymentTaken", "with": { "paid_total": "100.00" } },
      { "event": "RefundIssued", "with": { "amount": "60.00" } }
    ],
    "when": { "command": "IssueRefund", "with": { "amount": "40.00" } }
  },
  {
    "given": [
      { "event": "PaymentTaken", "with": { "paid_total": "100.00" } },
      { "event": "RefundIssued", "with": { "amount": "60.00" } }
    ],
    "when": { "command": "IssueRefund", "with": { "amount": "50.00" } }
  }
]
```

Note `paid_total` and `amount` are money — declared `number` in the payload
schema (§3.2) — so they cross the wire **as strings** (`"100.00"`, not
`100.00`), per the scalar alphabet (§3.3).

## 2. Decider scenarios — response array (out)

One outcome per request, **same order**. The first accepts and emits; the second
rejects.

```json
[
  { "emit": [ { "event": "RefundIssued", "with": { "amount": "40.00" } } ] },
  { "reject": "ORDER-REFUND-1" }
]
```

The first scenario refunds 40 against 100 paid / 60 already refunded — within the
paid total — so the Decider **accepts**, and the response carries the resulting
event in `emit`. The second refunds 50, which would push refunds to 110 over a
100 paid total, so the Decider **rejects** and names the invariant
`ORDER-REFUND-1`. An accept that produced no event would be `{ "emit": [] }`. If
a runner ever emitted a response carrying **both** `emit` and `reject`, the
response is a rejection — **reject wins**.

## 3. Projector scenario — request/response pair

A projection request folds `given` through the relevant Projector from its
initial view; the response is the resulting view state.

**Request (in):**

```json
{
  "given": [
    { "event": "PaymentTaken", "with": { "paid_total": "100.00" } },
    { "event": "RefundIssued", "with": { "amount": "60.00" } }
  ]
}
```

**Response (out):**

```json
{
  "paid_total": "100.00",
  "refunded_total": "60.00",
  "fully_refunded": false
}
```

The view is a flat object of `field → scalar`. `paid_total` and `refunded_total`
are `number`-declared fields, so they appear **as strings**; `fully_refunded` is
a genuine wire `boolean`. Equality against the expected view is **full-state**:
the response must match the expected object *exactly*. Adding an unexpected
`last_refund_at` field, or dropping `fully_refunded`, is **as non-conformant as a
wrong value** — a superset is not a pass. That is why the response schema
constrains value types but leaves the field set to the full-state comparison
rather than to `additionalProperties: false`.

## What a conformance runner must do to be conformant (spec §6.3.1)

1. Read a **JSON array of requests on stdin**; discriminate each by shape (a
   decision request has `when`, a projection request does not).
2. For each request, start from **fresh state** — fold `given`, then decide
   `when` (Decider) or return the folded view (Projector).
3. Write a **JSON array of outcomes on stdout**, exactly **one per request, in
   the same order** — position is the only correlation.
4. Keep every payload value inside the **wire scalar alphabet** (§3.3):
   `number`/`date` fields as strings; `boolean` and `integer` native.
5. **Exit 0.** A non-zero exit is a runner failure, not a non-conformance
   verdict — non-conformance is expressed in the outcome array (a `reject`, or a
   view that fails full-state equality), not in the exit code.
```
