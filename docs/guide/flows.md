# Flows — the end-to-end recipes

Each section is a complete recipe for one flow, with the exact commands. They
chain: the output of one is the input of the next. Lost mid-flow? Run
**`product guide`** — it always names your next step. For the vocabulary see
[concepts](framework-concepts.md); for the daily commands see
[everyday-use](everyday-use.md).

The running example is a **bookstore**. Seed it any time with
`product init -y --name bookstore --demo`.

---

## Flow A — Capture the What

The What is your product's domain and behaviour, agreed before any How. Two ways.

### A1. By hand (full control)

Author in dependency order — structure first, then behaviour:

```bash
product domain new context Catalog --label "Catalog" --purpose "Browse and buy books"
product domain new entity  Order   --label "Order"   --context Catalog \
  --definition "A customer order" --aggregate-root true
# event before the command that emits it
product domain new event   OrderPlaced --label "Order placed" --context Catalog --changes Order
product domain new command PlaceOrder  --label "Place order"  --targets Order --emits OrderPlaced
product domain new read-model OrderSummary --label "Order summary" --projects OrderPlaced

product domain validate     # → conformant — 5 node(s), 0 violations
```

### A2. Facilitated (an LLM scribes for you)

```bash
product author domain bookstore
```

This launches your configured agent CLI as a **scribe**: it interviews the room
and enters every fact through validated tool calls (you never write Turtle). A
non-conformant graph is impossible — the tools reject bad input in-loop. The
session ends with `session_finalize`, which exports conformant Turtle plus a
provenance record (participants, content hash). Preview the facilitation script
with `product prompts get author-domain`.

**Milestone:** a green `product domain validate`. That's the gate to the How.

---

## Flow B — Author the How

The How realises the What without changing its meaning. It can only reference
What that exists.

```bash
product how init                                  # scaffold how-contract.yaml
product how add decision  --label "Rust core"     --rationale "Single static binary, no runtime"
product how add principle --label "Pure core"     --rationale "Decision logic isolated from I/O"
product how add interface --label "REST checkout"
product how set app-contract --id "checkout-app"
product how validate                              # contract obeys the What
product how show                                  # read it back
```

Keep each decision single-responsibility, each with its own rationale and
rejected alternatives. Preview the facilitation with
`product prompts get author-how`.

---

## Flow C — Make behaviour executable

Where behaviour is interesting, derive a **Decider** (for an aggregate) or a
**Projector** (for a read model). The signature is *derived* from the event
model; you then author the logic and scenarios, and simulate it sound **before
any code exists**.

**Step 1 — derive the signature** (free, by construction):

```bash
product decider derive Order        # → writes .product/deciders/Order-decider.yaml
product decider list
```

The derived file is signature-only — `handles` / `emits` / `evolves_from`. Simulating
it now reports *"needs authored logic + scenarios"*: that's the honest next step.

**Step 2 — author the logic and scenarios** by editing the derived YAML. The
logic is a small **guarded state machine** (ADR-062); guards are structured
predicates or **CEL** expressions tied to the invariant they protect (ADR-063);
scenarios are the oracle — *given* prior events, *when* a command, *then* events
or a rejection:

```yaml
# .product/deciders/Order-decider.yaml  (append to the derived signature)
logic:
  initial: { item_count: 0 }
  decide:
  - on: PlaceOrder
    guards:
    - expr: "command.item_count > 0"     # a CEL guard …
      else_reject: OrderHasItems          # … tied to a real invariant id
    emit:
    - OrderPlaced
  evolve:
  - on: OrderPlaced
    set: { item_count: 1 }
scenarios:
- name: an order with items is placed
  given: []
  when: { command: PlaceOrder, with: { item_count: 2 } }
  then: { emit: [OrderPlaced] }
- name: an empty order is rejected
  given: []
  when: { command: PlaceOrder, with: { item_count: 0 } }
  then: { reject: OrderHasItems }
```

(`else_reject` must name an invariant in your What — add one with
`product domain new invariant OrderHasItems --context <ctx> --applies-to Order --statement "…"`.)

**Step 3 — prove it sound before any code:**

```bash
product decider simulate Order-decider   # → sound + complete — 2 scenario(s) over 1 command(s)
```

`simulate` runs every scenario through the authored state machine and checks the
Decider is **sound** (no scenario contradicts the logic) and **complete** (every
handled command is exercised). The same scenarios are the oracle reused *after*
realisation by `product decider conform <name> --runner …` (§6.3) — author once,
check twice.

**Projectors are symmetric** — derive the fold, author its `project` logic +
scenarios, simulate:

```bash
product projector derive OrderSummary     # → 'OrderSummary-projector'
product projector simulate OrderSummary-projector
```

> Naming: `derive <aggregate>` creates `<aggregate>-decider`; `simulate` / `show` /
> `validate` / `conform` take that **derived name**, not the aggregate. Same for
> projectors (`<read-model>-projector`).

For a genuinely irreducible computation (a hash, a solver), declare a
**named-algorithm primitive** instead (`product primitive …`) — specified by
reference plus an oracle, not pretended to be derivable.

---

## Flow D — Delivery and build

Carve a buildable slice, wrap it as a deliverable, and build.

```bash
product slice new checkout --anchor PlaceOrder     # anchor at a command/context/flow
product slice show checkout
product deliverable new checkout-v1 --slice checkout
product deliverable accept checkout-v1 --pass ...  # record acceptance verdicts
product deliverable done checkout-v1               # computes §7.2 'done' as a %
```

`deliverable done` reports a percentage with a checklist (domain conformance per
node, plus acceptance criteria) — it's a computed predicate, not a hand-flip.
When ready:

```bash
product build checkout-v1     # assemble the frozen SPMC context, run the gates
```

`build` assembles the frozen build context (slice + What + How + deciders) and
dispatches a worker to realise it against the verification gates. Group
deliverables into a `product release` when shipping several together.

---

## The whole arc, at a glance

```
author domain / domain new   →  domain validate   →  how init / how add
        (What)                     (the gate)            (How)
                                                            │
        decider derive / simulate  ◄───────────────────────┤
        (executable behaviour)                              │
                                                            ▼
        slice new  →  deliverable new  →  deliverable done  →  build
                          (Delivery)
```

`product guide` walks you along this arc one step at a time.
