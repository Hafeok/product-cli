# csharp-inventory

The C# stack binding's reader. Loads a solution through Roslyn's `MSBuildWorkspace` and writes
one JSON artefact — the **inventory** — that `product csharp …` consumes.

**It emits facts only.** Projects, types, members, every declared attribute with its arguments as
the compiler resolved them, the reference graph with *how* each target is used (`call`,
`construct`, `access`, `parameter`, `signature`, `generic-argument`, `type-test`, `resolve` — a
service-locator call's type argument — `type-reference`, `inherit`, `implement`, `attribute`),
every abstraction declared outside the solution that an edge lands on with its member shape
(`external_types`), and every call on an `IServiceCollection` as written — `AddControllers` as
much as `AddScoped<IFoo, Foo>()` — with its generic and `typeof` arguments, what its arguments
construct, whether a lambda is among them, and whether it sits inside a conditional (the
`registrations` array). Schema version 3 (2026-09-14). No field carries a judgement: there is no
slice, no role, no region, no verdict in the output. Every rule and classification lives in Rust,
against the store (`dec/ddd/batch-inventory-reader`, constraint 1). If this tool ever starts
deciding what counts as a slice, the exception that admits it has widened into a second decision
engine.

The artefact's shape is `schema/json/csharp-inventory/inventory.schema.json`, version `3`. The
Rust side refuses any `inventory_version` it does not know before reading another field. Which
calls register what, what each resolves to, and which edges are composition edges under the
denominator criterion, is decided Rust-side (`pf::csharp_di`, `pf::csharp_roles`,
`pf::csharp_walk`); the reader records the call and the use.

## Run

Targets .NET 10 on Roslyn 5.0, which reads `.slnx` solutions. Needs a .NET SDK able to load the
target solution (`MSBuildLocator` picks the installed one); restore the solution first, or the
compilations carry error types and the `diagnostics` array says so per project.

```bash
dotnet build -c Release tools/csharp-inventory
dotnet tools/csharp-inventory/bin/Release/net10.0/csharp-inventory.dll path/to/Solution.slnx --out inventory.json
product csharp inventory inventory.json          # version + schema check, counts
product csharp reach inventory.json --roots entry-point,public
product csharp delta inventory.json --event-model ordering.eventmodel.yaml
```

Workspace load failures and per-project compile-error counts land in the artefact's
`diagnostics` array rather than aborting the run, so a partial inventory says it is partial.

## What is not recorded

- Edges to targets outside the solution are kept when the target is an abstraction (an interface
  or abstract class — what a container satisfies), for `parameter` and `resolve` edges whatever
  the type (composition facts), and for `inherit`/`implement`/`attribute`; other external targets
  are dropped, or `System.*` would dominate. Every kept external target has an `external_types`
  entry with its shape.
- Implicitly declared members (record `Equals`, property accessors, compiler-generated closures).
- A `typeof(X)` attribute argument is rendered as the string `"T:X"` — the one lossy case.
- Target frameworks are read from the workspace's multi-target project name suffix only; a
  single-target project reports an empty list.
- Only `IServiceCollection` calls are recorded as registrations. Autofac, Ninject and other
  containers are not read (2026-09-13).
- Generic type arguments are recorded as their original definitions: `AddScoped<IHandler<X>, H>()`
  records `IHandler`1`, not the closed type. The consumer resolves by generic definition, which
  over-approximates when several closed instantiations of one interface are registered separately.

## The inventory is measurement

Re-derivable, never a determination, never committed — with one exception: the inventory of the
test fixture solution under `product-cli/tests/fixtures/csharp-inventory/` is committed as test
data, so workspace CI stays hermetic without a .NET SDK (`dec/ddd/fixtures-not-sdk`).
