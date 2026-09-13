# Gate 0 — orientation

**Register discipline.** Three registers, kept apart in every section: **in force** (the governance
ref, the prompt's prohibitions, the binding's schema), **`[PROPOSED]`** (everything this session
designs), **`[OPEN]`** (what only Emil can settle). Nothing below is ratified. No implementation
code was written for this gate.

**Governance ref.** `Hafeok/canon-governance` `c5383be06e5b181dc79307554a35cddeacbcd3e8`. The
invocation's `<REF>` arrived unsubstituted; head of `main` was taken and is recorded in
`bootstrap.md`. **`[OPEN]` O-1:** confirm this ref, or name the intended one.

**A defect in this session's own filing, reported.** The first draft of `arrived-inputs.md`
carried the determination schema's sha256 with one nibble wrong (`…cbb6…` for `…ccb6…`). It was
caught by a scripted comparison of every claimed hash against the filed bytes, run before the
bootstrap commit, and corrected before anything was committed. No record was rewritten; it is
reported here because a transcribed hash that nobody re-derives is the failure the filing rule
exists to catch, and it nearly happened.

---

## 1. The PRD — unsound, ambiguous, or unimplementable as written

Numbered so rulings can cite them. Severity is this session's reading.

**P-1 — The delta is a measurement of code *against a specification*, and the specification is the
thing a brownfield solution does not have.** *(load-bearing)* Every region in §5 is defined
relative to the act vocabulary: *declared* needs a `[Slice]` naming an act; *declarable* needs "a
symbol that corresponds to an act"; *unstructured* needs "the act boundary runs through this type".
The two ratios are defined relative to *declared* slices. On a real solution with zero attributes
and no event model, there are no declared slices, so reachable-undeclared is 0 of N by
construction, isolated-undeclared is everything, and no act exists for a symbol to correspond to.
The prompt's "it needs no adoption to test" holds for the tool's mechanics and not for the measure.
Gate 1 therefore needs, besides the solution: its act vocabulary and fact vocabulary in the
binding's `eventmodel.yaml` form, authored by Emil or the owning team — never by this session from
the code (R-A). Without them Gate 1 produces an inventory and a reachability structure, which is
not a delta and must not be reported as one.

**P-2 — "Declarable" has no mechanical criterion, and any non-mechanical one is a per-symbol
judgement — which is §12 condition 2 firing by construction.** "A symbol exists that corresponds to
an act" is not computable from attributes alone, and computing it from names or base types is
prohibited. A separator that is mechanical *is* available, one level down: with `[RealisesFact]`
declarations in place, a type's fact-references are facts, and every act's read and write sets are
in the vocabulary. Then: a type that references realised facts belonging to exactly one act is
*declarable* for that act; a type that references realised facts belonging to two or more acts has
an act boundary running through it and is *unstructured*, with the acts named; a type that
references no realised fact is neither and reports only in the reachable/isolated ratios. This
makes the three-region report depend on `RealisesFact` adoption rather than `Slice` adoption — the
cheaper of the two, since facts are fewer and more stable than acts — and it is honest about the
dependency: with no `RealisesFact` declarations the report collapses to two regions, which §12
says to report as two. `[PROPOSED]` as the separator; Emil's condition-2 prediction should be made
knowing this is the criterion the measurement would use.

**P-3 — §9 does not exist.** The status line says "the one candidate [falsifier] is at §9". The
document runs §8 → §10; the candidate is at §11.1. A dangling pointer in a document about dangling
references. Reported, not corrected — the PRD is an arrived input.

**P-4 — A profile cannot be filed as a determination *addressed to the act type*, under the schema
unchanged.** `$defs/address` requires `act_instance` (`minLength: 1`), and `check_resolution.py`
fails ADDRESS when the instance is not a slice the model names. There is no type-level address. The
schema's own worked example shows the form that *is* available: `DSC-0002` addresses one instance
(`PlaceOrder`) with `extent.axes.slice-type: travels-to all-command-slices`. `[PROPOSED]`: a profile
determination addresses one representative instance and travels; the act type is carried by the
address's `act_type` plus the travel region. The alternative is a schema change, which is
prohibited.

**P-5 — The profile body has nowhere to go inside a determination.** Root `additionalProperties:
false`; `statement` is a string. Roles, `must`, `must_not` cannot be encoded in the record. So "filed
as a determination" can only mean *referenced by* one: the profile document (with its own schema,
which the prompt's Gate 2 explicitly calls for) lives beside the store, and a pinned determination
names it in `allocation.settled_by` (e.g. `profile:rest-api-v1@<sha256>`). Pinning by content hash
makes "changing the architecture is a supersession" mechanical: a changed profile has a new hash,
the old determination still names the old one, and a new determination `supersedes` it. This is
not a second determination schema — the profile is not a determination — but the reader should see
that the profile *artefact* is a new artefact type. Flagged so it is not mistaken for one.

**P-6 — The attribute sample violates R-C.** `[Slice("PlaceOrder", Role.Handler)]` compiles the role
vocabulary into an enum in the attributes package. Roles are profile-defined and profiles are
determinations; an enum is "a rule with no address". `[PROPOSED]`: `[Slice("PlaceOrder", "handler",
Profile = "rest-api-v1")]` — strings, resolved Rust-side against the profile store. The attribute
package then carries no vocabulary at all.

**P-7 — The handler-role rules cannot be reused *directly*, because the two act vocabularies do not
carry the same information.** The Decider rules in `product-core` (`derive_decider`,
`validate_decider`, `rules_decider.rs`: decides-for-entity, no-foreign-commands, command-coverage,
output-alphabet-containment, state justification, Decider justification) run over the What graph,
where a command `targets` an aggregate and `emits` events. The binding's `eventmodel.yaml` has only
`slices: {type, name, reads, writes}` — no aggregate, no targets. Reuse is real but takes an edge:
the act instance names a command in the What; the Decider handling that command supplies the
signature; the C# facts (which `[RealisesFact]`-declared types a handler's members reference) are
checked *against that signature* — emitted facts ⊆ `emits` is output containment, `handles` ⊇
commands is coverage, and so on. The rule *definitions* are reused; what is new is the mapping from
C# facts to the rules' inputs. Where no Decider exists for the act, the handler role has only
containment and coverage over the slice's `reads`/`writes`. `[OPEN]` O-5: whether the `.product/`
What graph is present for the solution Gate 1 runs on; if not, the handler role's rule set at Gate
2 is the slice-level subset only, and that is stated on the profile.

**P-8 — §7's "the checks enforce this" names a check §6 does not list.** The alongside-mode rule
(old path removed or declared superseded with the cutover named) is a sixth check or an overclaim.
The prompt says five. `[PROPOSED]`: five checks are built; §7's rule is documented as
*read-enforced* at Gate 2 unless Emil rules a sixth.

**P-9 — Profile conformance can only *fail* on analyser-enforced rules.** A read-enforced rule is
a reader's obligation, and the check cannot fail on it. §6's table does not say so. Gate 2's
per-rule declaration (analyser-enforced / read-enforced) is what makes the table honest; the check's
output must list the read-enforced rules it did *not* evaluate beside the ones it did — the same
"uncovered set stated at the point of the claim" discipline as `does_not_cover`.

**P-10 — Reachability through DI is a judgement the inventory must not make.** Constructor
injection appears as a type reference to an interface; reaching the implementation requires knowing
which registration wins, which is runtime configuration. `[PROPOSED]`: the .NET tool emits
`implement` edges as facts; the Rust side traverses them or not under a flag, and *reports which*
beside every ratio. §12 condition 1 is measured both ways.

**P-11 — "Change coupling" as a cluster dimension needs the solution's git history.** Not in the
inventory (it is not a fact about the code). `[PROPOSED]`: the Rust side reads `git log` of the
solution path when present; the cluster dimension is reported as unavailable otherwise.

**P-12 — Where the store lives is unspecified.** §4 says "the determination store" and never places
it. `[PROPOSED]`: `.product/products/<name>/determinations/*.yaml` (each file a list validating
against the vendored schema unchanged), `eventmodel.yaml` beside it, profiles under
`.product/products/<name>/profiles/<id>.yaml`. Resolved through `pf::paths::product_base`, as every
other per-product artefact is.

**P-13 — Vocabulary collision, recorded.** The binding's "slice" is an act instance. `CLAUDE.md`
already separates three senses of "slice" in this repository (§7.1 feature, the atomic work unit,
the module pattern) and the binding's is closest to the second. The attribute stays `[Slice]`
because the PRD and the binding name it so; no Rust module, command or file is named `slice`.

---

## 2. Where this lands in `product-cli` — reuse and duplication

**Crate placement `[PROPOSED]`.** Rust side in `product-core/src/pf/` as pure slice modules, thin
adapter in `product-cli/src/commands/`, per the repository's Slice + Adapter rule. Proposed
modules, each under the 400-line gate: `determination.rs` (record loader + schema validation),
`eventmodel.rs` (act/fact vocabulary loader — a port of what `check_resolution.py` reads),
`csharp_inventory.rs` (the versioned inventory type + the version refusal), `csharp_delta.rs`
(regions, ratios, clusters), `csharp_checks.rs` (Gate 2). Command family `product csharp
{inventory-check, delta, check}` — named by the stack, so a second stack gets a sibling family.
None of these mutates the graph, so no MCP mirror is owed at Gate 1; Gate 2's profile filing
writes a determination and will owe one (`CLAUDE.md`, *Adding a New Command* step 7).

**.NET side `[PROPOSED]`.** A console project under `tools/csharp-inventory/` (not a Cargo
workspace member) on `Microsoft.CodeAnalysis.Workspaces.MSBuild` + `Microsoft.Build.Locator`.
Workspace CI stays hermetic per `dec/ddd/fixtures-not-sdk`: Rust tests run on a committed inventory
*fixture* produced from a small fixture solution, and the invocation that regenerates it is
documented — the same pattern the SARIF fixtures use. The fixture is test data, not canon, and is
committed as test data; the inventory of the *real* solution is never committed (carried risk 4).

**What is reused, not rewritten.**

| Need | Exists as | Used how |
|---|---|---|
| YAML/JSON loading | `serde_yaml` / `serde_json` in `product-core` | loaders |
| Schema validation of determinations | `jsonschema = "0.46"` — **dev-dependency of `product-cli` only** (`server.json` test) | promoted to a `product-core` dependency; the vendored `determination.schema.json` (draft 2020-12, `if`/`then`, `oneOf`) is loaded and applied **unchanged**. Hand-rolling its constraints in Rust would be a second schema by another name |
| Store location | `pf::paths::product_base` | resolves `<home>/determinations/`, `<home>/profiles/` |
| Atomic writes | `fileops::write_file_atomic`, `RepoLock` | Gate 2's filing verbs |
| Error model, exit codes | `ProductError` | every failure |
| Output seam | `CmdResult` / `Output::Both` | `--format json` for free |
| Handler-role rules | `pf::decider::{derive_decider, validate_decider}`, `rules_decider.rs` | Gate 2, via the mapping at P-7 |
| Attribute-as-declaration discipline | `ddd_core::surface::SymbolFacts.decorators` + configured `exported_attributes` | the vocabulary: *decorators are declarations; names are not* |
| Enforcement split shape | `pf::authoring_scope_enforce::EnforceFindings` (accepted / rejected / two gaps) | the *shape* of the delta report — not the code |
| Schema vendoring | `schema/json/{build-seam,codegen,conformance,authoring-scope}/` | `schema/json/csharp-inventory/` |
| Content hashing | `sha2` | profile pinning (P-5) |

**Duplication found — the thing the prompt says to find.**

**D-1 — A C# fact extractor already exists.** `ddd-lsp/src/adapter/csharp.rs` +
`csharp_facts.rs` read symbols from `roslyn-language-server` over LSP and slice attributes from
source text, under `dec/ddd/lsp-as-seam` ("the core consumes normalized LSP events and never
touches Roslyn … APIs directly") and `dec/ddd/adapter-policy-tables` ("one adapter + policy table
per new language, by design"). The PRD asks for a Roslyn-workspace tool. Building it makes two C#
readers in one repository. The letter of `lsp-as-seam` holds — the .NET tool keeps Roslyn out of
the Rust core by an artefact seam — but "one adapter per language" is strained, and saying the two
answer different questions (change-time contract surface vs whole-solution inventory with a call
graph) is true and is also the kind of rationalisation the prompt warns about. What the LSP path
cannot give: attribute *arguments* as typed constants (it slices lines of text), and a call graph
without one `references` request per symbol. **`[OPEN]` O-3: Emil rules whether the second reader
is accepted with that record, or whether Gate 1 must be attempted over the LSP adapter first.**
This session's recommendation is the .NET tool, with the decision recorded in `.ddd/` as a
decision beside `lsp-as-seam`, not silently.

**D-2 — Two act vocabularies.** `DomainGraph` already carries `commands`, `events`, `read_models`,
`flows`, `triggers` under `.product/`. The binding's `eventmodel.yaml` carries `slices` and
`facts`. The prompt requires the binding's vocabulary unchanged; nothing relates the two. A product
with both has two act vocabularies and no rule that they agree. **Not resolved here** — reported as
the duplication it is. `[OPEN]` O-4.

**D-3 — Nothing in Rust loads a determination.** Searched: `determination` occurs only in `.ddd/`
decision prose and docs; `act_instance`, `DSC-` nowhere in `*.rs`. The loader is new, once, in
`product-core`, and the search obligation (`CG-R-7`) is discharged for this repository: the
prohibitions the schema enforces are held *in the schema file* and nowhere in Rust, which is why
the schema is applied by a validator rather than re-expressed.

**D-4 — `binding` is already a module name** (`ddd-core/src/binding.rs`, seam-binding signing) and
`Slice`/`slice` is triply overloaded (P-13). Neither is used as a Rust name here.

---

## 3. The four rules, in this session's words

**R-A.** The inventory is evidence that symbols exist; acts come only from the act vocabulary. The
tool may say *this act has no realising symbol* and *this symbol realises no act*. It may never say
*this symbol is an act*. Read the same as written. **Flag:** the *declarable* region is one
attribute-placement away from a candidate slice. The reading that keeps R-A and R-D together is
that the delta is **indexed by act**, never by symbol: for every act in the vocabulary — declared,
declarable (a symbol set attaches), or unstructured (a type spans it). Undeclared symbols appear
only as counts by cluster in the two ratios. No per-symbol "could be a slice" list exists in any
output.

**R-B.** The act boundary is fixed by the specification; where a class disagrees, the class moves
or the act is declared not-attachable with the reason recorded. Read the same. **Flag:** §11.5 says
R-B has no mechanism. One exists inside the schema unchanged: a not-attachable declaration is a
determination addressed to the act, `allocation.class: residual` with a named principal, statement
naming the type and why. It is then in the store, addressed, and superseded like anything else,
rather than a comment in a report. `[PROPOSED]` for Gate 2.

**R-C.** How an architecture realises an act type is a determination in the store, not logic in the
tool. Read the same, with one boundary stated so it is not overclaimed: the *meaning* of an
analyser-enforced rule kind ("calls exactly one member of role X", "references no type of role Y")
is logic and lives in the tool as a closed vocabulary of rule kinds; which kinds a profile selects,
with which roles, is the determination. The selection has an address; the kind vocabulary is
documented as the compiled part. P-6 follows from this rule.

**R-D.** The delta lists gaps; it is never a queue of candidates; there is no bulk approval and a
confirmation click is not authorship. Read the same. Consequence: the CLI grows no `--accept`,
`--file-all`, or "declare these" verb. A declaration is a person placing an attribute in code, or a
determination filed one at a time under a principal.

No rule is read differently from the PRD. Two are read *further* (R-B's mechanism, R-C's boundary).

---

## 4. The inventory schema `[PROPOSED]`

Facts only. No region, no role resolution, no slice, no verdict. Every attribute on every symbol
is emitted — filtering to `[Slice]`/`[RealisesFact]` would be the tool deciding what matters.

To be vendored at `schema/json/csharp-inventory/inventory.schema.json`, draft 2020-12, root
`additionalProperties: false`.

```jsonc
{
  "inventory_version": "1",                    // string, required, top-level; Rust refuses any value not in its known set
  "produced_by": { "tool": "csharp-inventory", "tool_version": "0.1.0", "roslyn_version": "4.x" },
  "produced_at":  "2026-09-13T12:00:00Z",      // RFC 3339
  "solution":     { "path": "src/Foo.sln", "git_head": "<sha or null>" },
  "projects": [
    { "id": "P:Foo.Api", "name": "Foo.Api", "path": "src/Foo.Api/Foo.Api.csproj",
      "target_frameworks": ["net8.0"], "assembly": "Foo.Api" }
  ],
  "types": [
    { "id": "T:Foo.Api.Orders.PlaceOrderHandler",   // Roslyn documentation-comment id: stable, arity- and overload-exact
      "project": "P:Foo.Api", "namespace": "Foo.Api.Orders", "name": "PlaceOrderHandler",
      "kind": "class",                            // class | struct | record | record-struct | interface | enum | delegate
      "accessibility": "public",                  // public | internal | protected | private | protected-internal | private-protected
      "is_static": false, "is_abstract": false, "is_partial": false,
      "base_type": "T:System.Object",             // or null
      "interfaces": ["T:Foo.Api.IHandler`1"],
      "file": "src/Foo.Api/Orders/PlaceOrderHandler.cs", "line": 12,
      "attributes": [
        { "type": "T:Product.Binding.SliceAttribute",
          "positional": ["PlaceOrder", "handler"],  // typed constants rendered as JSON scalars, in order
          "named": { "Profile": "rest-api-v1" } }
      ] }
  ],
  "members": [
    { "id": "M:Foo.Api.Orders.PlaceOrderHandler.Handle(Foo.Api.Orders.PlaceOrder)",
      "declaring_type": "T:Foo.Api.Orders.PlaceOrderHandler", "name": "Handle",
      "kind": "method",                           // method | constructor | property | field | event
      "accessibility": "public", "is_static": false,
      "parameters": [ { "name": "command", "type": "T:Foo.Api.Orders.PlaceOrder" } ],
      "return_type": "T:System.Threading.Tasks.Task`1",
      "file": "…", "line": 20, "attributes": [] }
  ],
  "references": [                                 // the graph, as edges; reachability is computed Rust-side
    { "from": "M:…Handle(…)", "to": "M:…Save(…)",           "kind": "call" },
    { "from": "M:…Handle(…)", "to": "T:…OrderPlaced",       "kind": "construct" },
    { "from": "M:….ctor(…)",  "to": "T:…IOrderRepository",  "kind": "type-reference" },
    { "from": "T:…OrderRepository", "to": "T:…IOrderRepository", "kind": "implement" },
    { "from": "T:…Handler", "to": "T:…HandlerBase", "kind": "inherit" }
  ],
  "diagnostics": [                                // what could not be loaded — so a partial inventory says so
    { "severity": "warning", "project": "P:Foo.Legacy", "message": "…" }
  ]
}
```

Edge kinds: `call` (invocation, member→member), `construct` (object creation, member→type),
`type-reference` (parameter, return, field, property, generic argument; member/type→type),
`inherit`, `implement`, `attribute` (symbol→attribute type). Reachability at Gate 1 is the
transitive closure over `call` and `construct` from the declared members, plus `type-reference` to
pull types in; `implement` edges are traversed only under the DI flag (P-10), and the flag's state
is printed beside every ratio.

Symbol identity uses Roslyn's documentation-comment id format (`T:`/`M:`/`P:`/`F:`/`E:` with full
signature), so overloads and generic arity never collide and a rename is a new id — which is what
makes deletion detectable and orphaned attributes findable, and is exactly why a side file keyed by
name was rejected in §2 of the PRD.

**Version refusal.** `KNOWN_INVENTORY_VERSIONS = ["1"]` in `csharp_inventory.rs`; any other value,
or a missing field, is `ProductError` with the offending version in the message, before any field
is parsed.

**Weakest point of this schema:** `positional` attribute arguments are rendered as JSON scalars,
which loses the distinction between a `typeof(X)` argument and a string — acceptable for the two
attributes this binding needs, and stated so a third attribute with a `Type` argument does not
silently degrade.

---

## 5. The brownfield solution — named as far as it can be; access **not** confirmed

**What the listing shows.** The repositories reachable from this session's account include the
Context& application set under `CleverAS-App/` — `Backend`, `Payment-Api`, `Push-Api`, `Locations`,
`Sessions`, `UserAuthentication`, `ServicePortal`, `SelfService`, `Network`, `Environment`,
`AppDashboard`, `Registry`, `CertManager`, `Specification`, `TestKit`. The repository's own
decision records describe Context&'s repositories as application repositories governed under a C#
policy (`dec/cs/policy-rules-at-error`, `dec/ddd/internal-not-surface`). Which of them is C#, and
which is a DI-heavy solution large enough for §12's first condition to mean anything, cannot be
seen from the listing.

**Access attempted and denied.** `add_repo CleverAS-App/Backend` (read) was refused by the session's
permission classifier as an attachment of a third-party organisation's repository. The refusal was
not worked around. Attaching a Context& repository is Emil's act.

**Public fallbacks, and why they are not the answer.** `istern/Sitecore.AutoFixture.NSubstitute`
(a small 2018 library) and `Hafeok/TankWarsCodeChallenge` (a fork) are the only public C# candidates
on the list; neither is a brownfield application, and a measurement over either would settle
nothing §12 asks.

**`[OPEN]` O-2:** Emil names the solution and attaches it, **and** supplies or authors its
`eventmodel.yaml` (P-1). The SDK version that solution targets also matters: no .NET SDK is
installed here; `dotnet-sdk-8.0` is the apt candidate and `builds.dotnet.microsoft.com` is
reachable for others. A solution on a newer target needs that SDK for `MSBuildWorkspace` to load
it.

---

## 6. What Gate 1 needs from Emil before the measurement runs

Per the prompt and `CG-rule-08`, committed **before** the run, no re-roll:

1. **Prediction for §12.1** — the expected percentage of types reachable from declared entry points
   on the named solution, stated for both traversal modes at P-10 (with and without `implement`
   edges), since the DI question is the whole of condition 1.
2. **Prediction for §12.2** — whether *declarable* and *unstructured* separate without a per-symbol
   judgement, made knowing the proposed separator at P-2 depends on `RealisesFact` declarations.
3. **Prediction for §12.3** — what fraction of the profile's `must`/`must_not` rules are
   Roslyn-enforceable. The profile does not exist yet (Gate 2), so this is a prediction about the
   rules Emil expects to write; recording it now is what stops Gate 2 writing only enforceable ones.

And the items at O-2: the solution, access, and its act and fact vocabularies.

---

## 7. Open items, collected

| # | Question | Blocks |
|---|---|---|
| O-1 | Confirm the governance ref `c5383be0…`, or name the intended one | nothing; recorded |
| O-2 | Name and attach the brownfield solution; supply or author its `eventmodel.yaml` | Gate 1's measurement |
| O-3 | Accept a second C# reader (the .NET tool) beside `ddd-lsp`'s adapter, recorded as a decision — or require Gate 1 over the LSP path first | Gate 1's build |
| O-4 | The relation between `DomainGraph`'s commands/events/read-models and the binding's `eventmodel.yaml` | Gate 2 (P-7); not Gate 1 |
| O-5 | Whether a What graph exists for the solution — decides the handler role's rule set at Gate 2 | Gate 2 |
| O-6 | The three §12 predictions | Gate 1's run |
| O-7 | Store layout, command family name, schema path (P-12, §2) — proposed; ratify or rename | Gate 1's build |

## 8. Weakest point of this gate

**The delta cannot be measured without a specification for the solution, and the session cannot
write one.** Every region and both ratios are defined against an act vocabulary (P-1). If the
named solution has no `eventmodel.yaml` and Emil does not author one, Gate 1 measures an inventory
and a reachability structure — which tests whether the tool can be built, the question the prompt
says is *not* the interesting one — and cannot fire or clear any §12 condition. The second-weakest
point is P-2: the only mechanical declarable/unstructured separator found rests on `RealisesFact`
adoption, which is the load-bearing assumption the session cannot test (carried risk 1). Both are
stated here so that a Gate 1 report that looks clean is read with them in view.

**Hold.** Nothing proceeds to Gate 1 without an explicit ratification message.

---

## Appended note — 2026-09-13, after the Gate 0 rulings

**Nothing above is amended.** Rulings CG-R-51 … CG-R-56 are filed verbatim in `rulings-gate0.md`.
What they change about the report's standing:

- **O-1 closed.** `c5383be0…` is the intended ref.
- **P-1 → CG-R-51.** Gate 1 splits: **1a** (inventory, consumer, reachability from Roslyn-found
  conventional entry points; §12.1 fires or clears here) needs no vocabulary; **1b** (three regions,
  two delta ratios, §12.2) needs one. The commercial claim "day-one qualification instrument" is
  corrected by ruling: day one offers an inventory and structural measures; the delta is downstream
  of an authored act vocabulary.
- **P-2 → CG-R-52.** The separator is accepted **as a declared proxy** with three fields (proxy,
  original predicate, known divergence: shared value objects, DTOs and mapping types read as
  unstructured while being shared). The delta output carries those three fields wherever the region
  counts are printed. Divergence dominating the real measurement is a §12.2 firing, not grounds for
  an exception.
- **P-4, P-5 → CG-R-53.** The travel form and the content-hash profile binding are the designed
  pattern, ratified. The **anchor-instance defect** it exposes (a pattern-level determination is
  addressed to an arbitrary, privileged instance; deleting that slice orphans it) is filed as a
  candidate schema change against the binding — `proposal-binding-anchor-instance.md` — not fixed
  here.
- **P-6 → CG-R-54.** `Role.Handler` struck; roles are strings resolved at check time; an
  unresolvable role is a check failure.
- **D-1 / O-3 → CG-R-55.** The .NET reader is admitted as a bounded exception **superseding**
  `dec/ddd/lsp-as-seam` for batch inventory only, under two constraints (facts only; `ddd-lsp`
  wherever both can answer). Filed as `.ddd/decisions/batch-inventory-reader.yaml`, with an
  appended note on `lsp-as-seam.yaml` naming it. Beside-filing was ruled insufficient.
- **P-3, P-8, P-7 → CG-R-56.** §9 corrected and the cutover check added to §6 **in the PRD's
  source by Emil**; the filed copy under `inputs/` stays as it arrived, at its recorded hash. The
  slice-to-aggregate edge is an open item against the binding; the Decider rules are reused where
  the edge exists and the gap is stated where it does not — no aggregate is invented.
- **O-2, O-6 remain open**, both Emil's: the attached solution, and the three §12 predictions
  committed before any measurement. Gate 1a's tooling may be built and tested on a fixture in the
  meantime; no measurement of a real solution runs before the predictions land.
- **O-4, O-5, O-7** are not ruled on explicitly. O-7's proposals (store layout, `product csharp`
  family, `schema/json/csharp-inventory/`) proceed as proposed under Gate 0's ratification and
  remain renameable at Gate 1.
