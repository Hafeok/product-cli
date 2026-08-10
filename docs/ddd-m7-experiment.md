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

*Everything below was written after implementation; nothing above this
line was edited afterwards.*

---

## 5. Classification table

Every change outside `ddd-lsp/src/adapter/htmlcss*.rs`, measured as
`git diff --numstat` from the pre-registration commit (`e402d55`) to the
end of the session (commits: phase 1 `81323a6`, phase 3 `d86d5f6`,
phase 2 `298fefa`).

### 5.1 The new adapter module (not classified)

| File | + | What |
|---|---|---|
| `adapter/htmlcss.rs` | 151 | the hostless ADAPTER, the 5-row policy table, layer-graded visibility ranking, the unlayered-mix posture warning |
| `adapter/htmlcss_facts.rs` | 358 | CSS slicing (tokens with typed signatures, class vocabulary, `@layer` order) plus HTML slicing (stylesheet links; script/style bodies opaque) |
| `adapter/htmlcss_pair.rs` | 364 | the pair map, the enrichment hook, the bidirectional class-contract check with explicit coverage |
| tests (`htmlcss_*_tests.rs`) | 312 | facts, one trigger + one non-trigger per row, both contract directions, thresholds, the comment-stripping regression |
| **total** | **1185** | (873 non-test) |

### 5.2 Changes outside the adapter — classified

| File | +/− | Class | What, and why that class |
|---|---|---|---|
| `adapter/mod.rs` | +33/−3 | **event-source leakage** (~17) + glue (~16) | `hosted`, `enrich`/`EnrichFn` and their docs are E1/E2. Module decls, the `all()` entry, and three test touches are glue. |
| `ddd-mcp/src/intercept.rs` | +33/−77 | **event-source leakage** (~28) | The hostless branch, `Option<&mut Host>` through `intercept()`, the enrich call, `None` reference counts, the guarded reject-overlay. The −77 is dominated by the fitness split (§5.4). |
| `ddd-mcp/src/lang_tools.rs` | +6 | **event-source leakage** | `with_ready_host` refuses a hostless language with a pointer at `ddd_apply_edit` instead of spawning nothing. |
| `ddd-lsp/src/manager.rs` | +1 | **event-source leakage** | `warm_all` skips hostless adapters. |
| `adapter/{csharp,bicep,rust}.rs` | +2 each | **event-source leakage** | Consequential restatement of the two new fields. No policy row touched — the same category as M6's ±2 restatements. |
| `ddd-core/src/config.rs` | +43 | glue (predicted §4.4) | `PairConfig`/`PairUnit` + the three `detect` lists, format 4. The html/css field names are the predicted *naming* category — routing data, no language semantics. |
| `ddd-core/src/validate.rs` | +12 | glue | The format-4 gate check, mirroring the format-2/3 gates. |
| `ddd-core/src/{detect,stylelintconfig,htmlvalidateconfig,tokenfile}.rs` | +364/−3 | **manifest wiring** (§1.1) | Per-tool configured+emitted parsers plus the token-file source — the `cargolints` category, outside the adapter contract by M6 precedent. Includes their unit tests. |
| `ddd-cli/src/commands/report.rs` | +39/−4 | manifest wiring | The pair-contract section of `report escapes` (core cannot call ddd-lsp, so the join is CLI-level). |
| `ddd-core/src/lib.rs`, `ddd-mcp/src/lib.rs` | +4 | glue | Module registration. |
| `docs/{ddd-format-migrations,predicate-format}.md` | +26 | glue | The format-4 migration note (with the documented tool invocations) and `html-css` joining the artifact-class enumeration. |
| `ddd-core/src/render.rs` | +5/−12 | dogfood (phase 3) | The CSS constant became `include_str!` of the governed asset. Predicted clean and *not forced by admitting the language* — it is the governing of the surface, done on purpose. |
| tests + fixtures | +558 | test / fixture | `webpair_governed.rs` (180), governance webpair module (136), the render-sample test (50), the visual harness (85+32), the committed webpair fixture set with real stylelint 17.14.1 / html-validate 11.6.2 outputs. |
| `.ddd/` graph entries, `.stylelintrc.json`, `.htmlvalidate.json`, `render.css`, the sample page | — | dogfood / record | Claims, decisions, manifests, 38 correspondence rows, 19 seam declarations, the governed pair itself. |

### 5.3 The numbers

| Class | Lines added outside the adapter |
|---|---|
| **Contract-surface leakage** | **0** — `surface.rs`, `classify.rs`, `seam_event.rs`, `protocol.rs`, `host.rs`, `client.rs`, `govern_tools.rs`, `seam.rs`, `why.rs`, `store.rs` all byte-identical to their pre-M7 state |
| **Event-source leakage** | 2 instances (E1, E2), ≈58 lines |
| Genuine core bug | 0 (one *adapter* bug, §6.3) |
| Registration glue | ≈75 |
| Manifest wiring (outside the adapter contract, §1.1) | ≈400 |

### 5.4 Two fitness-gate splits, disclosed

The 400-line file gate forced two behavior-free splits mid-session:
`csharp.rs` → `csharp_facts.rs` (pushed over by preamble 1's enum-member
row, not by M7) and `intercept.rs` → `edits.rs` (pushed over by E1's ~28
lines — so this split *is* part of the event-source cost's footprint,
though it moved only pre-existing code). Neither changed behavior; both
are excluded from the leakage line counts above and disclosed here so the
numstat reconciles.

### 5.5 Prediction scored

| Predicted | Outcome |
|---|---|
| E1 — the serve layer assumes a spawnable host per adapter → forced, 30–60 lines | **Correct.** ~46 lines (mod.rs host half, intercept, lang_tools, manager, 3×restatement). |
| E2 — single-document facts; pair facts need an enrichment hook → forced, 10–20 lines | **Correct.** ~12 lines (mod.rs hook half + the intercept call site). |
| `&[RawSymbol]` dead weight, `reference_count: None`, `language: htmlcss` on both files, mock parser unreached, `adapter.<lang>` switches unused → latent | **Correct**, all five. One latent consequence not itemised in advance: with no document identity in the facts signature, the adapter dispatches CSS-vs-HTML by content sniffing (`looks_like_html`) — adapter-local, cost ~6 lines *inside* the adapter, but it is E2's fingerprint and belongs in DDD-adapter-05's evidence. |
| `surface.rs`/`classify.rs`/`seam_event.rs`/`govern_tools`/`seam`/`why`/`store` clean | **Correct for all.** `render.rs` was also predicted clean and moved — for the phase-3 dogfood extraction, not for language admission (§5.2). |
| The PolicyRow-lacks-an-`extra`-matcher risk → resolved adapter-locally or it is leakage | **Resolved adapter-locally.** Ownership and decorative exemptions live in the enrichment (drop) and the facts fn (never lift); `surface.rs` gained no matcher. The M6 orphan-impl shape repeated exactly. |

Nothing unpredicted leaked. The one file outside the prediction that moved
for non-dogfood reasons was none.

---

## 6. Governed-tail reading

The `html-css` class went to `enforce` (dec/web/htmlcss-enforce) before
any authored HTML/CSS landed. Three governed edits ran through
`ddd serve` stdio — the same batch-driver shape M6 needed, unchanged —
producing **38 correspondence rows** (seam-event/9–46) and **19 seam
declarations**.

| What happened | Count |
|---|---|
| Contract-surface edits rejected, then declared, then applied | 6 events-rounds / 3 edits |
| Surface events demanded (10 classes, 7 palette tokens, 2 surface tokens) | 19 |
| Declarations authored, all symbol-granular | 19 |
| Declarations with empty `verdict_knowledge` (the rubber-stamp signal) | 0 |
| Non-surface pair edits in the same window (token value changes, sample regeneration by the renderer) | no events, by design |

### 6.1 The class has a different edit rhythm, and it showed immediately

Rust's rhythm is one or two symbols per edit; M6's whole tail demanded 4
declarations. A stylesheet's rhythm is different: **its birth lands the
entire vocabulary at once.** The first governed edit (the verbatim
extraction of the render CSS) produced one rejection carrying ten
class demands, and preamble 2's symbol-granular matching — correct, and
freshly landed — demanded ten separate declarations for it. The
declarations were real (each status class genuinely encodes verdict
knowledge; the sentences wrote themselves), but the shape is clear: for
this class, a *vocabulary-granule* declaration — one seam covering a
named set of symbols landing together — may be the right amendment.
Filed as an observation for the principal, not applied; the
correspondence rows (ten rows, one edit, ten declarations) are the
evidence a future decision would cite.

### 6.2 What was absent: the warmup tax

M6's friction reading led with the 14-second first-call warmup and the
discarded `loading` call. The hostless tail had **none of that** — the
first `ddd_apply_edit` of the session answered in milliseconds. The
absence localises the loading cost cleanly: it is a *host* cost, not an
interception cost, which is what DDD-adapter-04 would predict.

### 6.3 Dogfood caught an adapter bug on first contact

The first `report escapes` run against the real pair flagged four "dead
selectors" that were CSS comment prose — the pair check's selector walk
did not strip comments, though the facts side did. Fixed by sharing the
facts side's stripper; regression test added. Adapter-internal (both the
bug and the fix), so it does not classify above — but it is worth
recording that the check's first real finding was about itself, and the
second real run came back honest: 20 selectors checked, `:root` reported
as outside the modelled subset rather than silently passed.

### 6.4 The voluntariness caveat, half-answered for this class

M6 §5.3 stands: routing an edit through `ddd_apply_edit` is voluntary,
and nothing about M7 changes that for *declarations*. But this class is
the first whose contract has an after-the-fact gate regardless of
routing: a class introduced or stranded outside the tool becomes an
`orphan-class`/`dead-selector` finding in `report escapes`, and a token
added outside the manifest becomes `UNGOVERNED tokens/--x` in `ddd diff`
— the analogue of `ddd what --strict`, which code still lacks. The pair's
seam failures are now findings whether or not the interceptor was in the
loop; only the *declaration* discipline remains voluntary.

### 6.5 One measured gap in the pair facts

The first 34 rows carry `pair_counterpart: ""` — the sample page did not
exist yet when the vocabulary and palette landed, so the enrichment had
nothing to resolve against. The final governed edit (the surface tokens),
run after the sample was committed, produced rows carrying the full pair
facts: both files, direction, token id. The gap is honest (the counterpart
genuinely wasn't there) but it documents an ordering sensitivity: pair
facts are only as complete as the pair map's resolution at edit time.

### 6.6 Preamble 2, exercised at n=10

The ten-class edit was the first real test of symbol-granular enforce
matching: ten surface events, ten declarations, and every row's
`linked_declaration` names its own symbol — zero of the mis-attributions
that corrupted `seam/rust/default-command` at M6. The corrupted M6 entry
itself remains untouched, per the ruling; the regression tests stand in
its place.

---

## 7. Proposed status consequences — for the principal, not applied

Nothing below was applied when this section was written. **Ruled
2026-08-10 (Emil): all three accepted** — §7.1 applied as the promotion
to `established` with the evidence extended on `DDD-adapter-01`, §7.2
filed as `.ddd/claims/DDD-adapter-05.yaml`, §7.3 applied to the PRD §11
risk row, and the §7.4 working entries plus the 19 staged seam
declarations accepted as reviewed. The §6.1 vocabulary-granule
observation stays an observation — deferring it was costless and no
decision was taken.

### 7.1 `DDD-adapter-01` — survived its scheduled falsification attempt; **strengthen**

The claim's falsifier — *contract-surface knowledge that cannot be
localised to a language adapter* — was given its best shot yet: an
artifact class whose contract surface is not symbol events at all, whose
producer is not an LSP host, and whose governed unit is two files. Every
piece of pair knowledge localised: token typing, class vocabulary, layer
grading, link rewiring, ownership, decorative thresholds, and the
cross-file checks all live in the adapter module; the surface vocabulary,
the classifier, and the row schema are byte-identical to their pre-M7
state. The PolicyRow-expressiveness gap (no `extra` matcher) repeated
M6's orphan-impl shape and was absorbed the same adapter-local way.

**Recommendation: extend the evidence with the language-four result and
promote `reported` → `established`** — the claim has now held across
four languages, three fact producers (two LSP hosts of different
lifecycle shapes, the What's typed graph, and a hostless parser), and one
scheduled falsification attempt at its own falsifier. If the promotion
feels early at n=4, the fallback is unchanged wording at `reported` with
the evidence extended; the drafted evidence text works for either.

### 7.2 `DDD-adapter-05` — proposed as new, **reported**: the event-source half

Drafted for filing on acceptance:

> **Statement.** The seam-event abstraction's *contract-surface* half is
> producer-independent, but its *event-source* half assumed one producer
> shape: a language server per language, one document per edit. Admitting
> a producer outside that shape costs bounded, named changes in the
> serve/host layer — never in the surface vocabulary. At M7 a hostless
> two-file producer cost 2 assumptions / ≈58 lines: (E1) every adapter
> has a spawnable host — `hosted` plus a hostless classification path;
> (E2) a symbol's facts derive from one document's text — an `enrich`
> hook for facts only the repo context carries (counterpart, direction).
> The adapter-local content-sniff dispatch (`looks_like_html`) is E2's
> fingerprint: the facts signature carries no document identity at all.
> This generalises DDD-adapter-04 (whose scope is LSP host *lifecycle*)
> to the producer boundary itself: each new producer *shape* — not each
> new language — is a budgeted core cost, and rust-analyzer's 114 lines
> plus this 58 are the first two entries in that budget's ledger.
>
> **Falsifier.** (a) A fifth producer shape (e.g. a build-system-backed
> producer, or a genuinely multi-document N>2 unit) forcing changes
> *outside* the two seams now expressible (`hosted`, `enrich`) — that
> would show the generalisation is still enumerating cases, not done.
> (b) Contract-surface leakage at any future producer — which takes
> DDD-adapter-01 down first and this claim with it.

### 7.3 PRD §11 amendment — proposed wording

The scope-creep row's mitigation currently ends at the M6 re-pricing.
Proposed continuation: *"…and the event-source boundary is priced the
same way (M7): a producer outside the LSP-host shape costs bounded
serve-layer changes (2 assumptions / ~58 lines for the hostless pair
producer), never surface-vocabulary changes; each new producer shape is a
budget line, each new language within a known shape is adapter-only."*

### 7.4 Working entries filed during the milestone (already in the graph, flagged for review)

`DDD-web-01` (the demand claim, reported), `DDD-web-02` (the palette
claim, projected), `DDD-web-visual-01` (the screenshot-diff evidence
path, reported, version-indexed to the Chromium build), the two ruled
decisions recorded verbatim from the session brief
(`dec/web/plain-pair-scope`, `dec/web/htmlcss-enforce`), and the three
seeded governance decisions (`dec/web/render-status-palette`,
`dec/web/token-discipline`, `dec/web/markup-validity`). These are
milestone working content in the M5/M6 tradition, not experiment status
consequences; the 19 seam declarations are likewise staged in the diff
for review.

---

## 8. What M7 leaves standing

- **The vocabulary-granule question** (§6.1): symbol-granular enforce is
  correct and now proven at n=10, but a stylesheet's birth suggests a
  set-granule declaration form. Observation for the principal.
- **Razor/Blazor and JS/TS** stay out per `dec/web/plain-pair-scope`;
  `<script>` bodies were opaque bytes throughout.
- **Chip backgrounds and the remaining raw surface values** are not
  tokenised — the seeded stylelint rule scopes to `color:` properties.
  Wider tokenisation and rule curation are a future curation session's
  content by design (the anti-goal held).
- **Emitted web-tool outputs are fixture-proven but not wired into this
  repo's CI** — the invocations are documented in the format-4 migration
  note; running Node tools in CI is a decision nobody has taken.
- **M6 §5.3's routing-voluntariness** remains open for the `code` class;
  this class now has its post-hoc gate (§6.4), code still does not.
