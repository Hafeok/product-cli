# Framework concepts — a plain-language primer

The framework models a product as three layers — **What**, **How**, and
**Delivery** — held in one graph. This page explains each term in a sentence or
two. For the normative definitions see
[product-framework-open.md](../product-framework-open.md); for a hands-on tour see
[getting-started-framework.md](getting-started-framework.md).

## The big idea

> Specify *what* the product means and *how* it's realised as **separate,
> linked models** — and never fuse them. The What is agreed before the How; the
> same What can drive several Hows. Everything is checked, not asserted.

---

## The What — what the product *is* and *does*

The What has two halves, both authored before any How.

| Term | In one sentence |
|---|---|
| **Bounded context** | A region of the product with its own consistent language (e.g. *Catalog*, *Billing*). The same word can mean different things in two contexts — you map that explicitly. |
| **Entity** | A thing with identity that lives in exactly one context (e.g. *Book*, *Order*). An **aggregate root** is an entity that owns a consistency boundary. |
| **Value object** | A thing defined only by its values, with no identity (e.g. *Money*, *ISBN*). |
| **Relation / invariant** | A typed link between entities (always with a rationale), or a rule that must always hold. |
| **Command** | Something a user asks the system to do (*PlaceOrder*). It targets an aggregate and must emit at least one event. |
| **Event** | Something that happened, in the past tense (*OrderPlaced*). Every event changes a real entity — behaviour can't reference structure that doesn't exist. |
| **Read model** | A view assembled for a screen or report (*OrderSummary*). It projects from the events that feed it. |
| **UI step** | The *What* of a screen: the projection it surfaces, the commands it offers, typed against abstract interactions — never concrete widgets (see [the v1.2 UI model](../product-framework-open.md#321-the-ui-step--the-what-of-a-screen)). |

The What is **type-checked**: `product domain validate` rejects a graph where an
event changes nothing real, or a command emits no event.

---

## Making behaviour executable

| Term | In one sentence |
|---|---|
| **Decider** | The executable form of an aggregate's behaviour — its signature is *derived* from the event model and it's *simulated* sound before any code exists. |
| **Projector** | The same idea for a read model: the executable fold that builds the view, derived and simulated. |
| **Named-algorithm primitive** | The honest escape hatch for a genuinely irreducible computation (SHA-256, a geometric solver) — specified by reference plus an oracle, not pretended to be derivable. |

---

## The How — realising the What

| Term | In one sentence |
|---|---|
| **Decision / principle / pattern** | The *Why*, declared once and referenced by pointer, each carrying its rationale. |
| **Contract** | A checkable statement about the realisation surface (e.g. decision logic kept in a pure core). |
| **Repository layout model** | Glob rules saying which files must, may, and must not exist — it both scaffolds and verifies. |
| **Reification** | `reify(AIO, context) → component`: how an abstract interaction becomes a concrete design-system control, per device — the UI half of the How. |

---

## Delivery — bringing it to a verifiable 'done'

| Term | In one sentence |
|---|---|
| **Slice** | A named, buildable section of the event model, pinned to an anchor (a command, context, or flow). |
| **Deliverable** | One slice plus its acceptance criteria; 'done' only when every criterion has a passing verdict. |
| **Release** | A group of deliverables, with its own computed 'done'. |
| **Work unit** | One bounded transformation with frozen input and a single output artifact. |
| **Verification** | A deterministic check whose oracle is *derived from the model* — one failure fails the build, with no override-by-assertion. |

---

## How the layers connect

```
        WHAT                         HOW                      DELIVERY
  bounded contexts            decisions / principles          slices
   entities, VOs        ───►   contracts, layout       ───►   deliverables
  commands → events           reification (UI)                 releases
   read models                                                 work units
        │                                                         │
        └──────────── one graph, every link typed & checked ──────┘
```

Run **`product guide`** any time to see where you are in this picture and what
to do next.
