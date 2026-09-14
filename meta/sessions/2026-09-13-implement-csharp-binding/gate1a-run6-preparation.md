# Run 6 preparation — what is built, what waits, held — 2026-09-14

Worked in the order CG-R-71 … 75 set. **Held before step 4 (predictions)**: the proxy
re-examination (`gate1a-proxy-reexamination.md`) yields three proposals (P-5 … P-7) that move
the denominator, and predictions made against proxies a ruling may change would be void on
arrival. Nothing has been walked.

## 1. Ground truth (CG-R-71) — done, `gate1a-ground-truth.md`

Reader recall 55/56, walk recall 45/56, walk precision 46/46 on A's `Web`, enumerated from
source and committed before the comparison. The measure is now a flag: `product csharp reach
--ground-truth <yaml>` prints reader recall and walk recall/precision beside every coverage
figure; the fixture carries its own enumeration (`ground-truth.yaml`, 20 edges: 20/20/20).

## 2. Repairs — done, tested on the fixture

| Item | Where | What |
|---|---|---|
| P-4 (O-16) | `csharp_reach.rs` | `implements:<T>` walks the inherit chain through in-solution bases |
| P-1 (O-13) | reader v4, `csharp_roles.rs` | external types carry base type + base interfaces, each described; marker/data-contract count members over the chain |
| P-2 (O-14, CG-R-75) | `csharp_walk.rs`, `csharp_di_knowledge.rs` | shape is not the test; a fifth state `registration-not-read(call)` from a declared table of framework registration calls, matched by resolved method id; `boundary` only when no reached call names the type; every report lists the reached external calls the table does not know |
| P-3 (O-15, CG-R-75) | reader v4, `csharp_inventory.rs` | projects carry referenced assemblies; a project referencing a test framework is outside the primary convention: no roots, no registrations read, no implementors for the boundary test, reported on its own row |
| O-17 | `csharp_walk.rs` | container-constructed = roots ∪ every implementation a reached, unconditional registration names |
| R-2 | reader v4 | calls chained off an `IServiceCollection` call are registration facts with their receiver (`AddHealthChecks().AddCheck<T>()`) |
| CG-R-73 | `csharp_resolution.rs`, render | every coverage figure is *x/y of the denominator, y/z of composition edges*; partial's disposition stated in words; B's table inconsistency was the *in denominator* column (it included partial) — the fraction was right, the column is now `scored` = resolved + unresolved |
| CG-R-69 | `reader-emission-rules-v4.md` | the emission rules restated in full, gaps named (R-1 Razor views, parameter attributes, type arguments dropped) |

Fixture: 49 types over four projects (one a test project), 19 registrations; the run on it
exercises every state (resolved, unresolved, partial, boundary, registration-not-read, excluded)
and every repair. Gates: `cargo t` green, clippy green, xtask green, `ddd validate`
conformant, seams bound, `diff-contracts` 0 undischarged.

## 3. Inventories — generated on v4, **unwalked**

| | v4 inventory |
|---|---|
| A | {"types":393,"regs":138,"ext":312,"diag":20} |
| B | {"types":6470,"regs":3064,"ext":1157,"diag":8} |

Both sit in the scratchpad beside the v3 files (instrument history). Nothing has run
`csharp reach` on them; the summaries above are `inventory` counts.

## 4. What waits on a ruling

- **P-5, P-6, P-7** (`gate1a-proxy-reexamination.md`): factory-provider read from provider ids
  only; collection injection as an edge to `T` (needs per-parameter type arguments in the
  reader); the data-contract role retired. P-5 alone moves a third of B's composition edges.
- **R-1**: why the Razor source generator's output is absent from the workspace compilation
  (342 `@inject` directives in B). Not attempted beyond establishing the fact.
- **Roots for A**: `implements:T:Microsoft.AspNetCore.Mvc.ViewComponent` (2 ground-truth
  edges); middleware named by `UseMiddleware<T>` has no fact to root on.
- Then: **both predictions**, committed against the rules as they stand after the ruling;
  **run 6**; **§12.1** on run 6.

## 5. Weakest point

The registration-knowledge table. It is the session's reading of what a dozen framework calls
register, written today from documentation memory, and it decides `registration-not-read` versus
`boundary`. Its only field test is the fixture's `AddMemoryCache`. Every report prints the
reached calls it does not know, which bounds the omission but not an error inside a row.
