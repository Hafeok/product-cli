# [PROPOSED] PRD — C# stack binding for product-cli

**Status:** `[PROPOSED]`. Not ratified. Nothing here carries a falsifier; the one candidate is at §9.

**Purpose.** Connect a declared act vocabulary to a C# codebase, so that a determination addressed to an act can be checked against the code realising it, and so that what is *not* specified is measurable rather than assumed.

**Scope.** One stack: C#/Roslyn. The act vocabulary, the fact vocabulary, and the determination schema are stack-neutral and come from the domain state change binding unchanged. Everything in this document is the stack-specific layer beneath them.

**Not in scope.** Generating code from specifications. Deriving act vocabularies from code. Any second determination schema.

---

## 1. The rules this rests on

Four, stated first because every design decision below follows from one of them.

**R-A — Correspondence runs act → symbol, never symbol → act.** Importing the codebase produces *evidence about what exists*. It does not produce slices. A symbol with no declared slice is unspecified code, not a discovered slice.

**R-B — An act boundary is never adjusted to fit an existing class.** Where the act boundary and the class boundary disagree, either the code moves or the slice is declared not-attachable with the reason recorded. Widening an act to fit the code makes the specification conform to the implementation, which is the reading-backwards prohibition arriving with everyone's consent. This is the load-bearing rule of the document.

**R-C — Stack shape is a determination in the store, not logic in the tool.** That a command slice is realised by a controller, a handler and a provider is a *profile*: how one architecture realises an act type. It is authored, addressed, and superseded like any other determination. A profile compiled into the tool is a rule with no address, enforced by code nobody can query — PR-1 violated in the instrument built to prevent it.

**R-D — The delta is a list of gaps, not a queue of slices.** Undeclared symbols are never turned into candidate slices for bulk approval. Bulk approval is not ratification, and a confirmation step does not convert derivation-from-code into authorship.

---

## 2. Slice declaration

**Anchored in the code, by attribute.** Not in a side file keyed by fully-qualified name: a rename orphans that mapping silently, and silent orphaning is the failure that kills the address space.

```csharp
[Slice("PlaceOrder", Role.Handler)]
public class PlaceOrderHandler { … }

[RealisesFact("Cart")]
public record CartState { … }
```

**Requirements.**

- The analyser reads attributes only. It never pattern-matches class names, namespaces, or folder layout. Naming conventions are conventions; an attribute is a declaration.
- `Slice` names an act instance in the act vocabulary and a role in the profile. Both are checked against declarations, not inferred.
- `RealisesFact` names a fact in the fact vocabulary. **A type may realise a fact without sharing its name** — the declaration connects them, not the spelling.
- Rename is safe, deletion is detectable, moving a type between layers is visible.

---

## 3. Profiles

A **profile** declares how one architecture realises an act type in this stack.

```yaml
profile: rest-api-v1
act_type: command
roles:
  - name: controller
    required: true
    must:  [ "receives transport concern only", "calls exactly one handler role" ]
    must_not: [ "contains a decision", "reaches a provider role directly" ]
  - name: handler
    required: true
    must: [ "…" ]
  - name: provider
    required: false
```

**Filed as a determination**, addressed to the act type, pinned, with extent travelling to all command slices in the context. Changing the architecture is a supersession, not a release.

**Reuse, do not reinvent.** The handler role is a Decider by another name. Its rules — signature derived from the model, exhaustive command coverage, output containment, rejection containment, event coverage on state evolution, and the coverage declaration — are already specified. A second set of handler rules written here would drift from them within a release.

**Multiple profiles coexist.** REST API, worker, function app, Databricks. A slice declares which profile it is realised under.

---

## 4. Import, and the language boundary

**product-cli is Rust; Roslyn is .NET.** The import therefore runs in a separate .NET tool and the two communicate by artefact, not by FFI.

```
  csharp-inventory (dotnet tool)          product-cli (rust)
  ─────────────────────────────          ──────────────────
  Roslyn workspace load        ──▶  inventory.json  ──▶  checks, delta, reporting
  attributes, call graph                                  determination store
```

**The .NET side does no judgement.** It emits facts: types, members, project and namespace, declared attributes with their arguments, and the call graph between declared entry points and what is reachable from them. Every rule, every check, every delta classification is on the Rust side against the determination store.

That boundary matters beyond convenience. If the .NET tool starts deciding what counts as a slice, the stack binding acquires logic that no determination addresses — R-C, one level down.

**The inventory is re-derivable and is never stored as a determination.** It is measurement, refreshed on demand. It is not committed.

**Inventory schema is versioned.** The Rust side refuses an inventory whose version it does not know rather than parsing it optimistically.

---

## 5. The delta — three regions, not two

The output that makes this sellable on day one, and the shape that stops it being misread.

| Region | Meaning | Cost |
|---|---|---|
| **Declared** | a slice is declared and realised | none |
| **Declarable** | no slice declared, but a symbol exists that corresponds to an act — someone need only say so | filing |
| **Unstructured** | no symbol corresponds to an act; the act boundary runs through the middle of a type and nothing can attach until code moves | engineering |

**The third region is the number nobody currently quantifies before quoting**, and it is the one the client most needs. It must state *why* each region is unstructured — the act boundary runs here, this type spans it — so the change is justified by a named act rather than by architectural taste. Without that, an assessment reads as a rewrite proposal, and the reader will not be entirely wrong.

**Report by cluster, not by symbol.** Four hundred undeclared types is unactionable. Grouped by namespace, by reachability from a declared entry point, or by change coupling, it is a small number of unspecified regions.

**Two ratios, and only one is urgent.**

- **Reachable-undeclared** — undeclared symbols reachable from a *declared* slice. This is a specification that looks complete and is not: the symbols it reaches carry determinations nobody wrote down. Same defect as a coverage claim with no uncovered set, one layer down.
- **Isolated-undeclared** — undeclared and unreachable from any declared slice. Honest untouched work.

---

## 6. Checks

Bidirectional, and both directions matter.

| Check | Fails when |
|---|---|
| **Slice resolution** | a declared slice names an act instance absent from the act vocabulary |
| **Realisation coverage** | a declared slice has no symbols declaring its required roles under its profile |
| **Profile conformance** | a role's declared `must` / `must_not` is violated |
| **Fact realisation** | an entity in the event model has no type declaring `RealisesFact`, or a type names a fact that does not exist |
| **Orphan attribute** | a `Slice` or `RealisesFact` attribute names something that no longer exists |

**Reported, not failed:** the three delta regions and the two ratios. A codebase with unspecified regions is not non-conformant; it is unspecified, and the measure says how much.

**What a green run does not mean.** That the determinations are right, that the act vocabulary is right, or that the realising code does what the determination says. It means every declaration resolves.

---

## 7. The two brownfield modes

**Change-on-demand.** Code moves only when a slice being specified needs something that exists. The delta becomes a *cost estimate per slice* — this one attaches to what exists, that one needs a boundary moved first — and no restructuring is unjustified by a named act.

Its limit, stated: unstructured regions persist indefinitely, and the coverage claim stays local. A green run must never read as system-wide.

**Alongside.** A new architecture supporting specification is built beside the existing one.

This adds a fourth delta state: **specified, realised in the new architecture, and still realised in the old**. If both paths remain live, a determination can be honoured on one and not the other while the store says it is settled.

**Required rule:** for any slice declared under a new profile, the old path is either removed or declared superseded with the cutover named. The checks enforce this; without it, the result is two systems, one specification, and nobody sure which served the last request.

---

## 8. Deliberately absent

- **No code generation.** This binds declarations to code; it does not write it.
- **No inference from naming.** Not from class names, namespaces, folder layout, or base types.
- **No candidate-slice generation** from the delta (R-D).
- **No second determination schema.** The existing one is used unchanged.

---

## 10. The slice-implementation exercise

Building the tool tests whether declarations resolve. It does not test the thing most worth knowing: **whether a determination set plus a profile is specific enough for an actor to build a conforming slice without asking what was meant.**

So the build ends with one slice implemented end to end by a machine actor, against the store and the profile, and the run is instrumented.

**Before the run**, record what you expect the actor to have to invent. Written down first, or the result is rationalised afterwards.

**During the run**, answer every question the actor asks — and record each one verbatim. The question is the datum. Answering is right for shipping and destroys the measurement unless the transcript survives.

**After the run**, report per category: settled by the profile, settled by a determination, asked about, silently decided. A slice built with six clarifications is a different result from one built with none.

**What this is not.** Not blinded, not controlled, no arms, no falsifier, no claim attaches. It is an observation on real work, and its value is that the categories it surfaces can be compared against the category list being enumerated independently for the notation experiment. **If the two lists disagree substantially, that is worth knowing before that experiment's quantities are fixed.**

**And the boundary, stated so a smooth run is not over-read.** The profile, the schema and the determinations are all authored by the same party running the exercise. A clean run shows the design is internally coherent. It says nothing about whether it is legible to anyone else, which remains open at §11.2 however well this goes.

---

## 11. Open

1. **No falsifier.** The candidate: a codebase with declared slices and profile conformance should show fewer determinations honoured in one path and not another than one without. Untested, and it requires two comparable codebases.
2. **Attribute overhead.** Whether teams will place attributes at all is unknown, and this design fails completely if they do not. It is the load-bearing adoption assumption and nothing here mitigates it.
3. **Reachability is coarse.** Call-graph reachability over-reports in a DI-heavy codebase, where nearly everything is reachable through a container. The reachable-undeclared ratio may be near-useless without a better notion of reach.
4. **Profile rule expressiveness.** `must` / `must_not` as prose strings are checkable by a person, not by Roslyn. Which subset becomes analyser-enforceable is undecided, and the honest position is that a profile carries both: rules the analyser runs, and rules a reader checks — declared as which.
5. **R-B has no mechanism.** Nothing stops someone widening an act to fit a class. It is a rule enforced by reading, like the others in this class.
6. **A working tool is weak evidence.** With a machine actor implementing it, almost any design can be made to run; the difficulty has moved from *can this be built* to *was this worth building*. What would make this design wrong is fixed at §12 before any code exists.

---

## 12. What would make this design wrong

Fixed before implementation, so a working tool cannot retroactively settle them.

- **Reachability is useless.** If, on a real DI-heavy solution, more than ~90% of types are reachable from some declared entry point, the reachable-undeclared ratio does not discriminate and §5's two-ratio reporting is dropped rather than defended.
- **The delta regions do not separate.** If *declarable* and *unstructured* cannot be told apart without a judgement call per symbol, the three-region report is a two-region report with a guess attached, and should be reported as two.
- **Profile rules are not enforceable.** If almost no `must` / `must_not` rule can be checked by Roslyn, profiles are prose with a schema around them, and §3 should say so rather than implying enforcement.
- **The slice run needs many clarifications.** If §10's exercise requires more clarifications than categories settled, the profile is not a specification and the gap is the finding.

A prediction is recorded for each of the first three **before** the measurement runs.
