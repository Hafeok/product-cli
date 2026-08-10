# M6 — the Rust adapter as an adapter-cost experiment

**Status:** pre-registration (prediction filed before implementation)
**Date filed:** 2026-08-10
**Claim under test:** `DDD-adapter-01` (status `projected`), and the broader
PRD §11 mitigation *"new languages require only an adapter + policy table by
design"*.

---

## 1. What is being measured

M6 adds a third language. The measurement is **how much of the core had to
move to admit it**. Every change outside `ddd-lsp/src/adapter/rust.rs` is
classified into exactly one of:

| Class | Meaning | Counts against the claim? |
|---|---|---|
| **registration glue** | wiring the adapter into routing/config | no — expected and bounded |
| **genuine core bug** | wrong for *all* languages, fixed here | no — filed separately |
| **leakage** | the core assumed something C#/Bicep-shaped and had to change to admit Rust | **yes** |

Leakage count > 0 weakens the claim. Zero leakage is the evidence that would
move it to `reported`.

### 1.1 One pre-declared exclusion

`dec/ddd/enforce-matching-tightens-to-symbol` (filed 2026-08-07) says the
enforce-mode matching change "lands in M6 alongside the Rust adapter". It is
**not implemented in this session**, and it is not in any of the three classes
above. Reasons, in order:

1. The session's hard constraint is no feature work beyond the adapter; a
   fourth category would contaminate exactly the number M6 exists to produce.
2. The decision's own stated acceptance test is a property of the
   correspondence rows — *"a row whose `linked_declaration` names a symbol
   other than its own should not occur in enforce mode"*. Running M6's
   governed tail **with the file arm still in place** is what produces the
   first real evidence for that decision. Changing the behaviour first would
   destroy the observation.

M6 therefore leaves it standing and reports what the rows show. The friction
reading in §5 is stated with this caveat attached: it is measured under the
*looser* matching rule and is therefore optimistic.

---

## 2. Prediction (filed before any code was written)

Files suspected of carrying leaked language assumptions, with the call on
whether Rust will **force** a change. "Latent" = the assumption is real but
Rust does not trip it, so it costs nothing here.

### 2.1 Predicted leakage — change forced

**L1 · `ddd-lsp/src/host.rs::initialize` — client capabilities are hardcoded.**
The `initialize` payload is a fixed literal with no adapter hook. rust-analyzer
emits its readiness signal *only* if the client declares
`experimental.serverStatusNotification: true`. Measured on rust-analyzer
1.95.0: with the capability declared, `experimental/serverStatus` arrives at
t+0.15s; **without it, zero such notifications are ever sent**. The core
assumed one fixed capability set serves every host — true for Roslyn and
bicep-ls, false for rust-analyzer.

**L2 · `Adapter::ready_flag` + `ServerState::flags` — readiness is modelled as
"a notification method name was seen at least once".**
That shape is Roslyn's (`workspace/projectInitializationComplete`, which is
sent once and means ready). rust-analyzer's signal is
**payload-discriminated**: `experimental/serverStatus` is sent first with
`quiescent: false` (t+0.15s) and again with `quiescent: true` (t+3.92s). A
method-name-only flag reports Ready at t+0.15s — during indexing — which is
precisely the dishonesty `Readiness::Loading` exists to prevent.

L1 and L2 land as one coherent change but are two distinct assumptions and are
counted as two.

**Note on not dodging this.** A readiness signal *could* be scavenged from
`$/progress` (token `rustAnalyzer/cachePriming`, `kind: end`), which needs no
extra capability and would avoid L1. That would be gaming the measurement:
`serverStatus`/`quiescent` is rust-analyzer's documented readiness protocol and
cachePriming is disableable and primes dependencies rather than the workspace.
The honest signal is used and the leakage is taken.

### 2.2 Predicted latent leakage — no change forced

- **`host.rs::build_inputs`** hardcodes `.sln` / `.slnx` / `.csproj`. Real C#
  leakage sitting in a language-neutral module, but reachable only behind
  `needs_open_handshake`, which Rust sets `false` (rust-analyzer discovers the
  workspace itself — the probe shows `cargo metadata: started/finished` at
  t+1.96s from `rootUri` alone). Costs nothing here.
- **`host.rs::request`** retries on the Roslyn-specific error string
  `"Document is null"`. Harmless for Rust.
- **`ddd-core/src/config.rs::AdapterEntry::internal_is_surface`** — a
  C#-flavoured *name* in the core config schema. Predicted to cost nothing,
  because the concept is language-neutral (crate-internal visibility is
  surface) and Rust's `pub(crate)` is the exact analogue of C# `internal`.
  Reusing the key is what the session prompt's "per the internal precedent"
  points at. Latent *naming* leakage, not structural.
- **`ddd-lsp/src/mock/parse.rs`** dispatches on file extension and falls
  through to a C# parser — language knowledge outside `adapter/`, declared as
  test scaffolding. Predicted to cost nothing: unlike the .NET SDK,
  rust-analyzer is a rustup component of the toolchain this repo already pins,
  so the Rust fixtures run against the **real host**. `dec/ddd/fixtures-not-sdk`
  is scoped to .NET and its rationale does not extend to Rust.
- **`ddd-core/src/cargolints.rs`** already reads `[workspace.lints.clippy]` —
  Rust knowledge that is *already* in the core. It belongs to the manifest/diff
  path, not the adapter contract, and is out of this session's scope. Named
  here so it is not mistaken for something M6 introduced.

### 2.3 Predicted clean — no change expected

- `ddd-core/src/surface.rs` — `SymbolFacts.visibility` and `.kind` are free
  strings and `visibility_rank` is an adapter function, so Rust's graded
  visibility (`pub` / `pub(crate)` / `pub(super)` / private) and its extra
  kinds (`trait`, `trait-impl`) should need no vocabulary change.
- `ddd-mcp/src/intercept.rs` — the whole interception loop.
- `ddd-lsp/src/classify.rs` — the before/after diff.
- `ddd-lsp/src/protocol.rs` — generic LSP shaping.
- `ddd-mcp/src/{dispatch,lang_tools,govern_tools,state}.rs` — the MCP layer.

### 2.4 Predicted registration glue

- `adapter/mod.rs` — `pub mod rust;` and one entry in `all()`.
- Config/docs mentioning `adapter.rust.*`.

### 2.5 A shape the table cannot express (predicted, adapter-local workaround)

`PolicyRow` matches on `changes` / `kinds` / `visibilities` / `exported_only`
only — there is no matcher over `SymbolFacts.extra`. The orphan-impl case
therefore cannot be a row keyed on an `extra` marker. Predicted resolution:
encode orphan-ness in the **normalized kind** (`trait-impl-orphan`), which is
adapter-local and forces no core change. If that works, it is evidence *for*
the claim: a language-specific escape shape absorbed by the adapter's own
normalization.

---

## 3. Measured facts about rust-analyzer (survey, 1.95.0)

From a direct LSP probe against a minimal cargo crate.

| Observation | Value |
|---|---|
| `documentSymbol` answers | t+0.15s, syntactic, before any indexing |
| `cargo metadata` | t+1.96s → t+1.96s (workspace discovery from `rootUri`) |
| `experimental/serverStatus` `quiescent: true` | t+3.92s |
| Same, without the client capability | **never sent** |
| Visibility in symbol payloads | **absent** — no `pub` / `pub(crate)` anywhere |

Symbol shapes (`name`, LSP `kind`, `detail`):

```
Greeter                kind=11  (trait)
  greet                kind=6   detail='fn(&self) -> String'
Person                 kind=23  (struct)
  name                 kind=8   detail='String'
impl Person            kind=19  detail=None
  new                  kind=12  detail='fn(name: String) -> Self'
  secret               kind=6   detail='fn(&self) -> u32'
impl Greeter for Person kind=19 detail=None      <-- trait impl, readable from the name
  greet                kind=6
Colour                 kind=10  (enum)
  Red                  kind=22  (enum-member)
inner                  kind=2   (module)
LIMIT                  kind=14  (constant)
Alias                  kind=26  (type alias)
```

Two consequences for the adapter, both adapter-local:

1. **Visibility must be sliced from source text.** LSP symbol kinds carry no
   `pub(crate)` granularity, exactly as the session's survey item anticipated.
   This is the same technique the C# adapter already uses
   (`declaration_slice` + `declared_visibility`), so it is adapter knowledge,
   not leakage.
2. **Trait impls are readable off the symbol name** — `impl Greeter for Person`
   at kind 19. The boundary-forming trait-impl row needs no extra request.
   Members of a trait arrive with `container_kind == 11`, the same LSP kind
   the C# adapter already keys `interface-member` on.

---

## 4. Classification table

*Filled in after implementation. Empty at pre-registration time.*

## 5. Governed-tail friction reading

*Filled in after the self-governance switch-on.*

## 6. Proposed status change

*Filed as a proposal to the principal, not applied.*
