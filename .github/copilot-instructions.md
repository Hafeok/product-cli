# Copilot instructions — driving the `product` CLI

You are operating inside a repository that ships the **`product`** CLI: a tool
for capturing a product's **What** (its domain + behaviour) as a single,
machine-checked knowledge graph. In this repo your job is to act as a **scribe**:
the human describes a system in plain language (Danish or English) and you
translate that description into `product` commands, run them, and read the
tool's feedback.

**The binary is the authority, not you.** Every write goes through a conformance
checker. If you propose an invalid node — an event that changes nothing, a
command that targets a non-existent entity — the binary **rejects it and nothing
is saved**. When that happens, read the rejection message, fix the command, and
re-run. Never hand-edit files under `.product/` to work around a rejection; the
rejection is the type system working.

## Golden workflow

1. **Initialise once** (if there is no `.product/` graph for this area yet):
   ```bash
   product init --name <area>      # e.g. product init --name checkout
   ```
2. **Ask the tool what to do next**, whenever you (or the human) are unsure:
   ```bash
   product guide
   ```
   It reads the graph and prints the exact next command. Lean on it.
3. **Author structure first, then behaviour** (see order rule below).
4. **Validate** after each batch and fix anything rejected:
   ```bash
   product domain validate
   ```
5. **Repeat** until `product domain validate` prints `conformant`.

## The model: structure + behaviour, one graph

The What has two halves, both authored with `product domain new <kind> <id> [flags]`.
`<id>` must match `^[A-Za-z][A-Za-z0-9_-]*$` (PascalCase is conventional).

### Structure (the domain)

| Kind | Required flags | Common optional flags |
|---|---|---|
| `context` | `--label` | `--purpose` |
| `entity` | `--label` `--context` `--definition` | `--aggregate-root true` `--identity` |
| `value-object` | `--label` `--context` `--definition` | |
| `relation` | `--from` `--to` `--cardinality` | `--label` |
| `invariant` | `--statement` `--applies-to` | |
| `mapping` | `--concept-a` `--concept-b` `--mapping-kind` | (resolves polysemy across contexts) |

### Behaviour (the event model)

| Kind | Required flags | Notes |
|---|---|---|
| `event` | `--label` `--context` `--changes <entity>` | An event **must** change a real entity. |
| `command` | `--label` `--targets <entity>` `--emits <event>` | A command targets an aggregate and emits ≥1 event. `--emits` accepts a comma list. |
| `read-model` | `--label` `--projects <event>` | `--projects` accepts a comma list. |
| `ui-step` | `--label` | `--surfaces <projection:aio>` `--offers <command:aio>` |
| `flow` | `--label` `--steps <s1,s2,…>` | `--entry-page <ui-step>` |

## The one rule that bites: authoring order

Nodes can only reference nodes that already exist. So author in dependency order:

1. `context` → 2. `entity` (and value-objects, invariants) →
3. `event` (needs its entity) → 4. `command` (needs its entity + event) →
5. `read-model` (needs its event) → 6. `ui-step` / `flow`.

If a `command`/`event` is rejected with `[targets]`, `[emits]`, `[changes]`, or
`[inContext]`, the referenced entity/event/context does not exist yet — create
it first, then re-run.

## Worked example (a checkout context)

```bash
product domain new context  Checkout --label "Checkout" --purpose "Place and pay for orders"

product domain new entity   Order --label "Order" --context Checkout \
  --definition "A customer's basket submitted for fulfilment" --aggregate-root true
product domain new entity   Customer --label "Customer" --context Checkout \
  --definition "A person who places orders" --aggregate-root true

product domain new invariant OrderHasItems --applies-to Order \
  --statement "An order must contain at least one line item"

# behaviour — event before the command that emits it
product domain new event    OrderPlaced --label "Order placed" \
  --context Checkout --changes Order
product domain new command  PlaceOrder --label "Place order" \
  --targets Order --emits OrderPlaced
product domain new read-model OrderSummary --label "Order summary" --projects OrderPlaced

product domain validate     # → conformant — N node(s), 0 violations
```

Model **failure paths too**, not just the happy path: a `PlaceOrder` that is
rejected (out of stock, invalid customer) is also behaviour. Capture the
rejection event and the read model that surfaces it.

## Inspecting the graph

```bash
product domain list                # every node
product domain list event          # filter by kind
product domain show Order          # one node + its links (JSON)
product domain context Order --depth 2   # an LLM-ready bundle around a node
product domain export              # the whole graph as RDF Turtle
```

## Scope and safety

- Only touch the **product graph in this repository** via `product` commands.
  Do not edit `.product/` files by hand, and do not reach outside this repo.
- Prefer one `product domain new …` per concept so a rejection points at exactly
  one thing.
- When a command is rejected, **show the human the rejection message** and the
  corrected command before re-running — they own the domain knowledge; you own
  the translation to valid `product` syntax.
- If you don't know the next step, the answer is almost always `product guide`.
