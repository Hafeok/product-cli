# The denominator criterion, as built — `[PROPOSED]`, held for ratification and for the predictions

**Register discipline.** In force: CG-R-63 … CG-R-67; Emil's criterion and role table (the run
message of 2026-09-14, filed as `invocation-gate1a-rulings.md`). `[PROPOSED]`: everything below.
`[OPEN]`: §6. **No figure for either solution appears here.** Under CG-R-63 and the message's
§4, the recomputation waits for both predictions, and both predictions wait for this criterion to
be ratified. The inventories for A and B were regenerated with the version-3 reader (facts) and
have not been walked.

---

## 1. What "an edge" is, operationally

The reader (inventory schema **version 3**) records *how* every reference is used, as a fact:
`call`, `construct`, `access`, `parameter` (a parameter's declared type; the member's own kind
says whether it is a constructor), `signature` (return, field, property type), `generic-argument`,
`type-test` (`is`/`as`/pattern/cast), `resolve` (the type argument of a `GetService`/
`GetRequiredService`/`GetServices` call on an `IServiceProvider`), `type-reference` (anything
else). It also records every abstraction declared outside the solution that an edge lands on
(`external_types`: kind, arity, method/property/event counts, and which of its members return
abstractions) — the O-11 fix — and keeps external targets for `parameter` and `resolve` edges
whatever the type.

**A composition edge** is:

- a `resolve` edge — the service locator asks the container; or
- a `parameter` edge of a **constructor** of a type **the container constructs** — a root the
  framework instantiates (`Main`'s type, a controller, a page model, an endpoint), or an
  implementation a resolved registration supplied.

Nothing else is. A method parameter is supplied by its caller; a field is assigned from a
constructor parameter that already counted; a type test chooses nothing; a return type is a
promise, not a dependency; a generic argument belongs to the edge it appears in. Constructor
parameters of a type *code* instantiates (`new Foo(x)`) are the caller's choice and do not count.
This is the criterion read literally: *satisfied by a chosen implementation at composition time*.

Counted once per (referencing type, target).

## 2. The roles, read off the target's shape through declared proxies

At every composition edge the target's role is read from its shape (in-solution: its members;
external: the recorded counts). Each rule is a proxy, with its divergence, printed on every
report (`csharp_roles::PROXIES`):

| Role | In denominator | Proxy | Known divergence |
|---|---|---|---|
| service | yes | an interface with a method (or event) that is none of the below; a registered abstract class; a concrete registered type | — |
| generic-dispatch | yes (resolved by registration; by declaration once `[Slice]` exists, CG-R-61) | an interface with generic arity > 0 that is not a factory | a generic service abstraction that is not dispatch (`IRepository<T>`) reads as dispatch |
| factory-provider | **partial** | `Func<>`, `Lazy<>`, `IServiceProvider`, `IServiceScopeFactory`, or an interface with a method or property returning an abstraction | a service that merely returns another service's result (a repository returning `IReadOnlyList<T>`) reads as a factory |
| marker | no | an interface declaring no methods, properties or events | an empty interface used as a DI key or type-test target is excluded although the container may register it |
| data-contract | no | an interface declaring properties only | a property-shaped service (`IOptions<T>.Value`) is excluded although composition chooses it |
| abstract-data | no | an abstract class no registration names as a service (CG-R-64) | one registered only through a wrapper the resolver does not read is excluded although it is a service |
| value | no | a struct, enum, delegate, `string` or `object` | a primitive the container does supply (an options-bound connection string) is excluded |

Order of application: the factory id list; then value kinds; then, for interfaces, marker →
data-contract → factory-provider (abstract returns) → generic-dispatch (arity) → service; for
classes, abstract-data unless registered. The `capability, tested for` row of Emil's table needs
no proxy: a `type-test` edge is never a composition edge, so it never reaches the role table.

## 3. The states, and the figure

At a composition edge: **excluded** (marker, data-contract, abstract-data, value — recorded by
role, never in the ratio), **partial** (factory-provider — the resolver still follows a
registration if one names the factory, and the state stays partial either way), **resolved**,
**unresolved** (with CG-R-62's reason). Coverage is `resolved / (resolved + unresolved)`; partial
is printed beside it and never divided. Types are reached, unresolved (implementing the target of
an unresolved edge), partial (likewise for a partial edge), or unreached; every figure prints with
all four.

One consequence to see: under the old walk a `call` on an interface member was itself resolved;
under the criterion it is not — a call reaches the implementations only because the composition
edge that supplied the interface already did. That is the criterion, not a regression.

## 4. The fixture under the criterion (a worked example, not evidence)

`product csharp reach inventory.json --roots entry-point` on the fixture:

```
reached: 28 of 39 types (71.8%) — unresolved: 1 — partial: 0 — unreached: 10
resolution coverage: 7 resolved of 8 scored composition edges (87.5%) — 1 unresolved — 2 partial
  composition edges: 12 — in denominator: 10 — excluded by role: 2
  edges by role: data-contract 1, factory-provider 2, generic-dispatch 3, marker 1, service 5
  unresolved by reason: conditional-registration 1
```

Every row was placed to be there: `IReadOnlyList<string>` and `IDomainEvent` as constructor
parameters (excluded, data-contract and marker); `IClockFactory` and `IServiceProvider` (partial);
`IHandler<>`, `IValidator<>` (generic dispatch, resolved by registration); `IComparer<Money>` — an
**external** abstraction with an in-solution registration, now followed to `MoneyComparer` (O-11's
mechanism, exercised); `IAudit` (conditional, unresolved). A type test on the marker and the data
contract as a method parameter never become composition edges.

## 5. What O-11's fix changes on the real solutions, stated before the numbers

The 15 (A) and 256 (B) registrations whose service is external will now be reachable through
`parameter`/`resolve` edges to those abstractions. Whether they *are* reached depends on the same
composition-edge rule as everything else. The handler question on A is answered by whichever of
these holds: the consumers of `IMediator`/`ISender` are container-constructed types with those as
constructor parameters (then resolved → `Mediator.Mediator`, whose generated dispatch reaches the
wrappers and handlers only if *its* dependencies are composition edges the walk can read), or they
are not. No prediction is made here; the message says none is committed until this is ratified.

## 6. Open, for ruling

| # | Question |
|---|---|
| C-1 | **Ratify the criterion's operational form** (§1–§3) — in particular that constructor parameters count only on container-constructed types, and that partial is never divided. |
| C-2 | The seven proxies as stated, or amended. Each divergence is a known misclassification the numbers will carry. |
| C-3 | Whether `resolve` edges whose target is a concrete registered type (a self-registration asked for by name) are `service` (as built) or their own row. |
| O-12 | Restated in `gate1a-o11-and-run4.md` §5: the resolver's next idioms — Orchard's ~700 generic registration-wrapper calls (under this criterion, the *factory/provider* partial row) and interface-typed data (now excluded by the data-contract proxy where property-shaped). Unruled. |

Then both predictions, against this criterion, before `product csharp reach` runs on A or B.

**Hold.**
