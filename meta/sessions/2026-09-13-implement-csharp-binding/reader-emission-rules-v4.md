# The reader's emission rules — schema v4, restated in full (CG-R-69, CG-R-71 order step 3)

**2026-09-14.** What `tools/csharp-inventory` emits and what it does not, stated before any
prediction is committed against it. A denominator is not defined if the instrument silently
omits a class of its members (CG-R-69); every omission below is named.

## Types and members

1. **Types**: every named type declared in source in any project of the solution, nested types
   included, compiler-generated names (`<…>`) excluded, plus the type holding the entry point
   whether or not Roslyn gives it a source location. Source-generated types are included when the
   generator's output is in the workspace compilation: the Mediator generator's is; **the Razor
   source generator's is not** (R-1, cause not established) — Razor views, and every `@inject`
   in them, are absent (A: 34 directives; B: 342 across 222 views, by source grep).
2. **Members**: methods, constructors, properties, fields, events (`Ids.IsRecordedMember`), with
   accessibility, static-ness, entry-point flag, parameter names and types, return type.
3. **Attributes** with resolved arguments are emitted for types and members. **Parameter
   attributes are not emitted**: `[FromServices]` on a parameter (method injection) is invisible
   (A 0, B 7 by source grep).
4. **Projects** carry id, name, path, assembly name, target framework where the workspace names
   it, and (v4) **every referenced assembly by name** — the consumer reads test-framework
   membership from that list; the reader classifies nothing.

## References (the graph)

5. Each edge is `(from member, to symbol, kind)` with kind one of `call`, `construct`, `access`,
   `parameter`, `signature`, `generic-argument`, `type-test`, `resolve`, `type-reference`,
   `inherit`, `implement`, `attribute`, resolved through the semantic model. Edges are
   de-duplicated per `(from, to, kind)`.
6. **A reference carries the target's original definition**: type arguments are dropped
   (`IRepository<A>` and `IRepository<B>` from one constructor are one edge; the arguments
   appear only as separate `generic-argument` edges from the same member, unattributed to a
   parameter). Consequence: collection injection (`IEnumerable<T>`) cannot be read as an edge to
   `T` (P-6 proposes per-parameter type arguments).
7. **Edges whose target is declared outside the solution** are emitted when the target is an
   abstraction (an interface or an abstract class), or the edge is a `parameter` or a `resolve`;
   `inherit`, `implement` and `attribute` edges are emitted whatever the target. Every other
   external target is dropped.
8. **External types** (`external_types`) are described when they are the target of an emitted
   edge under rule 7 **or (v4) the base type or a base interface of an in-solution type or of
   another described external type**: kind, arity, abstractness, declared member counts, the
   abstract return types of their members, and (v4) their **base type and base interfaces**, each
   described in turn — so member counts can be summed over the interface chain (P-1).
9. `resolve` edges are the type arguments of a call whose receiver is (or implements)
   `IServiceProvider`, matched by the interface's symbol id.

## Registrations

10. **A registration fact is a call whose receiver is `IServiceCollection`** (or a type
    implementing it), as an extension or instance method, **or (v4, R-2) a call whose receiver is
    the result of a registration call earlier in the same statement** — `services.AddHealthChecks()
    .AddCheck<T>()`, `AddIdentity<,>().AddEntityFrameworkStores<>()`. Each carries the site, the
    receiver's type (v4), the method's symbol id and name, generic and `typeof` arguments, the
    types constructed in its arguments, whether a lambda is among them, whether it sits in a
    conditional, file and line. A builder held in a variable is not followed: stated limit.
11. Every such call is emitted, `AddControllers` as much as `AddScoped`. Which calls register what
    is decided Rust-side (`csharp_di`, and the registration-knowledge table for calls whose bodies
    are the framework's — CG-R-75).

## Diagnostics

12. Per project: the compile-error count with the first error; per workspace: every load
    failure. A consumer reads these before any figure (A's `BlazorAdmin` has 12 compile errors in
    every inventory so far).

## What the consumer does with them (for completeness, not emission)

- Test projects: a project whose referenced assemblies include a test framework
  (`TEST_FRAMEWORK_ASSEMBLIES`) is outside the primary convention (CG-R-75).
- Container-constructed: roots ∪ every implementation a reached, unconditional registration
  names (O-17).
- Composition edges: constructor parameters of container-constructed types, service-locator
  arguments, `[Inject]`/`[FromServices]` properties on container-constructed types.
- States: resolved · unresolved(reason) · partial · boundary · registration-not-read(call) ·
  excluded(role). Coverage = resolved / (resolved + unresolved), always printed with
  scored / composition edges (CG-R-73).
