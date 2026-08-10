# M6 — the Rust adapter as an adapter-cost experiment

**Status:** complete — prediction filed 2026-08-10 in commit `a8871da`, before any
code was written; §4 onward added after implementation.
**Date filed:** 2026-08-10
**Claim under test:** `DDD-adapter-01` (status `projected`), and the broader
PRD §11 mitigation *"new languages require only an adapter + policy table by
design"*.

---

## 1. What is being measured

M6 adds a third language. The measurement is **how much of the core had to
move to admit it**. Every change outside `ddd-lsp/src/adapter/rust*.rs` is
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
encode orphan-ness in the **normalized kind** (shipped as `trait-impl-unresolved`), which is
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

---

## 4. Classification table

Every change outside `ddd-lsp/src/adapter/rust*.rs`, measured as
`git diff --numstat` from the pre-registration commit (`a8871da`) to the end
of the session. Adapter-internal lines are shown for scale but are not
classified — they are the thing being paid for.

### 4.1 The new adapter module (not classified)

| File | +/− | What |
|---|---|---|
| `adapter/rust.rs` | +262 | host wiring, the 15-row policy table, visibility ranking, posture warning |
| `adapter/rust_facts.rs` | +351 | visibility grading, signature slicing, impl-shape parsing |
| `adapter/rust_tests.rs` | +227 | facts unit tests |
| `adapter/rust_policy_tests.rs` | +246 | one trigger + one non-trigger per row |
| **total** | **+1086** | |

### 4.2 Changes outside the adapter — classified

| File | +/− | Class | What, and why that class |
|---|---|---|---|
| `adapter/mod.rs` | +65 / −5 | **leakage** (57) + **glue** (8) | `ReadySignal` + `no_extra_capabilities` + the two `Adapter` fields are leakage (L1, L2). `pub mod rust`, the `all()` entry, and the routing test are glue. |
| `host.rs` | +33 / −19 | **leakage** | Adapter-supplied capabilities merged into `initialize`; readiness evaluated through `ReadySignal` rather than a method-name set. |
| `client.rs` | +9 | **leakage** | `ServerState::last_params`, so a payload-discriminated readiness signal is readable at all. |
| `adapter/csharp.rs` | +3 / −2 | **leakage** | Consequential restatement of one field. No policy row touched. |
| `adapter/bicep.rs` | +3 / −2 | **leakage** | Same. |
| `state.rs` | +11 | **core bug** | `raw_str`: `opt_str` trims, and `new_text` is a whole file. |
| `intercept.rs` | +3 / −2 | **core bug** | `resolve_new_text` reads content through `raw_str`. |
| `rust-toolchain.toml` | +3 / −1 | **glue** | `rust-analyzer` as a pinned component. |
| `.github/workflows/product-ci.yml` | +5 | **glue** | The matching `components:` line. |
| `tests/fixtures/rust/*` | +126 | test fixture | The fixture crate. |
| `ddd-lsp/tests/rust_host.rs` | +196 | test | Lifecycle acceptance against the real host. |
| `ddd-mcp/tests/rust_governed.rs` | +425 | test | Row sweep + the interception loop. |

### 4.3 The numbers

| Class | Files | Lines added |
|---|---|---|
| **Leakage** | 5 | **114** |
| Genuine core bug | 2 | 14 |
| Registration glue | 3 | 16 |
| Test / fixture | 3 | 747 |
| *(adapter module — the cost being measured)* | *4* | *1086* |

**Leakage count: 2 instances, 114 lines, all inside the LSP host layer.**

### 4.4 Prediction scored

| Predicted | Outcome |
|---|---|
| L1 — fixed client capabilities → forced | **Correct.** |
| L2 — readiness as a bare method name → forced | **Correct.** |
| `build_inputs` `.sln`/`.csproj` → latent, no change | **Correct** — Rust sets `needs_open_handshake: false`. |
| `"Document is null"` retry → latent, no change | **Correct.** |
| `AdapterEntry::internal_is_surface` → reused, no change | **Correct.** `pub(crate)` rode the C# switch unmodified. |
| `mock/parse.rs` → no Rust dialect added | **Correct**, via `dec/ddd/rust-host-is-real`. |
| `surface.rs`, `classify.rs`, `protocol.rs`, `intercept.rs`, MCP layer, `config.rs` → clean | **Correct for all but `intercept.rs`**, which changed for a bug unrelated to Rust (§4.5). |
| Orphan impls need an `extra` matcher the table lacks → resolved adapter-locally via the kind | **Correct.** Encoded as `trait-impl-unresolved`; no core change. |

Nothing unpredicted leaked. The one file outside the prediction that moved
(`intercept.rs`) moved for a defect that predates Rust.

### 4.5 The core bug, stated plainly

`ddd_apply_edit` had been deleting the trailing newline of every file it
wrote, in every language, since M4. `opt_str` trims and treats blank as
absent — right for names and ids, wrong for a whole file — and
`resolve_new_text` read `new_text` through it. Six files in this workspace
were observed mangled during the governed tail before the cause was found; a
control file edited outside the interceptor kept its newline. Filed as
`DDD-arch-07`, fixed by `state.rs::raw_str`, declared as `seam/mcp/raw-str`.

This is the category the pre-registration set aside as *not* counting against
the adapter-cost claim, and it does not: it is wrong for C# and Bicep in
exactly the same way. What Rust supplied was the first author who read the
bytes back.

---

## 5. Governed-tail friction reading

The `code` artifact class was switched to `enforce` after fixture acceptance
passed, and the rest of the session ran governed: eight `ddd_apply_edit`
calls against this repo's own Rust sources, producing **8 correspondence
rows** and **4 seam declarations** — the first entries `.ddd/seams/` has ever
held.

| What happened | Count |
|---|---|
| Contract-surface edits rejected, then declared, then applied | 4 events / 3 edits |
| Non-surface edits applied untouched | 6 |
| Seam declarations authored | 4 |
| Declarations filed with empty `verdict_knowledge` (the rubber-stamp signal) | 0 |

### 5.1 Where the friction actually was

**It was not in authoring the declarations.** The rejection payload arrived
with the facts pre-filled — symbol, kind, signature, visibility grade,
reference count, the rule and its claim — and with a template whose
`verdict_knowledge` was blank. Nothing had to be re-derived; the only work
was the sentence about what the boundary lets the other side learn, which is
the work the mechanism exists to force. `DDD-friction-01` predicted exactly
this, and nothing in the session contradicts it. Rubber-stamping did not
occur, but four declarations is far too small a sample to say it will not.

**It was in the first call of every session.** `ddd serve` starts hosts
lazily, so the first `ddd_apply_edit` of a fresh session returns
`{"status": "loading"}` rather than an outcome — honest, and correct
behaviour, but it means every governed session begins with a discarded call
and a ~14s warmup on this workspace. An agent that treated `loading` as an
error rather than as "retry" would read the surface as broken. This is the
concrete form of PRD §11's solution-load risk, and it is real for Rust too.

**It was in the tooling around the surface, not the surface.** `ddd serve`
speaks stdio only, and same-session declaration matching requires the
declare and the re-apply to be one process. Driving governed edits from a
shell therefore needed a batch driver holding one session open. An agent
with the tools connected has no such problem; a human at a terminal has no
in-surface path at all, which is the shape `DDD-friction-02` describes.

### 5.2 The reading is optimistic, by a known and now-measured amount

Per §1.1, `dec/ddd/enforce-matching-tightens-to-symbol` was left
unimplemented so its own falsifier could be observed. It fired on the first
governed edit.

That edit added two `pub` items to one file. Two declarations were authored,
one per symbol. Both surface events linked to the **first** declaration,
because `match_declarations` still admits the file arm and returns the first
hit:

```
seam-event/3  symbol: DEFAULT_COMMAND  linked_declaration: seam/rust/default-command
seam-event/4  symbol: host_command     linked_declaration: seam/rust/default-command
```

`seam-event/4` is verbatim the row the decision pre-registered as its
acceptance test: *"a row whose `linked_declaration` names a symbol other than
its own should not occur in enforce mode."*

The consequence is worse than over-admission. `link_seam_metadata` writes the
matched event's LSP-derived facts onto the matched declaration, so
`seam/rust/default-command` — a declaration about a `const` — now carries
`symbol: host_command`, `kind: fn`, `signature: fn host_command() -> Vec<String>`.
A machine-authored structural field on a correspondence entry holds another
symbol's facts, which is precisely what `DDD-arch-04` names as the thing that
makes the dataset unfalsifiable. `seam/rust/host-command`, meanwhile, was
never matched and so carries no LSP facts at all.

The corrupted entry is **left in place** with a `notes` field recording why;
correcting it now would delete the observation. It should be corrected on the
same commit that lands the enforce-matching change.

Read against that: had the file arm not been there, the two edits would have
demanded and linked per symbol, and the friction count would be unchanged —
the arm cost nothing in *effort* here and cost the dataset its integrity.

### 5.3 The larger limit: routing to the interceptor is voluntary

§5.2 is the smaller caveat. This is the bigger one, and it qualifies the
words "self-governance" everywhere they appear above.

Interception binds edits that arrive through `ddd_apply_edit`. Nothing binds
an edit to arrive that way. A contract-surface change made with an ordinary
editor — `pub fn foo()` typed into a file — is never classified, never
demanded, and never logged. There is no after-the-fact check either: neither
`ddd diff` nor `ddd report escapes` computes contract-surface coverage, so an
undeclared `pub` item introduced outside the tool produces no finding at any
later point.

Every one of the eight correspondence rows in §5 is therefore an edit that
was **volunteered**. The switch to `enforce` changed what happened to edits
already routed through the tool; it did not change what fraction of edits
were routed. M6 demonstrates that the interceptor works when invoked. It does
not demonstrate that the interceptor is in the loop.

This matters against PRD §2, which is where the tool's whole justification
sits: *"prompt rules are exhortation an agent can drift past. A tool in the
edit loop is a policy-level commitment — the check is part of the
arrangement, not the agent's residual discretion."* If routing to the tool is
itself discretionary, then so is the commitment, and the difference between
this arrangement and a prompt rule is smaller than §2 claims. Nothing in M6
tests that difference.

`DDD-arch-06` does not cover this. It conditions the dataset on the **surface
stratum** — surface versus non-surface, and interception mode — and states
that the base rate of surface-touching edits is unrecoverable. The **routing
stratum** is a further and larger conditioning it does not mention: what
fraction of edits reached the classifier at all. On this repository that
fraction is unknown and, with the current instrumentation, unknowable.

The What surface has already met this problem and answered it: `product
domain new` on the CLI is not intercepted, and `ddd what --strict` is the
gate that catches what the write path misses. Code has no equivalent gate.
Supplying one — a coverage check that reads the declared seams against the
`pub` surface actually present in the tree — is the obvious shape, and is
filed here as an observation rather than a claim, because it is a decision
for the principal and not a finding M6 established.

---

## 6. Proposed status changes — for acceptance, not applied

Nothing below has been applied to the graph.

### 6.1 `DDD-adapter-01` — `projected` → **`reported`**, unchanged in wording

> Per-language contract-surface definitions expressed as adapter policy tables
> are falsifiable against where boundary defects actually occur; wrong rows
> get fixed in the adapter, never in the core.

Its falsifier is *"a boundary defect class that no policy-table row could have
named — i.e. contract-surface knowledge that cannot be localised to a language
adapter."* M6 looked for one and did not find it. Every Rust contract-surface
assumption localised: graded visibility (`pub(crate)`/`pub(super)`/`pub(in …)`),
container capping, trait definitions and members, trait impls as boundary
participation with no keyword to key on, derive changes as trait-impl
authorship, and the orphan-impl escape — which the table could not express as
a row matcher and which the adapter absorbed into its own kind normalisation
instead, with no core change. The surface vocabulary in `ddd-core/src/surface.rs`
was not touched.

One corroboration worth naming: `DDD-adapter-03` reports that the C# table
drops `enum-member` before any row is consulted. The Rust table lists
`enum-member` in `DECL_KINDS` and demands a variant addition on a `pub` enum.
The gap really was adapter-local — a second adapter did not inherit it.

**Recommendation: promote to `reported`.** Evidence text is drafted and can be
filed on acceptance.

### 6.2 `DDD-adapter-04` — filed as **`reported`** (new)

The host-lifecycle half that `DDD-adapter-01` does not cover. This is where
the two leakage instances land, and it is the honest answer to PRD §11's
broader phrasing: *"new languages require only an adapter + policy table by
design"* is **true of contract surface and false of host lifecycle**, and the
mitigation as written does not distinguish them.

**Recommendation: accept as filed, and amend the PRD §11 risk row to say
"adapter + policy table + any host-lifecycle shape the LSP layer cannot yet
express."**

### 6.3 `DDD-arch-05` — `projected` → **`reported`**

Its exposure was hypothetical until this session. It now has a row id
(`seam-event/4`), a corrupted declaration to point at, and a mechanism —
`link_seam_metadata` propagating the mismatch into machine-authored fields —
that the claim's current wording does not mention.

**Recommendation: promote to `reported` and extend the statement to cover the
metadata-corruption consequence.**

### 6.4 `DDD-arch-02` — `projected`, **unchanged**

*"The LSP protocol carries enough information … to classify contract-surface
edits for C# and Bicep without direct Roslyn or Bicep API access."* Rust now
supports the same conclusion — every fact came from `documentSymbol` plus
source slicing, no rustc and no `syn` — but the claim names two languages and
extending it is a rewording, not a status change.

**Recommendation: leave alone, or reword to name the protocol rather than the
language list. Principal's call.**

### 6.5 `DDD-arch-07` — filed as **`reported`** (new)

The shared-accessor defect. Filed, fixed, and declared. No promotion needed;
listed so it is not mistaken for something the adapter caused.

---

## 7. What M6 leaves standing

- **`dec/ddd/enforce-matching-tightens-to-symbol` is not implemented.**
  Deliberate (§1.1), and the session produced its evidence instead. It should
  land next, together with the correction of
  `.ddd/seams/seam-rust-default-command.yaml`'s metadata block.
- **`dec/ddd/enum-member-gap-priced` is not implemented.** The M5 report
  assigns it to M6, but it is a C#-adapter policy change, which this session's
  constraints exclude. The Rust table demonstrates the fix shape.
- **The M6 and M7 milestone rows are not in the PRD.** §10 ends at M5, and the
  flip condition this session was to evaluate is filed nowhere — see
  `dec/ddd/m6-proceeds-no-flip`. Both need the principal.
- **`ddd render` (M2.5) shipped** — `ddd-core/src/render.rs`, the CLI
  subcommand, and `ddd-cli/tests/render.rs` are all present. Untouched here.
- **PRD §9.3 is unwritten.** The Rust policy table is documented in its own
  claim strings and in this report; the committed PRD is an older revision
  than the working draft (M5 report §6), so it was not edited from here.
