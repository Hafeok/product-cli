# Product (Next) — The What Metamodel

**The artifact kinds, typed edges, and SHACL shapes that make up a conformant What.**

*Working draft 0.1. Implements §7 of [`product-next-architecture-spec.md`](./product-next-architecture-spec.md) and §3 of [`product-framework-spec.md`](./product-framework-spec.md). This is the schema everything downstream references — author it first, before any How.*

---

## 1. Purpose

The What is one RDF graph of typed nodes describing **what the system is** (the domain model, structure) and **what happens** (the event model, behaviour), plus the **Delivery** partition layered on top. This document defines:

- the **node kinds** and their fields ([§3](#3-structure--the-domain-model)–[§5](#5-delivery--the-partition)),
- the **typed edges** between them — the derivation contract for the What ([§6](#6-the-typed-edge-catalogue)),
- the **SHACL shapes** that validate it, split into framework **meta-shapes** (shipped) and adopter **instance shapes** ([§7](#7-shapes-meta-vs-instance)),
- the **authoring surface** and how it compiles to canonical RDF ([§2](#2-authoring-surface-and-encoding)).

Two vocabularies are in play throughout:

- **`pf:`** — `https://product.dev/framework#` — the framework metamodel (classes `pf:Entity`, `pf:Event`, …; properties `pf:changes`, `pf:targets`, …). Shipped by the tool.
- **a repo namespace** — e.g. `ex:` → `https://acme.example/what#` — the adopter's instances. Declared per repo ([architecture §3](./product-next-architecture-spec.md#3-identity-global-from-line-one)).

---

## 2. Authoring surface and encoding

**Authored as Markdown + YAML front-matter; canonical form is RDF.** Authors keep `product-cli`'s ergonomics — short names, one file per node, prose body for human context — and the tool compiles each file to RDF triples in the repo namespace. The RDF is the storage and resolution form; the front-matter is the typing form. This is the concrete realisation of architecture §3's "authors type short names; the tool expands them to URIs."

**Short-name → URI expansion.** A bare name (`Order`) resolves within the file's own bounded context. A `context:Name` reference (`billing:Invoice`) resolves to another context in the same What repo. Cross-context references **must** be backed by a `ContextMapping` ([§3.6](#36-contextmapping)) or validation fails — this is how "never assumed" (framework §3.1) is enforced mechanically.

**File layout** (bounded contexts as folders, per architecture §2):

```
what/
  <context>/
    context.md            # the BoundedContext node + its glossary
    entities/<Name>.md
    value-objects/<Name>.md
    aggregates/<Name>.md
    events/<Name>.md
    commands/<Name>.md
    read-models/<Name>.md
    flows/<Name>.md
    invariants/<Name>.ttl  # SHACL instance shapes (authored as Turtle)
  mappings/<A>-<B>.md      # cross-context mappings live above any one context
  delivery/
    features/<slug>.md
    releases/<slug>.md
  what.shapes.ttl          # generated: instance shapes compiled from invariants/
```

Every node gets a stable URI from its kind + context + name; renaming is an explicit, logged operation (the request-log machinery kept from the core).

---

## 3. Structure — the domain model

### 3.1 BoundedContext

A namespace within the What. Inside it, a term has exactly one meaning (the **ubiquitous language**, framework §3.1).

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Context name; becomes a URI segment |
| `purpose` | ✓ | One sentence: what this context is responsible for |
| `glossary` | ✓ | term → definition map; the ubiquitous language for this context |

### 3.2 Entity

A concept **with identity** — two entities with identical attributes are still distinct.

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Entity name (unique within its context) |
| `identity` | ✓ | The attribute(s) that constitute identity |
| `attributes` | ✓ | name → type map (types are value-objects or primitives) |
| `description` | ✓ | What this entity *means* in business terms |

### 3.3 ValueObject

Compared **by value**, immutable. No identity.

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Value-object name |
| `attributes` | ✓ | name → type map |

### 3.4 Aggregate

A consistency boundary: a root entity plus the members that change together.

| Field | Required | Meaning |
|---|---|---|
| `root` | ✓ | The root `Entity` (the only external reference point) |
| `members` | ✓ | Entities / value-objects inside the boundary |

### 3.5 Relation

A typed link between entities that carries **cardinality and rationale** (framework §3.1 — rationale is mandatory, not optional).

| Field | Required | Meaning |
|---|---|---|
| `from` / `to` | ✓ | The related entities |
| `cardinality` | ✓ | `1:1` \| `1:N` \| `N:M` |
| `rationale` | ✓ | *Why* this relation exists — the business reason |

### 3.6 ContextMapping

An **explicit** cross-context correspondence. Without it, a cross-context reference is a validation error.

| Field | Required | Meaning |
|---|---|---|
| `from` / `to` | ✓ | The two contexts |
| `type` | ✓ | DDD relationship: `shared-kernel` \| `customer-supplier` \| `conformist` \| `anti-corruption-layer` \| `published-language` |
| `correspondences` | ✓ | term ↔ term equivalences across the boundary (e.g. `sales:Customer ≡ billing:Payer`) |

---

## 4. Behaviour — the event model

### 4.1 Event

A past-tense fact. **Every event `changes` at least one entity** (framework §3.2, §4 rule — enforced by meta-shape).

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Past tense (`OrderPlaced`) |
| `changes` | ✓ | The `Entity`(s) this event mutates — **≥1** |
| `payload` | – | Value-objects / attributes carried |
| `description` | ✓ | What occurred, in business terms |

### 4.2 Command

An intent. **Targets exactly one aggregate** and **emits** the events it may produce.

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Imperative (`PlaceOrder`) |
| `targets` | ✓ | The `Aggregate` it acts on — **exactly 1** |
| `emits` | ✓ | The `Event`(s) it can produce |
| `payload` | – | Input value-objects / attributes |

### 4.3 ReadModel

A projection. **Projects** the entities it reads, optionally **built-from** events.

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Read-model name |
| `projects` | ✓ | The `Entity`(s) it exposes — **≥1** |
| `built-from` | – | The `Event`(s) it is derived from |

### 4.4 Flow

A timeline composing commands, events, read-models, and interface steps — the artifact a non-engineer signs.

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Flow name |
| `steps` | ✓ | Ordered list; each step references a `Command`, `Event`, `ReadModel`, or `InterfaceStep` |

### 4.5 InterfaceStep

A human-facing step inside a flow (a screen, a prompt). Keeps the flow readable as a timeline with interface steps (framework §3.2).

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Step name |
| `actor` | ✓ | Who performs it |
| `presents` / `captures` | – | Read-models shown / commands issued |

---

## 5. Delivery — the partition

Per framework §7.1, features and releases are **subgraphs of the What**, and they live in the What repo ([architecture §5](./product-next-architecture-spec.md#5-delivery-across-the-seam)).

### 5.1 Feature

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Feature name |
| `covers` | ✓ | The `Flow`(s) this slice delivers (typically one) |
| `concepts` | – | Entities/aggregates it touches (derivable from the flow; declarable for emphasis) |
| `depends-on` | – | Other features this one requires (the build-order DAG) |

### 5.2 Release

| Field | Required | Meaning |
|---|---|---|
| `name` | ✓ | Release name |
| `includes` | ✓ | Member `Feature`(s) |

The **closed-cut** requirement (no included element depends on an excluded one, framework §7.2) is a meta-shape, not a field.

---

## 6. The typed-edge catalogue

Every relationship above is a `pf:` property. These are the derivation-contract links (framework §9) for the What half; the How adds `derived_from` / `conforms_to` / `applies` / `realizes` / `enforces` across the repo boundary.

| Edge | Domain → Range | Source field |
|---|---|---|
| `pf:contains` | BoundedContext → Entity/ValueObject/Aggregate | (folder membership) |
| `pf:partOf` | Entity/ValueObject → Aggregate | Aggregate.members |
| `pf:root` | Aggregate → Entity | Aggregate.root |
| `pf:relatesTo` | Entity → Entity (reified via Relation) | Relation.from/to |
| `pf:maps` | ContextMapping → BoundedContext (×2) | ContextMapping.from/to |
| `pf:changes` | Event → Entity | Event.changes |
| `pf:targets` | Command → Aggregate | Command.targets |
| `pf:emits` | Command → Event | Command.emits |
| `pf:projects` | ReadModel → Entity | ReadModel.projects |
| `pf:builtFrom` | ReadModel → Event | ReadModel.built-from |
| `pf:hasStep` | Flow → Command/Event/ReadModel/InterfaceStep | Flow.steps |
| `pf:covers` | Feature → Flow | Feature.covers |
| `pf:includes` | Release → Feature | Release.includes |
| `pf:dependsOn` | Feature → Feature | Feature.depends-on |

---

## 7. Shapes — meta vs. instance

Two SHACL layers, both validated against the graph snapshot. A shape violation is a verification failure, not a warning ([architecture §10](./product-next-architecture-spec.md#10-decided)).

### 7.1 Meta-shapes (shipped by the tool)

Enforce the framework's What rules — the same for every adopter. These make Level 1 conformance mechanical:

- **MS-1 event-changes** — every `pf:Event` has ≥1 `pf:changes` to a `pf:Entity` (framework §3.2).
- **MS-2 command-targets** — every `pf:Command` has exactly one `pf:targets` to a `pf:Aggregate`.
- **MS-3 readmodel-projects** — every `pf:ReadModel` has ≥1 `pf:projects` to a `pf:Entity`.
- **MS-4 relation-complete** — every `Relation` carries both `cardinality` and non-empty `rationale`.
- **MS-5 no-implicit-cross-context** — any reference whose subject and object are in different contexts is backed by a `ContextMapping`; otherwise violation (framework §3.1 "never assumed").
- **MS-6 behaviour-references-structure** — `changes`/`targets`/`projects` ranges must resolve to existing structure nodes (framework §3.2 "behaviour may not reference structure that does not exist").
- **MS-7 closed-cut** — for every `Release`, no `pf:Feature` it `includes` `dependsOn` a feature it does not include (framework §7.2).
- **MS-8 context-language** — every `BoundedContext` declares a non-empty `glossary`.

### 7.2 Instance shapes (the adopter's)

The domain `Invariant`s — authored as Turtle in `what/<context>/invariants/`, compiled into `what.shapes.ttl`. These are the adopter's proprietary constraints (framework open/closed line, §1) — e.g. "an `Order` total equals the sum of its line items," "a `Subscription` cannot be `active` without a `paymentMethod`." The tool validates them; it ships none.

---

## 8. Worked example

A minimal `sales` context, end to end. Authored form (front-matter elided to essentials):

```yaml
# what/sales/entities/Order.md
kind: Entity
name: Order
identity: [orderId]
attributes: { orderId: OrderId, status: OrderStatus, lines: "[OrderLine]" }
description: A customer's request to purchase, from draft through fulfilment.
```
```yaml
# what/sales/aggregates/Order.md
kind: Aggregate
root: Order
members: [Order, OrderLine]
```
```yaml
# what/sales/commands/PlaceOrder.md
kind: Command
name: PlaceOrder
targets: Order            # → Aggregate
emits: [OrderPlaced]
```
```yaml
# what/sales/events/OrderPlaced.md
kind: Event
name: OrderPlaced
changes: [Order]          # → Entity   (satisfies MS-1)
description: The customer committed to the purchase.
```
```yaml
# what/sales/flows/Checkout.md
kind: Flow
name: Checkout
steps: [ReviewCart, PlaceOrder, OrderPlaced, OrderConfirmation]
```
```yaml
# what/delivery/features/checkout.md
kind: Feature
name: Checkout
covers: [sales:Checkout]
```

Compiles to (canonical RDF, abbreviated):

```turtle
@prefix pf: <https://product.dev/framework#> .
@prefix sales: <https://acme.example/what/sales#> .

sales:Order        a pf:Entity ; pf:inContext sales: ; pf:identity "orderId" .
sales:OrderAgg     a pf:Aggregate ; pf:root sales:Order ; pf:partOf sales:OrderLine .
sales:PlaceOrder   a pf:Command ; pf:targets sales:OrderAgg ; pf:emits sales:OrderPlaced .
sales:OrderPlaced  a pf:Event ; pf:changes sales:Order .
sales:Checkout     a pf:Flow ; pf:hasStep sales:PlaceOrder, sales:OrderPlaced .
ex:checkout        a pf:Feature ; pf:covers sales:Checkout .
```

MS-1…MS-8 all pass; a How repo can now pin this snapshot and realise `Checkout`.

---

## 9. Conformance rules for the What (normative summary)

1. Every node is a `pf:`-typed RDF resource with a stable URI in its repo namespace.
2. Structure and behaviour live in **one graph**; behaviour edges (`changes`/`targets`/`projects`) must resolve to existing structure (MS-6).
3. Every event `changes` ≥1 entity; every command `targets` exactly one aggregate; every read-model `projects` ≥1 entity (MS-1/2/3).
4. Every relation carries cardinality **and** rationale (MS-4).
5. Cross-context references exist only through a declared `ContextMapping` (MS-5).
6. Each bounded context declares its glossary (MS-8).
7. Features and releases are subgraphs; a release cut is closed (MS-7).
8. Domain invariants are SHACL instance shapes; a violation fails verification.
9. Depth is proportional to behavioural complexity — a trivial CRUD concept needs no flow; the framework requires the *interesting* behaviour be modelled, not every triviality (framework §3.2). *(Advisory: not a hard shape.)*

---

## 10. Open questions

1. **Reification of `Relation`** — model as a reified node (richer: carries rationale/cardinality as triples) vs. a plain property with annotations. Leaning reified, for queryability.
2. **Glossary as nodes vs. literals** — should each ubiquitous-language term be its own `pf:Term` node (linkable from entities) or a literal map on the context? Nodes enable "where is this term used?" queries.
3. **Interface-step depth** — how much UI detail belongs in the What before it bleeds into the How. Needs a boundary rule.
4. **Feature `concepts` derivation** — auto-derive touched concepts from the covered flow vs. require explicit declaration. Auto-derivation is less to maintain; explicit is more reviewable.
