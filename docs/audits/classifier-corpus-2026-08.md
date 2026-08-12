# Classifier-corpus reading — hand-labelled recall / false-demand (2026-08)

**What is under test.** Spec success criteria 1–2 (classifier recall and
false-demand rate) over the M8 repository-diff path, via the instrument
named in the research protocol §3
([`ddd-research-protocol.md`](../ddd-research-protocol.md)): a fixture set
of labelled changes per language, measured through
`ddd_lsp::revdiff::diff_contracts` — the same shared classifier the edit
interceptor runs (spec invariant 4), so a number measured here is a number
about both paths.

The corpus doubles as the **acceptance harness for policy-table
amendments**: a row change that silently trades recall for friction fails
the harness by case name.

---

## 1. The instrument

- **Corpus:** `ddd-cli/tests/corpus/<language>/<nn>-<name>/` — each case is
  one `before.*`/`after.*` file pair plus `labels.yaml` naming, per changed
  symbol, whether the change **is** contract surface (`surface:`) or **is
  not** (`non_surface:`), under the app-repo default posture
  (`internal_is_surface` off). A `change: any` label marks a symbol whose
  correct classification is *no event at all* (e.g. a body-only edit, a
  same-type token value change); any surface event on it is a false demand.
- **Harness:** `ddd-cli/tests/corpus.rs` — one temp git repo per language,
  each case replayed as a commit pair and classified with `diff_contracts`
  over that pair; one `HostManager` per language, so rust-analyzer starts
  once. Per case, every `surface` row must appear as a surface event and no
  `non_surface` row may; the test fails on any mismatch.
- **Re-run:** `cargo test -p ddd-cli --test corpus -- --nocapture`
  (set `CORPUS_VERBOSE=1` for the full per-case event dump). Runs in normal
  CI; rust-analyzer is a pinned component of the toolchain.

## 2. How the labels were produced — provenance, stated honestly

Labels were written by the M8 implementing session, by reading each diff
against the fixed policy-row semantics (`ddd-lsp/src/adapter/rust.rs`,
`ddd-lsp/src/adapter/htmlcss.rs`) **before** running the classifier over the
corpus — not by copying classifier output. That discipline is what let the
corpus catch a real defect on its first run (§4). The residual caveat: the
labeller also maintains the classifier, so the labels test the
implementation against the table's stated claims, not against an
independent reading of "contract surface". Independent labelling (a second
person, or labels derived from observed downstream breakage) is future
work, and the recall/false-demand numbers below should be read with that
limit attached.

## 3. Measured numbers (2026-08-12, harness passing)

| Language | Cases | Surface labels matched (recall) | Non-surface labels flagged (false-demand) |
|---|---|---|---|
| rust | 12 | 9/9 = **100%** | 0/19 = **0%** |
| htmlcss | 9 | 10/10 = **100%** | 0/3 = **0%** |

Rust covers every policy row from both sides: pub/private fn addition,
pub/private signature change, visibility promotion, `pub(crate)` member
under the default posture, trait-impl addition, pub removal, derive change,
trait-member addition, plus two naturalistic mixed edits. The rust
non-surface count includes the §4 guard rows. HTML+CSS covers token
add/remove/retype, same-type value change (silence expected), class
add/remove, `@layer` reorder, stylesheet-link rewiring on the HTML side,
plus one mixed theme pass.

## 4. What the corpus caught on its first run

The initial run passed vacuously on labels that only named the *intended*
changes. Dumping full event lists showed the classifier also emitting
phantom `signature-changed` **surface** events on symbols the diffs never
touched — enum members (`Red`, `Green`) and a struct field (`id`) — in 6 of
12 rust cases. Cause: `declaration_slice` in
`ddd-lsp/src/adapter/rust_facts.rs` read an 8-line lookahead that ran past
a terminator-less declaration's own extent (an enum member or field has no
`{`/`;`/`=` to cut at), so any edit in the lines below an enum polluted its
members' signatures. The same facts fn feeds the interceptor, so this was
a live false-demand source on both paths.

Resolution: the slice is now capped at the symbol's own range end (a fact
rust-analyzer already reports) — an adapter-facts fix, no policy-row or
classifier-logic change. The affected cases carry explicit `non_surface`
guard rows (`change: any`) for the previously-phantom symbols, so a
regression fails the harness by name; a unit test
(`a_terminator_less_declaration_never_reads_past_its_own_range`) pins it in
`ddd-lsp` as well. No hand label was changed to match classifier output.

## 5. Not covered — scoped follow-up

- **C# and Bicep**: no real host is available in this measurement
  environment (the workspace CI has no .NET SDK — `dec/ddd/fixtures-not-sdk`),
  so their corpora are the named follow-up, to be built where their hosts
  run. Note that `csharp_facts.rs` carries its own unbounded
  `declaration_slice`; whether C# exhibits the §4 bleed is exactly the kind
  of question its corpus must answer — it should be checked first.
- **Posture variants**: all rust cases run the default app-repo posture;
  `internal_is_surface: true` is exercised by the adapter's own tests but
  has no corpus cases yet.
- **Label independence**: see §2 — single-labeller provenance is the
  standing caveat on both headline numbers.
