# Proposed framework additions for v1.8 — the reify layer

**Status:** proposal, targeting `product-framework` v1.8.0.
**Source:** the reference implementation in this repo (`product reify` — PR #22): two
language backends (C#/.NET, Kotlin/JVM), an external-backend plugin protocol, and the
conformance loops, all verified end-to-end against real toolchains. Everything below
is running code here; this document extracts what belongs in the open standard.
**Process:** these changes are authored upstream in `../product-framework` and re-synced
into `docs/product-framework-open.md` (patching the build-seam links back to
`schema/json/build-seam/` as usual).

The organising principle, consistent with §5.1: **every seam is a protocol, not a
linkage.** The framework gains no new verification kinds — every check below lands in
the existing §6.3 table (behavioural conformance, seam). What it gains is the missing
*encodings and protocols* that let any third party realise or verify a What without
using this repo's tooling.

---

## 1. §3.2 — Command and Event payload schemas

**Gap.** §3.2.2 already commits to payload fields normatively — an input AIO "is bound
to a **command payload field**, and that field's type … come[s] from the domain model"
— but neither §3.2 nor §9 defines how a Command or Event *declares* those fields. The
concept was normative prose with no encoding, so payload shapes could only be inferred
from Decider logic and scenarios. With one system that is a private inconvenience;
between two systems (a service and an app) the payload is a **public wire contract**
and must be authored, not guessed.

**Proposed addition to §3.2** (after the building blocks):

> A Command or Event MAY declare its **payload schema**: a list of named fields, each
> optionally typed from the §3.1 datatype vocabulary (`string · integer · number ·
> boolean · date`). The payload schema is the wire contract between systems; where a
> Decider's or Projector's scenarios also exercise a field, a declared type takes
> precedence over anything inferable. Input AIOs (§3.2.2) derive their fields and
> validation from this schema.

**Encoding (§9).** Mirror the entity-attribute encoding:

```turtle
d:PlaceOrder a pf:Command ;
  pf:hasField [ pf:attrName "amount" ; pf:attrType "integer" ] ;
  pf:hasField [ pf:attrName "note" ] .
```

New derivation-contract row:

| Link | Meaning |
|---|---|
| `has_field` | this command/event declares this named, optionally typed payload field (§3.2) |

**Also:** promote the datatype vocabulary — currently named only on `TypeConstraint`
(§3.1 data shapes) — to the single normative type vocabulary for entity attributes,
data-shape constraints, and payload fields.

*Reference implementation:* `Command.fields` / `Event.fields` (`pf/model.rs`), Turtle
emit/parse (`pf/turtle.rs`, `pf/seed.rs`), canonicalization, CLI `--fields name:type`,
MCP `fields` argument; declared-over-inferred precedence in `pf/reify_infer.rs`
(`infer_shape`). Round-trip losslessness proven by the `maximal()` fixture.

---

## 2. §3.3 — the value alphabet (normative)

**Gap.** The Decider/Projector value space (`Scalar`) is boolean · 64-bit integer ·
string, but the spec never states it. The conformance protocol (§3 below) depends on
it, so it must be pinned.

**Proposed addition to §3.3:**

> Decider and Projector values — aggregate state fields, payload values, view fields —
> are drawn from the **wire scalar alphabet**: `boolean`, 64-bit signed `integer`, and
> `string`. A payload field declared `number` or `date` (§3.2) travels **as a string**
> on the conformance wire until the alphabet is extended; the declared type still
> governs generated typed contracts and interface schemas.

Extending the alphabet (decimal, date, lists) is explicitly a candidate for a later
minor version; the degradation rule above keeps v1.8 honest about the limitation
rather than silent.

---

## 3. §6.3.1 (new) — the behavioural-conformance wire protocol

**Gap.** §6.3 requires behavioural conformance — "the realised behaviour or projection
produces identical outputs to [the Decider/Projector] across the same scenarios" — but
specifies no protocol, so only this repo's tooling can currently *claim* it. This is
the §5.1 move applied to verification: fix the contract that crosses the seam, fix
nothing about the far side.

**Proposed new subsection §6.3.1:**

> A **conformance runner** is any process that reads a JSON array of requests on stdin
> and writes a JSON array of outcomes on stdout, one per request, in order, exiting 0.
> Payload values are drawn from the wire scalar alphabet (§3.3).
>
> **Decision requests** (one per Decider scenario): `{ "given": [EventRef…], "when":
> CommandRef }`. The runner folds `given` into fresh aggregate state, decides `when`,
> and answers `{ "emit": [EventRef…] }` or `{ "reject": "<invariant-id>" }`. An
> `EventRef`/`CommandRef` is a bare id string or `{ "event"|"command": "<id>",
> "with": { field: scalar, … } }`; a missing `with` is the empty payload. If a
> response carries both keys, `reject` wins.
>
> **Projection requests** (one per Projector scenario): `{ "given": [EventRef…] }`.
> The runner folds `given` into a fresh view and answers the view state as a JSON
> object of `field: scalar`.
>
> **Equality.** A decision outcome matches the oracle iff the accept/reject verdict,
> the rejected invariant id, the emitted event ids **in order**, and each emitted
> payload match exactly. A projection matches under **full-state equality**: the
> answered object equals the oracle's folded state — an extra field is as
> non-conformant as a wrong one.
>
> Passing this protocol against the model's Deciders and Projectors **is** the §6.3
> behavioural-conformance kind, in any implementation language.

**Schemas:** `schema/json/conformance/` — `decision-request`, `decision-response`,
`projection-request`, `projection-response` (framework repo keeps them under
`preview/conformance/`, this repo patches on sync, mirroring the build-seam handling).

*Reference implementation:* `product decider conform --runner` / `product projector
conform --runner` (`pf/decider_conform.rs`, `pf/projector_conform.rs`); conforming
runners exist in C# (`Program.g.cs`) and Kotlin (`Main.g.kt`), both driven by the same
Rust oracle in the verification record.

---

## 4. §5.2 (new) — the Reify seam: oracle manifest and file plan

**Gap.** Rendering a verification shell for a new language currently requires linking
against this repo. The extension seam should be a protocol, exactly like §5.1.

**Proposed new section §5.2 (peer of the Build seam):**

> The **Reify seam** is the boundary between the deterministic oracle and a language
> backend. What crosses it outbound is the **reify manifest**: the whole oracle **by
> value** — per-aggregate payload schemas (declared §3.2 fields over inference),
> Decider and Projector scenarios, flow chains with every step's outcome pre-computed
> by the oracle, screen facts (surfaces, offers, non-waived degraded states, a
> present-state fixture folded by the relevant Projector), and the graph content hash
> it was computed from. What returns is a **file plan**: `{ "files": [{ "path",
> "content", "overwrite"? }] }`, paths relative and contained (no absolute paths, no
> `..`), `overwrite: false` marking realiser-owned scaffolds written once and never
> regenerated. The consumer — not the backend — appends the provenance manifest
> (§7.3.1), so the drift gate covers every backend's output identically. A backend is
> any process consuming the manifest on stdin and answering the file plan on stdout;
> the framework fixes nothing else about it.

**Schemas:** `schema/json/reify/` — `manifest` (versioned; `manifest_version: "1"`)
and `file-plan`.

*Reference implementation:* `pf/reify_manifest.rs`, `pf/reify_backend.rs`
(`external_plan`), `product reify manifest` / `product reify plugin`, MCP
`product_reify_manifest`. The integration suite drives a 10-line Python backend
through the full loop, including drift detection on its output tree.

---

## 5. §4.2 — the realisation declaration and the delegation tier

**Gap.** Which backend renders a system's verification shell, at which level of
delegation, is a code-shaping §4.2 decision — but it had no home in the contract, so
it leaked into CLI flags. New vocabulary is needed for the tier itself.

**Proposed addition to §4.2:**

> A How contract MAY declare **realisations**: one entry per rendered verification
> shell, each carrying an id, the backend that renders it, the **delegation tier**,
> and optionally the §3.2.5 system it realises, a type/package namespace, an output
> location, and (for external backends) the command implementing the Reify seam
> (§5.2). The delegation tier is one of:
>
> - **`full`** — the backend generates typed domain contracts and frames; the realiser
>   authors behaviour inside them.
> - **`oracle-only`** — the backend generates only the verification shell (the adapter
>   seam, the generated facts, the conformance runner); the realiser owns the entire
>   domain design behind the adapters.
>
> Construction is delegated by the tier; **judgement never is** — at either tier the
> generated facts and conformance runners derive from the model, never from the
> realiser. A declared tier the backend does not support, an unknown backend, an
> external backend without its command, or a `system` link naming an undeclared
> system, are findings. Realisation emission is derived from these declarations; a
> realisation choice living outside the contract is unrecorded How.

*Reference implementation:* `Realisation` on `HowContract` (`pf/how.rs`), §4.2 rules
in `pf/how_validate.rs`, `product reify emit [--id]`, MCP `product_reify_emit`
(`realisation` argument).

---

## 6. §4.4 — the canonical interface projection

**Gap.** §4.4 already mandates that interface contracts are "generated from the domain
model, not hand-written" and names OpenAPI for REST — but without a canonical mapping,
two conforming generators produce incompatible surfaces, and the "derivation link is
the traceability" clause has no concrete form.

**Proposed addition to §4.4:**

> The **canonical REST projection** of the event model is: each command is
> `POST /commands/{command-id}` (request schema = its payload schema; `200` answers
> the sanctioned events in wire form; `409` answers a rejection naming the violated
> invariant), and each read model with a Projector is `GET /views/{read-model-id}`
> (response schema = the projector's view fields). The generated document carries the
> graph content hash as `x-pf-graph-hash` — the concrete form of the derivation link.
> An adopter MAY substitute a different mapping, but a conforming interface contract
> is always generated, always schema-typed from the §3.2 payload declarations, and
> always hash-pinned.

*Reference implementation:* `pf/reify_openapi.rs`; `openapi.g.json` emitted into every
reified tree, identical across languages.

---

## 7. §7.3.1 (new) — provenance stamping and the drift gate

**Gap.** §7.3's `realises_version` is a declaration; nothing makes it checkable
against an actual artifact.

**Proposed new subsection §7.3.1:**

> Every generated realisation artifact SHOULD carry the **graph content hash** of the
> exact specification it was generated from — the hash over the canonical graph
> encoding plus the authored Decider/Projector artifacts — both per-file (a generated
> header) and as a tree-level **provenance manifest** listing the generated files
> (scaffolds excluded: they are realiser-owned). Built artifacts SHOULD surface the
> hash and the realised What-version as embedded metadata (assembly metadata, build
> constants). The **drift gate** recomputes the hash from the current graph and fails
> when it no longer matches a tree's recorded hash: generated code the What has moved
> past is stale by construction, and `realises_version` becomes a *checkable claim* —
> two artifacts stamped with the same hash are realisations of the same specification,
> whatever their languages.

**Encoding:** provenance manifest fields `product`, `namespace`, `what_version`,
`graph_hash` (`sha256:` prefixed), `generator`, `generated_files`.

*Reference implementation:* `product reify check`, `provenance.g.json`, per-file
`pf:graph-hash` headers, .NET `AssemblyMetadata` / Kotlin `PfProvenance` stamping.
Demonstrated: a C# tree and a Kotlin tree pinned to the same hash, both conformant
under the same oracle.

---

## 8. §10 — conformance-rule amendments

Proposed edits to the normative summary (house numbering to be settled upstream):

- **Rule 6 (How contents), append:** "…and, where realisations are rendered, the
  realisation declarations (§4.2): backend, delegation tier, and target system per
  rendered shell — a realisation choice not captured in the contract is unrecorded How."
- **Rule 9 (verification), append:** "Behavioural conformance is claimable in any
  implementation language via the conformance wire protocol (§6.3.1); the protocol is
  the contract, the runner is the executor's concern."
- **New rule:** "Generated realisation artifacts carry the graph content hash they
  were derived from (§7.3.1); a hash mismatch is drift, and no realisation claim
  (`realises_version`, feature `done`, behavioural conformance) is made from a
  drifted tree."

---

## 8a. §11 — the design system as a bound, reify-consumed artifact

**Gap.** §11 (Preview) defines the manifest a conforming design system publishes, but
leaves it a free-floating file a validator inspects after the fact: nothing binds a
What/How to *a particular* design system, the manifest carries no implementation
(no component sources, no token values — no pixels), and reification proceeds happily
past a coverage gap the preview check would have caught.

**Proposed additions:**

1. **The binding is How.** The How contract's screen-composition contract (§4.5) names
   the design system it realises screens against, by id + version. The choice is a
   graph fact, not a CLI flag; a stored system is an addressable artifact
   (`.product/design-systems/<id>/`, vendored by value like every other seam input).
2. **The manifest gains an implementation half** — the *design bundle*: per-component
   implementation pointers keyed by target (`web`, …), token **values** per declared
   theme, and Atomic-Design **templates**. The declaration/implementation split stays
   explicit: §11.3 wholeness checks the declaration; a **bundle check** confirms every
   catalog CIO has an implementation per declared target and every token a value per
   declared theme.
3. **Coupling is a plan-time gate.** Where a design system is bound, a reify run
   resolves `reify(AIO, context) → CIO` for every UI step *before emitting anything*;
   a §11.2 coverage gap fails the plan. The resolved map is emitted by value
   (`design-system.g.json`, hash-pinned) together with the token surface
   (`tokens.g.css`) and a design-system provider seam beside the screen seam.
4. **Drift extends to the design system.** Provenance pins the manifest hash alongside
   the graph hash; the drift gate reports a stale tree when either moves.
5. **A presentation target exists.** The `web` backend renders one page per UI step,
   composed only of catalog CIOs (closed vocabulary, checkable on the output's
   `data-cio` attributes), styled exclusively through token custom properties — the
   reference instance §11's Preview note asks for.

**Implementation:** `pf/manifest.rs` (+`manifest_bundle.rs`), `pf/ds_store.rs`,
`pf/reify_ds.rs`, `pf/reify_web.rs`, the `product design-system` CLI family, and the
`product_design_system_*` MCP tools (How phase). Verified by tc_1090–tc_1094.

---

## 9. Compatibility and versioning

Everything here is **additive** — v1.7.0 graphs remain valid v1.8.0 graphs — hence a
minor bump to **1.8.0**. Two operational notes:

- Adding payload fields to an existing Command/Event changes the graph content hash
  (correctly: the wire contract changed), so reified trees report drift until
  regenerated.
- The §3.3 alphabet limitation (no decimal/date/list scalars) is now *stated* rather
  than discovered; extending it is the natural v1.9 candidate and will version the
  conformance protocol (`manifest_version`, schema `$id`s) rather than mutate it.

## 10. Reference-implementation index

| Addition | Spec home | Implementation | Verified by |
|---|---|---|---|
| Payload schemas | §3.2, §9 | `pf/model.rs`, `pf/turtle.rs`, `pf/seed.rs` | round-trip `maximal()`, tc_1077 |
| Value alphabet | §3.3 | `pf/decider_logic.rs` (`Scalar`) | conform loops, both languages |
| Conformance protocol | §6.3.1 | `pf/decider_conform.rs`, `pf/projector_conform.rs` | .NET + JVM runners vs one oracle |
| Reify seam | §5.2 | `pf/reify_manifest.rs`, `pf/reify_backend.rs` | tc_1079/1080 (Python backend e2e) |
| Realisation + tier | §4.2 | `pf/how.rs`, `pf/how_validate.rs`, `reify emit` | §4.2 unit tests, tc_1081 |
| Canonical REST projection | §4.4 | `pf/reify_openapi.rs` | openapi tests, tc_1077 |
| Provenance + drift gate | §7.3.1 | `reify check`, `provenance.g.json` | tc_1074/1078/1080 |
| Design-system binding + bundle + web target | §11 | `pf/ds_store.rs`, `pf/reify_ds.rs`, `pf/reify_web.rs` | tc_1090–1094 |
