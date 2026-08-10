# M7 — the HTML+CSS pair adapter as a seam-event-abstraction experiment

**Status:** prediction filed before any adapter code was written; §6 onward
added after implementation.
**Date filed:** 2026-08-10
**Claim under test:** `DDD-adapter-01` (status `reported`) — this is language
four, the falsification attempt its evidence text scheduled. `DDD-adapter-04`
is *not* under test: its own falsifier (b) scopes a non-LSP artifact class
out, and choosing adapter-local parsing (§2) means falsifier (a) — a fourth
LSP host with near-zero wiring — is not exercised either.

---

## 1. What is being measured

M7 adds a fourth language — the HTML+CSS **pair**, governed as one unit at
its seam. The measurement is the same as M6's: **how much of the core had to
move to admit it**. Every change outside `ddd-lsp/src/adapter/htmlcss*.rs`
is classified into exactly one of:

| Class | Meaning | Counts against DDD-adapter-01? |
|---|---|---|
| **registration glue** | wiring the adapter into routing/config | no — expected and bounded |
| **genuine core bug** | wrong for *all* languages, fixed here | no — filed separately |
| **contract-surface leakage** | the core's *surface vocabulary or classification* had to change to express tokens, class vocabulary, layers, or pair-ness | **yes** — this is the falsifier firing |
| **event-source leakage** | the core assumed the before/after facts arrive from an LSP host serving one document, and had to change to admit a hostless pair producer | no — outside DDD-adapter-01 (its falsifier is scoped to contract-surface knowledge), filed as its own claim in the adapter-04 family |

The fourth class is what M7 stresses that M6 did not. Rust changed the
answer to "how does a host announce readiness"; HTML+CSS changes the answer
to "what produces the events at all". The pair's contract surface — tokens,
class vocabulary, `@layer` structure — is not symbol events from a language
server, so this session tests whether the core's seam-event abstraction
(`surface.rs` vocabulary, `classify.rs` diffing, `seam_event.rs` rows, the
PRD §8 outcome semantics) admits a non-LSP producer without the *surface*
half moving.

Reading the result onto the claim: zero contract-surface leakage at its own
falsifier **strengthens** `DDD-adapter-01`; any contract-surface leakage
**weakens** it. Either way §7 proposes the status consequence to the
principal and applies nothing.

### 1.1 One scope note on the manifest path

`ddd-core/src/detect.rs` and its per-tool siblings (`editorconfig.rs`,
`bicepconfig.rs`, `cargolints.rs`) are the M2 detection machinery, which the
M6 report explicitly placed outside the adapter contract ("the namespace
routing table below is the whole per-language adapter for M2";
`cargolints.rs` named as Rust knowledge that belongs to the manifest/diff
path). The stylelint / html-validate / token-file ingestion M7 adds there is
the same per-tool invocation-adapter category — counted and reported, but
under **manifest wiring**, not under the adapter-cost classes above.

---

## 2. The event source — priced, and adapter-local parsing chosen

The candidate LSP hosts are the vscode-extracted pair:
`vscode-css-language-server` and `vscode-html-language-server`
(npm `vscode-langservers-extracted`). Priced under the DDD-adapter-04
budget, rust-analyzer's 114 lines being the reference point:

- **Two hosts for one governed unit.** The pair is one artifact class; the
  interceptor would have to start, ready-check, and overlay against *two*
  child processes per edit pair, and the `Adapter` abstraction binds one
  host per language — the pair would leak its two-ness into the host layer
  immediately.
- **Two lifecycle budgets.** Each server carries its own wiring line —
  capabilities, readiness shape, workspace behaviour. Both are
  ready-after-initialize (cheaper than rust-analyzer), call it 30–60 lines
  each — but it is 2× per-server cost plus a **Node.js runtime as a
  prerequisite of every governed repo**, for a Rust CI gate.
- **They do not produce the facts the table needs.** The CSS server's
  `documentSymbol` is a selector outline; the HTML server's is an element
  outline. Custom-property *typing* (dimension ↔ color ↔ keyword), class
  *vocabulary* (defined vs consumed), `@layer` membership and order, and
  every cross-file pair fact would still be sliced from source text in the
  adapter — the same technique the C# and Rust adapters already use for
  visibility. The hosts would be paid for and then bypassed.

**Decision: adapter-local parsing, no LSP host.** The Rust precedent applies
verbatim: parsing within the adapter is adapter knowledge, not leakage. This
also keeps the experiment honest — wiring two servers that produce nothing
the table consumes would dodge the very question (does the core admit a
non-LSP producer?) that M7 exists to answer.

---

## 3. Existing-tooling survey for the class-contract check (PRD §10 requires it)

The load-bearing checker: classes referenced in HTML ⊆ classes defined in
owned CSS ("orphan classes"); selectors in owned CSS ⊆ structures present in
the paired HTML ("dead selectors").

- **stylelint** core rules: syntax, complexity, and value discipline over
  CSS alone — no HTML awareness, so neither direction is expressible.
  Its value here is the *closure stack* (§5): rule-id-keyed findings that
  join the M2 manifest machinery, and token discipline
  (`declaration-property-value-disallowed-list` banning raw values where a
  token exists).
- **stylelint-no-unused-selectors** (plugin): unmaintained, JSX-oriented,
  one direction only, Node-resident — a CI gate dependency this workspace
  deliberately avoids (`dec/ddd/fixtures-not-sdk` posture).
- **PurgeCSS / UnCSS**: dead-CSS *removal* tools, not checkers; one
  direction; no rule-id join, no governance join; Node-resident.
- **html-validate**: markup validity with machine-readable output — joins
  the closure stack for the HTML side — but has no cross-file CSS
  awareness.

**Residue to build:** the bidirectional pair check, in the adapter, with the
two finding classes joining `ddd report escapes`. Matching is
necessary-condition decomposition: compound selectors split into simple
parts (class / element / id), pseudo-classes and pseudo-elements stripped,
and any selector containing constructs the checker does not model (attribute
selectors, combinators beyond descent) is checked on its modelled parts and
counted in an explicit coverage line (`dec/ddd/report-coverage-explicit`).
Thresholds — decorative one-off class globs, the pair map itself — are
config, not code (hard constraint 2).

---

## 4. Prediction (filed before any code was written)

### 4.1 Predicted event-source leakage — change forced

**E1 · the serve layer assumes every adapter has a spawnable host.**
`Adapter` carries `default_command`, `ready`, `language_id`,
`extra_capabilities`, `needs_open_handshake` — all host fields — and
`apply_edit` routes unconditionally through `with_ready_host`, which spawns
the child on first use (`manager.host()`); `warm_all` starts every
registered adapter's host. A hostless adapter needs: a way to say it has no
host, a classification path in `intercept.rs` that does not open/overlay a
host document (empty `RawSymbol` slices — the facts fn parses text), no
reference-count requests (rows carry `None`), and guards in
`with_ready_host`/`warm_all`. Estimate: 30–60 lines across
`adapter/mod.rs`, `ddd-mcp/src/intercept.rs`, `ddd-mcp/src/lang_tools.rs`,
`ddd-lsp/src/manager.rs`.

**E2 · facts are assumed derivable from one document alone.**
`facts: fn(&str, &[RawSymbol], &AdapterFlags)` has no document identity and
no repo context, so pair-level facts — counterpart file(s), direction
(definition side vs consumption side) — cannot be produced where facts are
made. Predicted change: one optional enrichment hook on `Adapter`
(`fn(root, file, config, &mut events)`), called by `intercept.rs` after
classification, populating `SymbolFacts.extra` so the seam-event rows carry
the pair facts. Estimate: 10–20 lines outside the adapter.

E1 and E2 are the event-source class: they say nothing about *what is
contract surface* in HTML+CSS — that knowledge is predicted to localise
completely (§4.3). If they land as predicted they are filed as a new claim
(`DDD-adapter-05`): the M6 host-lifecycle finding generalised — the
language-neutral layer had one producer shape (an LSP host per language,
one document per edit), and each producer shape outside it is a budgeted
core cost.

### 4.2 Predicted latent — no change forced

- `facts`'s `&[RawSymbol]` parameter is dead weight for a parsing adapter —
  LSP-shaped, costs nothing (empty slice in, ignored).
- `SeamEvent.reference_count` is `Option` already; hostless rows carry
  `None`.
- `SeamEvent.language` will carry `htmlcss` for both files of the pair; the
  pair identity rides `extra`, not a schema change.
- `mock/parse.rs` dispatches to a C# parser for unknown extensions — never
  reached (the pair adapter's tests need no mock host at all).
- Config `adapter.<language>` entries: `command` is meaningless for a
  hostless adapter (ignored); `internal_is_surface` unused (the pair grades
  by layer, not by a crate-internal analogue); `exported_attributes` unused.

### 4.3 Predicted clean — the DDD-adapter-01 test proper

- **`ddd-core/src/surface.rs`** — unchanged. `SymbolFacts` free-string
  `kind`/`visibility`/`signature` plus the `extra` map are predicted to
  express every pair fact: tokens as `kind: token` with `signature` = the
  value's *type* (`color` | `dimension` | `keyword` | `other`, so a retype
  is `SignatureChanged` and a same-type value change produces **no event**
  — that silence is the point of tokens); class vocabulary as
  `kind: class` with membership = `Added`/`Removed`; `@layer` order as its
  own single symbol whose `signature` is the layer list; stylesheet links
  (`<link rel=stylesheet>`) as `kind: stylesheet-link` — rewiring the pair
  is itself boundary-forming. `@layer` membership grades visibility:
  `unlayered` outranks `layer:<name>`, via the adapter's own
  `visibility_rank`, exactly the mechanism C#/Rust visibility uses.
- **`ddd-lsp/src/classify.rs`** — unchanged; pairing by
  (container, kind, name) and change naming carry over.
- **`ddd-core/src/seam_event.rs`** — unchanged; pair facts ride `extra`.
- **`ddd-mcp/src/govern_tools.rs`**, `seam.rs`, `why.rs`, `store.rs`,
  `render.rs` — unchanged.

Known risk, named in advance (the M6 orphan-impl shape): `PolicyRow` has no
matcher over `extra`, so "only in owned CSS" cannot be a row predicate. The
predicted resolution is adapter-local, again: ownership and decorative
exemptions are resolved in the adapter's facts/enrichment (non-surface
symbols simply not lifted), never in `surface.rs`. If instead `surface.rs`
must gain an extra-matcher, **that is contract-surface leakage and weakens
the claim** — the pre-registered failure mode.

### 4.4 Predicted registration glue

- `adapter/mod.rs`: `pub mod htmlcss;`, the `all()` entry, the routing test,
  and the artifact-class test gaining `html-css`.
- The artifact-class vocabulary: `html-css` joins `predicate-format.md`'s
  enumeration (PRD §13 already admits HTML+CSS as a governed artifact
  class) and `.ddd/config.yaml`'s `intercept_by_class`.
- Config format 4 (`ddd-core/src/config.rs` + migration note): the `pair`
  section (units, decorative-class globs) and `detect` gaining
  `stylelint` / `htmlvalidate` / `tokens` input lists. The field names are
  html/css-flavoured naming in a core schema — the same latent *naming*
  category `AdapterEntry::internal_is_surface` was for C# at M6: routing
  data, no language semantics.
- Manifest wiring (§1.1): stylelint/html-validate config + output parsers,
  token-file parser, `namespace_for` driver routing, CLI `report escapes`
  merge of the pair-check findings.

### 4.5 The policy table, pre-registered

| row | change | kind | surface | claim (short form) |
|---|---|---|---|---|
| `web-token-membership` | added/removed | `token` | yes | custom properties are the stylesheet's params; membership is contract |
| `web-token-retype` | signature-changed | `token` | yes | dimension ↔ color ↔ keyword retype is a contract change; same-type value change is no event at all |
| `web-class-membership` | added/removed | `class` | yes | the class vocabulary is consumed by HTML, defined by owned CSS |
| `web-layer-order` | any | `layer-order` | yes | `@layer` order is the visibility grading of the whole sheet |
| `web-stylesheet-link` | added/removed | `stylesheet-link` | yes | which CSS an HTML document consumes is the pair boundary itself |

Non-surface by default, resolved in the adapter (not rows): inline
`<style>`/`style=` styling local to one file (not lifted as symbols),
decorative one-off classes (config globs, not lifted), a class's property
*body* (a styling tweak inside `.btn { … }` is no event — the vocabulary,
not the styling, is the contract).

Both fixture directions per row (trigger + non-trigger), plus both
class-contract check directions, land in a paired fixture set.

---

*Sections §5+ (classification table, numbers, prediction scored, the
governed tail, proposed status consequences) are written after
implementation; nothing above this line is edited afterwards.*
