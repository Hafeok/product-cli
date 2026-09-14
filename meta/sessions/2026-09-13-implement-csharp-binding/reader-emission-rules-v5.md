# The instrument, restated in full — reader emission rules (schema v5), the proxy set, the consumer's rules (CG-R-80 step 3)

**2026-09-14.** The artefact both run-6 predictions are made against. Supersedes
`reader-emission-rules-v4.md` (kept as history). §4 lists the rules that were never written down
before this restatement — reported here, before the predictions, as CG-R-80 requires.

## 1. Reader emission rules (`tools/csharp-inventory`, `inventory_version: "5"`)

1. **Types**: every named type declared in source in any project of the solution, nested
   included, compiler-generated names excluded, plus the type holding the entry point.
   Source-generated types are included when the generator's output is in the workspace
   compilation: the Mediator generator's is; the Razor source generator's is not.
2. **Blind spot (CG-R-78), counted as a fact**: per project, the Razor files (`.cshtml`/`.razor`)
   the workspace lists as additional documents and the lines in them beginning `@inject`. No edge
   is ever made from a directive. A: 71 files, 45 directives; B: 1,610 files, 342 directives.
3. **Members**: methods, constructors, properties, fields, events, with accessibility,
   static-ness, entry-point flag, return type, and (v5) **the type arguments of every parameter
   type and of the return/property/field type**, as original definitions.
4. **Attributes** with resolved arguments for types and members. **Parameter attributes are not
   emitted** (`[FromServices]` on a parameter is invisible; B 7 by source grep).
5. **Projects**: id, name, path, assembly, target framework where the workspace names it,
   every referenced assembly by name (v4), the Razor counts of rule 2 (v5).
6. **References**: `(from member, to symbol, kind)`, kind ∈ {call, construct, access, parameter,
   signature, generic-argument, type-test, resolve, type-reference, inherit, implement,
   attribute}, resolved through the semantic model, de-duplicated per `(from, to, kind)`.
7. **A reference carries the target's original definition**; the closed type's arguments are
   emitted as `generic-argument` edges from the same member and, since v5, on the parameter and
   return-type facts of rule 3 (so collection injection is readable per parameter).
8. **External targets** are kept when the target is an abstraction, or the edge is a `parameter`
   or a `resolve`; `inherit`, `implement` and `attribute` edges are kept whatever the target;
   every other external target is dropped.
9. **External types** are described when they are the target of a kept edge, or the base type or
   a base interface of an in-solution type or of another described external type: kind, arity,
   abstractness, declared member counts, abstract return types, base type, base interfaces.
10. **`resolve` edges** are the type arguments of a call whose receiver is or implements
    `IServiceProvider`, matched by symbol id. `GetServices<T>` and `GetService<T>` are not
    distinguished: both arrive as a single resolution of `T` (stated gap).
11. **Registration facts**: a call whose receiver is `IServiceCollection` or a type implementing
    it, or a call whose receiver is the result of a registration call earlier in the same
    statement (chained builders, R-2); each with site, receiver type, method symbol id and name,
    generic and `typeof` arguments, constructed argument types, lambda presence, conditional
    position, file and line. A builder held in a variable is not followed.
12. **Diagnostics**: per project the compile-error count with the first error; per workspace
    every load failure.

## 2. The proxy set in force (each with its divergence and measured incidence, CG-R-77)

Printed on every report by `csharp_roles::PROXIES`; retired readings by `RETIRED_PROXIES`.

| Role | Proxy | Divergence | Incidence |
|---|---|---|---|
| marker | no method, property or event anywhere in the interface chain | a DI-key or type-test marker is excluded although registered | 0 of 293 (A+B run 5); the pre-v4 reading misclassified 293 of 293 (repaired, P-1) |
| factory-provider | the declared provider ids only: `Func<>`, `Lazy<>`, `IServiceProvider`, `IServiceScopeFactory`, `IServiceProviderFactory<>` (P-5) | a solution's own factory interface reads as a service, resolved to what composition chose | unmeasured against a ground truth; on B run 5 provider ids were 39 of 346 |
| generic-dispatch | arity > 0, not a provider | a generic service reads as dispatch; same resolution path | 48 of 48 (A+B run 5) were generic services — no denominator effect |
| abstract-data | an abstract class no registration names | one registered through an unread wrapper is excluded | 0 of 9 (A+B run 5); 8 read as boundary/registration-not-read before the role |
| value | struct, enum, delegate, string, object | a container-supplied primitive is excluded | 0 of 3 (B run 5) |
| *retired:* data-contract | properties only | property-shaped services excluded | **181 of 181** (B run 5) — the divergence was the field (P-7) |
| *retired:* factory by return type | a member returning an abstraction | services returning an abstraction read as factories | **231 of 346** services + 76 collection injection (B run 5) (P-5, P-6) |

Other proxies in the programme, per CG-R-77's retroactive clause: the CG-R-52 separator and the
CG-R-57 attribution rule (`csharp_delta`) now carry `incidence: unmeasured` with the reason; the
binding's DP-5 fields are Emil's repository.

## 3. The consumer's rules (`csharp_walk`, `csharp_di`, `csharp_reach`)

- **Test projects** (CG-R-75): a project whose referenced assemblies include one of
  `xunit.core`, `xunit.v3.core`, `nunit.framework`, `MSTest.TestFramework`,
  `Microsoft.VisualStudio.TestPlatform.TestFramework`, `TUnit.Core` is outside the primary
  convention: no roots, its registration sites unread, its implementors not counted for the
  boundary test, its types on their own row.
- **Roots** are the stated convention; `implements:<T>` walks the inherit/implement chain
  through in-solution bases (P-4).
- **Container-constructed** = roots ∪ every implementation an unconditional registration at a
  reached site names (O-17). Registrations the resolver parses: the lifetime methods,
  `Add`/`TryAdd`/`Replace` with a `ServiceDescriptor`, `AddHostedService`, `AddDbContext`,
  `AddCheck` (chained), keyed, `Decorate`, and the scanning calls.
- **Composition edges**: each constructor parameter of a container-constructed type; each
  `resolve`; each `[Inject]`/`[FromServices]` property (by symbol id) of a container-constructed
  type. A parameter of `IEnumerable<T>`, `IReadOnlyList<T>`, `IReadOnlyCollection<T>`, `IList<T>`,
  `ICollection<T>` whose collection type is **not itself registered** is **collection injection**:
  the edge is to `T`, resolved to every reached registration of `T` (P-6, its own classification).
  Edges are counted once per `(from type, target, injection)`.
- **State, in this order** (see §4): a provider id → *partial*; a value → *excluded*; an external
  type no production type implements and no parsed registration names → *registration-not-read
  (call)* when a reached call in the knowledge table registers it, else *boundary*; a role
  outside the denominator → *excluded*; otherwise resolution through the reached registrations →
  *resolved* or *unresolved (reason)*.
- **Coverage** = resolved / (resolved + unresolved), printed as *x/y of the denominator, y/z of
  composition edges* (CG-R-73), with partial's disposition in words.
- **Table coverage** (CG-R-79): of the reached external registration calls, how many the resolver
  parses, the table knows, neither — printed per run, with the unknown calls ranked.
- **Ground truth** (CG-R-71): `--ground-truth` prints reader recall *over C# source, Razor views
  not covered* and walk recall/precision.

## 4. Rules surfaced by writing this down — findings, before the predictions (CG-R-80)

None of these was stated before this document; each was decided in code.

- **F-1 The order of state assignment.** Provider → value → library-provided → role → resolution.
  Before this commit `IServiceProvider` and `Func<>` were *boundary* (library-provided read
  first) and `string` parameters of container-constructed types were *boundary* too. Fixed on
  the fixture; the order is now a rule.
- **F-2 A registered collection type is a single dependency.** `IReadOnlyList<string>` registered
  as an instance is resolved as itself, not read as collection injection of `string`.
- **F-3 Edges are per type, not per site.** Two constructors of one type naming the same target,
  or a service-locator call repeated in two methods, are one edge. The ground truth is at that
  granularity too. Never stated; it lowers every population count relative to call sites.
- **F-4 `GetServices<T>` is read as a single resolution** (rule 10) — a reader gap for
  service-locator collection injection; unmeasured.
- **F-5 The delta's walk also excludes test projects** (its ratios are over production types)
  and starts from declared roots only; it never stated either.
- **F-6 Conditional registrations do not make their implementation container-constructed**
  (O-17 reads unconditional sites only); the type is reached only if something resolves it.

## 5. Denominator membership, stated explicitly (CG-R-83, added before run 6)

- **In**: resolved, unresolved.
- **Out**: partial, boundary, excluded, and — **for run 6 only** — `registration-not-read`,
  because that is the rule the three predictions were made against and a denominator is not
  moved after predictions are committed.
- **From run 7**: `registration-not-read` is **inside** the denominator and counts against
  coverage, reported as its own subset (CG-R-83: it is the instrument's ignorance of a registration
  that exists; excluding it would let coverage rise as the table learns less).
- Run 6 prints both figures, labelled: *coverage (rule in force, not-read outside)* and *coverage
  with not-read inside (the run-7 rule)*.
- **Roots for A** gain `implements:T:Microsoft.AspNetCore.Mvc.ViewComponent` (CG-R-84).

## 6. Amendments for run 7 (CG-R-85 … CG-R-88, schema v6) — the artefact run 7's predictions are made against

- **Reader v6 (O-18, ratified CG-R-87).** A registration fact also carries the type arguments of
  `ServiceDescriptor.<Lifetime><I, C>()` calls nested in its argument list (matched by the
  descriptor type's symbol id). The resolver reads `TryAddEnumerable(ServiceDescriptor.
  Scoped<I, C>())`, `Replace(…)` and kin as the pair they state. B reached 57 such calls in run 6;
  their services were *boundary* or *no-registration* there.
- **Host-builder provider (CG-R-87).** Once a production entry point is reached, `ILogger<T>`,
  `ILoggerFactory`, `IConfiguration`, `IHostEnvironment`, `IWebHostEnvironment`,
  `IHostApplicationLifetime`, the `IOptions*` family and `IServiceProviderIsService` are
  *registration-not-read (host builder)* whether or not a call appears in source. `ILogger<T>`
  now classifies the same in A and B.
- **Denominator (CG-R-83, in force from run 7).** `registration-not-read` is **inside**:
  coverage = resolved / (resolved + unresolved + registration-not-read). The run-6 form
  (not-read outside) is printed beside it, labelled, for continuity. Boundary, partial and
  excluded stay outside.
- **Headline (CG-R-86).** The **scored fraction** — scored / composition edges, where scored =
  resolved + unresolved (the run-6 definition of scored is kept so the two runs' fractions are
  comparable; under CG-R-83 the *denominator* of coverage is wider than *scored*) — is the first
  figure printed; coverage follows. §12.1 is read against the scored fraction.
  **Proposed form for that reading, not ruled:** the same three bands as before — fires below
  75%, provisional 75–<85%, clears at 85% — applied to the scored fraction. Held for ruling; run 7
  prints the fraction either way.
- **The run (CG-R-88).** `run7.sh` builds the binary it measures with, exits on any failing step
  (`set -euo pipefail`), and writes `run7-instrument.txt` (commit, working-tree state, binary
  and inventory hashes) before the first measurement.

Restated table coverage, fixture: of 17 reached external calls the resolver parses 14, the table
knows 3, 0 unknown. Ground truth on the fixture: 24 edges, 24/24/24.

## 7. Scored fraction, defined (CG-R-90); the error bound (CG-R-89) — added before run 7

Two readings, both reported on every run from run 7:

- **Scored fraction, first reading** = (resolved + unresolved) / composition edges — *what
  received a resolution verdict the resolver could act on*. **The session's run-7 prediction was
  made against this reading**, and CG-R-91's likewise; both are scored against it.
- **Classified fraction, second reading** = (resolved + unresolved + registration-not-read) /
  composition edges — *what received any verdict at all*. Reported, scored against nothing.
- **Error bound on the reachable/isolated split (CG-R-89)** = the unscored fraction under the
  second reading = (partial + boundary + excluded) / composition edges. A registration-not-read
  edge is classified and cannot silently reclassify a type; the unscored ones can. **§12.1's
  fire/clear form is retired**: no disposition is taken until the split exists (the delta needs
  Gate 1b's act vocabulary), and the bound is reported with the split when there is one. The
  proposal of §6 (three bands on the scored fraction) is withdrawn as refused.
