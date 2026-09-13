# csharp-inventory

The C# stack binding's reader. Loads a solution through Roslyn's `MSBuildWorkspace` and writes
one JSON artefact — the **inventory** — that `product csharp …` consumes.

**It emits facts only.** Projects, types, members, every declared attribute with its arguments as
the compiler resolved them, and the reference graph (`call`, `construct`, `access`,
`type-reference`, `inherit`, `implement`, `attribute`). No field carries a judgement: there is no
slice, no role, no region, no verdict in the output. Every rule and classification lives in Rust,
against the store (`dec/ddd/batch-inventory-reader`, constraint 1). If this tool ever starts
deciding what counts as a slice, the exception that admits it has widened into a second decision
engine.

The artefact's shape is `schema/json/csharp-inventory/inventory.schema.json`, version `1`. The
Rust side refuses any `inventory_version` it does not know before reading another field.

## Run

Needs a .NET SDK able to load the target solution (8.0 or later; `MSBuildLocator` picks the
installed one).

```bash
dotnet build -c Release tools/csharp-inventory
dotnet tools/csharp-inventory/bin/Release/net8.0/csharp-inventory.dll path/to/Solution.sln --out inventory.json
product csharp inventory inventory.json          # version + schema check, counts
product csharp reach inventory.json --roots entry-point,public
product csharp delta inventory.json --event-model ordering.eventmodel.yaml
```

Workspace load failures land in the artefact's `diagnostics` array rather than aborting the run,
so a partial inventory says it is partial.

## What is not recorded

- Reference edges of kind `call`, `construct`, `access` and `type-reference` are emitted only when
  the target is declared in the solution; `System.*` targets would otherwise dominate.
  `inherit`, `implement` and `attribute` edges keep external targets, since a framework base type
  or attribute is exactly what a root convention may name.
- Implicitly declared members (record `Equals`, property accessors, compiler-generated closures).
- A `typeof(X)` attribute argument is rendered as the string `"T:X"` — the one lossy case.
- Target frameworks are read from the workspace's multi-target project name suffix only; a
  single-target project reports an empty list.

## The inventory is measurement

Re-derivable, never a determination, never committed — with one exception: the inventory of the
test fixture solution under `product-cli/tests/fixtures/csharp-inventory/` is committed as test
data, so workspace CI stays hermetic without a .NET SDK (`dec/ddd/fixtures-not-sdk`).
