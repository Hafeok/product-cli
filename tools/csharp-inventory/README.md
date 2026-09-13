# csharp-inventory

The C# stack binding's reader. Loads a solution through Roslyn's `MSBuildWorkspace` and writes
one JSON artefact — the **inventory** — that `product csharp …` consumes.

**It emits facts only.** Projects, types, members, every declared attribute with its arguments as
the compiler resolved them, the reference graph (`call`, `construct`, `access`, `type-reference`,
`inherit`, `implement`, `attribute`), and every call on an `IServiceCollection` as written —
`AddControllers` as much as `AddScoped<IFoo, Foo>()` — with its generic and `typeof` arguments,
what its arguments construct, whether a lambda is among them, and whether it sits inside a
conditional (the `registrations` array, schema version 2). No field carries a judgement: there is no
slice, no role, no region, no verdict in the output. Every rule and classification lives in Rust,
against the store (`dec/ddd/batch-inventory-reader`, constraint 1). If this tool ever starts
deciding what counts as a slice, the exception that admits it has widened into a second decision
engine.

The artefact's shape is `schema/json/csharp-inventory/inventory.schema.json`, version `2`. The
Rust side refuses any `inventory_version` it does not know before reading another field. Which
calls register what, and what each resolves to, is decided Rust-side (`pf::csharp_di`); the reader
records the call.

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

- Reference edges of kind `call`, `construct`, `access` and `type-reference` are emitted only when
  the target is declared in the solution; `System.*` targets would otherwise dominate.
  `inherit`, `implement` and `attribute` edges keep external targets, since a framework base type
  or attribute is exactly what a root convention may name.
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
