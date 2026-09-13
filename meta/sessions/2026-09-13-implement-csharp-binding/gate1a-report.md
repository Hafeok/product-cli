# Gate 1a — the inventory reader, the consumer, reachability: built and tested; **not yet measured**

**Register discipline.** In force: CG-R-51 … CG-R-56 (`rulings-gate0.md`). `[PROPOSED]`: the
build below and every design choice in it. `[OPEN]`: the two items that keep the measurement from
running — Emil's, listed in §5.

**What this gate is not.** Gate 1a's measurement has not run. No brownfield solution is attached,
and no §12 prediction is committed; under CG-rule-08 the measurement waits for both. What is
reported here is that the instrument exists, what it does, and what it found on its own fixture.
A working tool is weak evidence (carried risk 2), and a fixture is the weakest solution there is.

---

## 1. What was built

| Piece | Where | Lines |
|---|---|---|
| The reader (.NET) | `tools/csharp-inventory/` — `Program.cs`, `Collector.cs`, `References.cs`, `Ids.cs`, `Model.cs` | 5 files |
| The artefact schema | `schema/json/csharp-inventory/inventory.schema.json` (version `1`) + `example.md` | |
| The consumer | `product-core/src/pf/csharp_inventory.rs` (load, **version refusal before parsing**, schema validation through the `jsonschema` crate against the vendored schema unchanged, index) | 264 |
| The event-model loader | `product-core/src/pf/eventmodel.rs` — the binding's `eventmodel.yaml`, unchanged | 105 |
| Reachability (§12.1) | `product-core/src/pf/csharp_reach.rs` | 194 |
| The delta (Gate 1b, built now, runs only with a vocabulary) | `product-core/src/pf/csharp_delta.rs` | 305 |
| CLI | `product csharp {delta, inventory, reach}` — `product-cli/src/commands/csharp.rs` | 137 |
| Fixture | `product-cli/tests/fixtures/csharp-inventory/` — a three-project solution, its inventory, the binding's `ordering.eventmodel.yaml` | |
| Tests | 19 unit (`*_tests.rs` siblings) + 5 CLI (`tests/csharp_binding.rs`) | |

Every file is under the 400-line gate; every function under the 40-statement gate. Gates run:
`cargo t` (every binary, green after one fix below), `cargo clippy --workspace -D warnings -D
clippy::unwrap_used` (green), `cargo xtask check` (0 errors), `ddd validate` (279 entries, 0
warnings), `ledger verify` (conformant), `ddd diff-contracts main` (5 new `pub` surfaces,
discharged by signed bindings on two seams).

**The reader emits facts only** (CG-R-55, constraint 1). Its output has no field for a slice, a
role, a region or a verdict; the schema's root is `additionalProperties: false` and a test
(`schema_rejects_a_judgement_field`) proves a `region` field is refused. Every attribute on every
symbol is recorded with its arguments as the compiler resolved them — nothing is filtered by
attribute identity, and attributes are matched Rust-side by resolved type id, never by name
suffix. Symbol identity is Roslyn's documentation-comment id, exact over overloads and arity, so a
rename is a new id (PRD §2: deletion detectable, orphans findable).

**Roots are a stated convention.** `product csharp reach --roots entry-point,public,attribute:<T:…>,
implements:<T:…>` — the four conventions CG-R-51 names, chosen Rust-side and printed as the first
line of every report. `Compilation.GetEntryPoint` is recorded by the reader as a fact
(`is_entry_point`); that it is a *root* is the consumer's choice. DI traversal
(`--through-implementations`: interface → implementors, base → derived) is opt-in and printed
beside every ratio, so §12.1 is measured both ways (P-10).

**The delta is indexed by act** (R-A, R-D as read at Gate 0 §3): per act in the event model —
*declared* (a `[Slice]` names it), *declarable* (an undeclared type's referenced facts are covered by
the act's positions), *unstructured* (a type spans it), or *unrealised* (no symbol references its
facts). The CG-R-52 proxy is a `const` in the module and is printed, all three fields, on every
delta report. Undeclared symbols appear only as counts by namespace in the two ratios.

## 2. What the fixture run shows — and what it cannot

The fixture solution was written to contain each case once. On it:

```
$ product csharp reach inventory.json --roots entry-point
reached: 15 of 27 types (55.6%)      # Main constructs its own services; interfaces stop the walk
$ product csharp reach inventory.json --roots entry-point --through-implementations
reached: 17 of 27 types (63.0%)
$ product csharp reach inventory.json --roots public
reached: 25 of 27 types (92.6%)      # the public surface of a demo is nearly everything
$ product csharp delta inventory.json --event-model ordering.eventmodel.yaml
Declared: 1     command:PlaceOrder      T:Shop.Api.Orders.PlaceOrderHandler
Declarable: 4   read-model:Cart (CartService), and the three commands ICartReader's single fact covers
Unstructured: 1 command:ConfirmOrder    spanned by CheckoutService (constructs OrderPlaced + OrderConfirmed)
Unrealised: 0 … after the attribution rule in §3; read-model:OrderSummary before it
unresolved: BogusHandler declares act 'Nonexistent'; orphan: Ghost realises 'Ghost'
ratios: reachable-undeclared 4 of 13, isolated-undeclared 9 of 13
```

**Fixture, not evidence.** The cases were placed by the party that wrote the separator, and the
numbers say nothing about a real solution. `public` reaching 92.6% of a demo is not §12.1 firing —
it is a 27-type fixture with almost nothing private. The percentages above are printed so the
reader can see the instrument produces the shape the PRD asks for; they carry no finding.

## 3. Defects found during the build, reported

- **Two symbol-id shapes broke the schema on first contact.** Roslyn returns no documentation id
  for array types (`string[]`) or generic type parameters (`TCommand`); the reader's fallback
  emitted display strings and the vendored schema refused them. Caught by the unit test that
  validates the fixture against the schema. Fixed in the reader (arrays and pointers render from
  their element id; type parameters keep Roslyn's `!:` prefix, which names no type) and the
  schema pattern now admits `!:`. Recorded because the *schema* caught it — the Rust consumer's
  serde model would have parsed both silently.
- **A type chosen as a root did not stand for its members.** The first closure walked a root
  type's own edges (inherit/implement/attribute) and never entered its methods, so
  `implements:<IHandler>` reached nothing. Caught by `attribute_and_implements_roots_select_declared_symbols`.
- **Spanning-type attribution was over-broad.** The first rule attributed a spanning type to
  every act touching any of its facts, which made `read-model:OrderSummary` unstructured because
  `CheckoutService` touches `OrderPlaced`, which `OrderSummary` reads. Caught by
  `every_act_gets_exactly_one_region`. **The proxy's separator is unchanged** (no act's positions
  cover the type's facts → unstructured); what changed is which acts a spanning type is
  *attributed to*: those that write a fact it constructs (a `construct` edge), or, when it
  constructs none, those that read a fact it reaches. This is a refinement inside CG-R-52's
  operationalisation, stated in the module's doc comment; it is flagged here so Emil can rule
  whether it is within the ruling or a change to it. **`[OPEN]` O-8.**
- **The subcommand-sort gate failed once** on the new enum; reordered. Trivial, listed because
  `cargo t` was red for it.

## 4. What is deliberately absent, with dates

- **Change coupling as a cluster dimension** — not built (2026-09-13). It needs the solution's git
  history, which the inventory does not carry (P-11). `reach` clusters by namespace and by root
  convention.
- **Target frameworks** — the reader reports them only from a multi-target project's name suffix;
  a single-target project reports an empty list (stated in the tool README).
- **The five checks, profiles, the determination loader** — Gate 2.
- **`product csharp` MCP mirrors** — none owed at this gate; nothing writes to `.product/`.

## 5. What the measurement needs — both Emil's

| # | Item | Blocks |
|---|---|---|
| O-2 | The named brownfield solution, attached (the classifier refused a third-party-org attach; not worked around) | the 1a run |
| O-6 | The §12.1 prediction, committed before the run: expected reachability percentage under `entry-point`, `public`, and the framework-handler convention Emil names for that solution (`attribute:<T:…>` / `implements:<T:…>`), each with and without `--through-implementations` | the 1a run |
| O-8 | Whether §3's attribution refinement is within CG-R-52 | the 1b report's wording |

When O-2 lands, the run is: install the SDK the solution targets (8.0 is installed here), build
`tools/csharp-inventory`, run it on the solution, then `product csharp inventory` and `product
csharp reach` under each convention, both traversal modes. The inventory is not committed.

## 6. Weakest point

The reader's reference graph is **syntactic, per member body, one hop**: it resolves invocations
and object creations through the semantic model, but it does not see reflection, DI registrations
expressed as strings or lambdas the container invokes later, source generators, or anything
behind `dynamic`. On a solution that wires handlers by convention at startup, `entry-point`
reachability will under-report and `implements:` will be the only honest root — which is itself a
convention the reader of the report has to accept. The second-weakest point is §3's attribution
rule: it was arrived at by making a fixture case come out right, which is exactly how a proxy
acquires an exception, and it is flagged as O-8 rather than absorbed.

**Hold.** No measurement runs before O-2 and O-6 land.
