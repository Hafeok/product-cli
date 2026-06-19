# Getting started — model a product as What, How, and Delivery

This is a hands-on tour of the **framework graph**: you'll go from an empty
directory to a small, conformant product model in about fifteen minutes. We'll
model a tiny **bookstore**. Every command below is real — copy them as you go.

> New to the vocabulary (What/How, bounded context, command/event, decider)?
> Skim [framework-concepts.md](framework-concepts.md) first — one paragraph each.

At any point you are lost, run **`product guide`**. It reads your graph and
tells you exactly which step you're on and the next command to run.

---

## 0. Install and initialise

```bash
product init --name bookstore
```

This creates `.product/` — the home for your graph. Now ask the guide where to
start:

```bash
product guide
```

```
── Your framework journey ──
  [ ] Captured a What model
  [ ] What is conformant
  [ ] How contract scaffolded
  [ ] Delivery slice carved
  [ ] Deliverable wrapped

Start by capturing your product's What — its domain and behaviour.
Next:
  $ product author domain bookstore
```

There are two ways to capture the What. **Facilitated** (recommended for a real
product) launches an LLM that interviews you and scribes the graph:

```bash
product author domain bookstore     # interactive — needs an agent CLI
```

For this tutorial we'll author **by hand** so every node is explicit.

---

## 1. The What — structure (the domain)

Start with a **bounded context** — a region of the product with its own
language — then the **entities** inside it. Authoring is validated *as you go*:
a node that breaks a framework rule is rejected, not saved.

```bash
product domain new context Catalog --label "Catalog" --purpose "Browse and buy books"

product domain new entity Book  --label "Book"  --context Catalog \
  --definition "A book offered for sale" --aggregate-root true
product domain new entity Order --label "Order" --context Catalog \
  --definition "A customer order" --aggregate-root true
```

> **Required fields matter.** A context needs `--label`; an entity needs
> `--label`, `--context`, and a business-language `--definition`. Miss one and
> the node is rejected with the rule it broke — that's the type system working,
> not a bug.

## 2. The What — behaviour (commands and events)

Behaviour is **commands** users issue and the **events** they cause. The golden
rule: *every event changes a real entity, every command targets an aggregate and
emits an event.* So author in dependency order — **event first**, then the
command that emits it:

```bash
product domain new event   OrderPlaced  --label "Order placed" \
  --context Catalog --changes Order
product domain new command PlaceOrder   --label "Place order" \
  --targets Order --emits OrderPlaced
product domain new read-model OrderSummary --label "Order summary" \
  --projects OrderPlaced
```

Check your work:

```bash
product domain list
product domain validate      # → "conformant — 6 node(s), 0 violations"
```

If `validate` reports violations, `product guide` will route you to fix them
before going further. A green `validate` is the gate between What and How.

---

## 3. The How — realising the What

The What says *what the product means*; the **How** says *how it's built* —
decisions, principles, contracts, and the repository layout — without changing
the meaning. Scaffold a starter contract:

```bash
product how init
product how add decision --label "Rust for the core" --rationale "..."   # optional
```

## 4. Delivery — slices and deliverables

A **slice** is a buildable section of your event model, pinned to an **anchor**
(a command, context, or flow). Carve one over the checkout behaviour:

```bash
product slice new checkout --anchor PlaceOrder
```

Wrap it as a **deliverable** — a slice plus its acceptance criteria:

```bash
product deliverable new checkout-v1 --slice checkout
```

## 5. See the whole picture

```bash
product status        # counts across What / How / Delivery
product guide         # your journey checklist + the next step
```

```
── Your framework journey ──
  [x] Captured a What model
  [x] What is conformant
  [x] How contract scaffolded
  [x] Delivery slice carved
  [ ] Deliverable wrapped
```

---

## Where to go next

- **Make behaviour executable.** `product decider derive Order` derives a
  Decider for the `Order` aggregate; `product decider simulate` proves it sound
  *before* any code. (Read-models get a symmetric `product projector`.)
- **Build it.** `product build checkout-v1` assembles a frozen build context and
  runs the realisation against the verification gates.
- **Explore a node's context.** `product domain context Order --depth 2` gives an
  LLM-ready bundle around any node.

Keep going:

- **[Everyday use](everyday-use.md)** — the commands you'll reach for day to day.
- **[Flows](flows.md)** — the full recipes for authoring the How, making behaviour
  executable, and delivery + build.
- **[Concepts](framework-concepts.md)** — any term you're unsure of.
- **[Workshop runbook](../workshop-runbook.md)** — running a session for a team.

Lost at any step? **`product guide`.**
