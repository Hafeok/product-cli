# csharp-inventory fixture

`Fixture/` is a three-project solution built to exercise the C# stack binding's consumer without
a .NET SDK in CI:

| Project | Holds |
|---|---|
| `Product.Binding` | the `[Slice(act, role, Profile = …)]` and `[RealisesFact(fact)]` attributes — strings, never an enum (CG-R-54) |
| `Shop.Domain` | the fact vocabulary of `ordering.eventmodel.yaml` realised, names deliberately differing from fact ids where the PRD allows (`CartState` realises `Cart`); a shared value object (`Money`) with no fact; an orphan (`Ghost`) naming a fact the vocabulary lacks |
| `Shop.Tests` | an MSTest project (referenced assembly `MSTest.TestFramework` is the fact the consumer classifies it by, CG-R-75): registers `IClock`/`IAudit` doubles at a test site and implements `IAudit` — none of which reaches the primary convention |
| `Shop.Api` | an entry point wiring `Microsoft.Extensions.DependencyInjection`, a declared handler (`PlaceOrderHandler`), a declarable type (`CartService`), a type the act boundary runs through (`CheckoutService`), a `[Slice]` naming an act the vocabulary lacks (`BogusHandler`), an unreached type (`Dead`), a local `[Endpoint]` attribute for the attribute-root convention, one registration per resolver case: explicit generic, open-generic `typeof` pair (`IValidator<>`), factory lambda (`IIdGenerator`), self-registration (`OrdersEndpoints`), conditional (`IAudit`), an interface registered by nothing (`IClock`); and one case per role of the denominator criterion at a constructor parameter of `OrdersEndpoints`: a factory (`IClockFactory`), a provider (`IServiceProvider`, also used as a service locator), an external abstraction with an in-solution registration (`IComparer<Money>` → `MoneyComparer`), a data contract (`IReadOnlyList<string>`), a marker (`IDomainEvent`, also type-tested in `CartService`), a boundary edge (`ILogger<T>` — external, implemented and registered by nothing in the solution, CG-R-68); and a component the framework satisfies by property injection (`OrdersPanel : ComponentBase`, `[Inject]` `ICartReader` resolving through the registration and `[Inject]` `ILogger<T>` on the boundary); an interface inheriting its only member (`IAuditStore : IReadStore<string>`, P-1); `IMemoryCache` registered by `AddMemoryCache()`, a call the resolver does not parse (registration-not-read, CG-R-75); `PingCheck` registered by `AddHealthChecks().AddCheck<PingCheck>()`, a chained builder call (R-2); `AuditSink`, registered and resolved by no edge (O-17), taking `IEnumerable<IIdGenerator>` (collection injection, P-6) and `Func<IAudit>` (a provider id — the partial row, P-5) |

`inventory.json` is the reader's output for it (schema v5). `ground-truth.yaml` is the hand
enumeration of `Shop.Api`'s composition edges (CG-R-71): `product csharp reach --ground-truth`
measures reader recall and walk recall/precision against it. `ordering.eventmodel.yaml` is the domain state
change binding's shipped example, byte-identical to the arrived input filed at
`meta/sessions/2026-09-13-implement-csharp-binding/inputs/…/examples/ordering.eventmodel.yaml`
(sha256 `06d32bfa…`).

## Regenerate

```bash
dotnet build -c Release tools/csharp-inventory
cd product-cli/tests/fixtures/csharp-inventory
dotnet build Fixture/Fixture.sln
dotnet ../../../../tools/csharp-inventory/bin/Release/net10.0/csharp-inventory.dll Fixture/Fixture.sln --out inventory.json
```

`produced_at` and `git_head` change on every regeneration; nothing reads them.
