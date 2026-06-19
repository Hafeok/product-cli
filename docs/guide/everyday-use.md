# Everyday use — the daily driver

Once your product is modelled, these are the commands you reach for day to day.
New here? Start with the [tutorial](getting-started-framework.md); for the
term-by-term meaning see [concepts](framework-concepts.md); for the end-to-end
recipes see [flows](flows.md).

> The one habit: when you don't know what to do next, run **`product guide`**.
> It reads your graph and prints the exact next command.

## Orient

```bash
product guide      # where you are in the journey + the next command
product status     # counts across What / How / Delivery (+ the meta graph)
```

## Read the graph

```bash
product domain list                     # every What node
product domain list entity              # filter by kind (entity, command, event, aio, …)
product domain show Order                # one node and its links
product domain context Order --depth 2  # an LLM-ready bundle around a node
product domain export                   # the whole What graph as Turtle
```

## Change the graph

Author in dependency order — the thing a node points at must already exist
(an event's entity, a command's event). Each write is validated in-loop; a node
that breaks a rule is **rejected and not saved**, with the rule it broke.

```bash
# add — `<kind> <id>` plus the fields that kind needs
product domain new entity Cart --label "Cart" --context Catalog \
  --definition "A shopping cart" --aggregate-root true

# edit fields of an existing node
product domain edit Cart --definition "A member's shopping cart"

# remove (warns if it leaves dangling references)
product domain rm Cart
```

Required fields by kind (the common ones):

| Kind | Needs |
|---|---|
| `context` | `--label`, (`--purpose`) |
| `entity` | `--label`, `--context`, `--definition`, (`--aggregate-root`) |
| `event` | `--label`, `--context`, `--changes <entity>` |
| `command` | `--label`, `--targets <entity>`, `--emits <event>` (repeatable) |
| `read-model` | `--label`, `--projects <event>` (repeatable) |

## Check health

```bash
product domain validate     # What graph conformant? (exit 1 on violations)
product how validate        # How contract obeys the What
product conformance check   # the Two Pillars clause set (meta graph)
```

A clean `product domain validate` is the gate between What and How. If it
reports violations, `product guide` routes you to fix them.

## Hand context to an agent

```bash
product domain context PlaceOrder --depth 2   # bundle around a command/flow
product slice context checkout                # the concrete build-context for a slice
```

## Command map (framework graph)

| You want to… | Command |
|---|---|
| See where you are / what's next | `product guide` |
| Project overview | `product status` |
| Capture the What (facilitated) | `product author domain <product>` |
| Capture the What (by hand) | `product domain new …` |
| Validate the What | `product domain validate` |
| Author the How | `product how init`, `product how add …` |
| Make behaviour executable | `product decider derive <aggregate>`, `product projector derive <read-model>` |
| Carve a slice | `product slice new <id> --anchor <node>` |
| Wrap a deliverable | `product deliverable new <id> --slice <slice>` |
| Is a deliverable done? | `product deliverable done <id>` |
| Build it | `product build <deliverable>` |

Reach for [flows](flows.md) when you want the full recipe for any one of these.
