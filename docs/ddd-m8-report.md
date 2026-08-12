# M8 Report — Enforcement closure

**Scope:** `dec/ddd/m8-enforcement-closure`, executed under the eight
rulings the principal settled 2026-08-12 (recorded in the ledger — see
§6). **The measure of success, restated from the brief:** a change made
outside the governed path is caught by CI, a declaration proves which
change it discharged, and the record that results is one the principal
can accept entry by entry — because nothing in it was signed on their
behalf.

The chain, end to end, is shipped:

> change → detected contract event → justified obligation → declaration
> signing the change → durable discharge → authoritative CI result

---

## 1. Shipped, by phase

### Phase 1 — the shared classifier over a diff

`ddd_lsp::revdiff::diff_contracts` takes both sides of every changed
file from git and routes them through the *same* `classify_edit` +
adapter policy tables the interceptor uses — a library entry point, not
an MCP re-serve (ruling 3: MCP is a path, not a boundary). Hostless for
the HTML+CSS pair; LSP overlay snapshots for hosted languages, including
virtual opens for files absent on disk. A host that fails or text that
will not read is a skipped-file finding, never a clean result (spec
invariant 9). `ddd diff-contracts <base>..<head>` is the CLI; findings
carry stable ids (`contract/<language>/<file>#<symbol>@<change>`), and
the CI-facing commands (`diff-contracts`, `validate`, `diff`,
`report escapes`) gained `--json` — spec §6's named gap, closed.

One classifier, proven: `revdiff_tests::diff_path_matches_edit_path_on_the_same_change`
asserts identical classification from both paths on the same change
(invariant 4).

### Phase 2 — declarations that sign the change

Seam declarations carry signed **bindings** (seam format 2): subject
symbol, before/after content hashes, base revision. The signed subject
is the transition, not the resulting state (ruling 1) — a declaration
reused against a different change landing on identical content does not
bind, tested at both the CLI gate and the interceptor. A dirty parent
state refuses to bind, naming the constraint (ruling 2; the refusal
message cites it). Binding hashes ride the ledger's canonical-JSON law
through its now-public primitives under `ddd.seam-binding.v1` — one
canonicalisation scheme, per-payload field sets (ruling 7).

Session matching is **removed, not layered** (ruling 4): the serve
process keeps no session log; enforce-mode matching is
signature-only against stored declarations; warn mode links generously
but discharges nothing. The rejection demand pre-fills the exact
binding to copy back (`dec/ddd/rejection-facts-prefilled`, extended).
In CI, discharge is judged by **chain composition**: a file's aggregate
transition is discharged when signed hops compose from its before-hash
to its after-hash and a hop names the event's symbol; a declaration
that does not bind is a finding, not a warning.

### Phase 3 — the ledger migration

79 entries filed into the `ddd-governance` set (namespace `hafeok.ddd`,
floor T1, ground characterised, owner the principal): 44 decisions, the
risk-acceptance record (`risk/ddd/undeclared-what-boundaries`, migrated
as the priced escape it states — exposure, acceptor of record, review
date 2027-02-07), 33 seam declarations (criterion, discharged by the
new `contract:` scheme at stage `pr`), and the filed watched-edge
ontology question. Manifest entries were absorbed as `analyzer:`
discharges on their governing decisions (see §3, deviation 3).

- **Every entry is allocated-awaiting-acceptance. No acceptance was
  filed** (ruling 8) — `ledger verify` is conformant with 79 pending.
- **Basis pins upgraded to content hashes** (ruling 7, invariant 2):
  `claim:<id>@sha256:…` under `ddd.claim-content.v1`. Unmoved claims
  hash at current content; the three deliberately-kept drifted edges
  recovered their pinned state from git history — never guessed — and
  keep reporting as drift, now exactly.
- **The three watched-not-grounding edges** (`DDD-adapter-02` on
  `dec/ddd/internal-not-surface`, `DDD-gates-01` on `dec/rust/no-unwrap`,
  `DDD-adapter-01` on `dec/ddd/m6-proceeds-no-flip`) migrate as
  `watched:` tokens — explicitly not ground — and the ontology question
  is filed as `question/ddd/watched-edge-kind` (ledger entry, judgment
  allocation, awaiting the ruling).
- **`dec/ddd/interceptor-not-extension` migrated as-is** at format 2,
  its claim edge carried as `indeterminate:DDD-arch-03@…` — marked, not
  typed, not laundered into a claim basis. **It awaits the principal's
  ruling** (audit F-7h).
- **The permanent concordance** (`.ddd/concordance.yaml`, ruling 6) maps
  every historical id both ways; `ddd why` resolves either spelling and
  shows the other. It carries the two ids created after it was scoped:
  the risk record (migrated, mapped) and `DDD-method-07` (**stance:
  remains a `.ddd` claim — claims are the ddd ontology's own and do not
  migrate**; the row states this explicitly).
- The `contract:` discharge scheme entered the ledger format through
  the ledger's own amendment procedure at its current version (ruling
  5): spec v1.4 / `format: 3`, migration note filed, L6 renumbered to
  spec v1.5 / `format: 4` — the same renumbering precedent L3 set.
  Hashing untouched; `CANONICAL_FORM` stays `v1`.

### Phase 4 — the authoritative gate

Readiness and completeness are the ledger's verdicts: `ddd report
escapes` and `ddd render` delegate to `ledger verify` and restate the
result (gates, pendency, disposition coverage) — no second rulebook.
The exact pin check runs there too: every `claim:`/`watched:`/
`indeterminate:` token on a latest version compared against the claim's
current content hash. CI gained `ddd validate` on every build and
`ddd diff-contracts <base>..HEAD` on every pull request (and the CI
path filters now include the ddd crates and `.ddd/` — previously CI did
not trigger on them at all). Invariant rows 2–5 flipped on evidence;
row 5 carries its named residual (a direct push to the default branch
sits outside the pull-request gate; branch protection closes that door,
and it is a forge setting, not a repo artifact).

### Phase 5 — the measurement

See §5.

---

## 2. The acceptance criteria, checked

| Criterion | Where it is proven |
|---|---|
| Bypass-catch, end to end: an editor edit committed with git, never touching MCP, produces a CI finding | `ddd-cli/tests/contracts.rs::an_out_of_band_contract_change_is_a_ci_finding`; the CI workflow step; and the gate caught its own construction — the M8 diff raised 154 contract-surface events that had to be discharged before its own gate would pass (§4) |
| A declaration binds to a specific transition; reuse against a different change landing on identical content is refused | `contracts.rs::a_binding_is_refused_against_a_different_change_landing_on_identical_content`; `ddd-mcp/tests/serve.rs::m8_a_binding_does_not_discharge_a_different_transition` |
| Dirty-tree binding refused with a message naming the constraint | `serve.rs::m8_dirty_parent_state_refuses_to_bind` ("never binds uncommitted parent state … M8 ruling 2") |
| Every historical id resolves through the concordance; pins are exact hash comparisons; migrated entries pending; both gates conformant | `ddd-cli/tests/migrate.rs`; `ddd validate` (192 entries) + `ledger verify` (conformant, 79 awaiting) on this tree |
| One classifier: diff path ≡ edit path | `ddd-lsp/src/revdiff_tests.rs::diff_path_matches_edit_path_on_the_same_change` |
| Report | this document |

---

## 3. Deviations, reported not reconciled

1. **The `.ddd/decisions/` files were not deleted.** "Bootstrap form
   retired" is implemented as *retired as the record*: the ledger entry
   is the decision's record of note (identity, versioning, acceptance
   state); the historical file remains as the pinned content artifact —
   rationale, principal, typed bases — sealed by a `ddd-content:` hash
   on the ledger version. An edit to the file breaks the pin and
   surfaces as drift. This keeps decision-time records byte-identical,
   keeps `ddd validate`/`why`/`render` whole, and gives acceptance real
   content to sign. If the ruling intended physical removal, that is a
   follow-up act on top of this state, not a rework.
2. **Pre-format-5 in-file pins keep their status+date form.** Bumping
   the 38 format-2 decisions to format 6 would force typing every basis
   — a re-decision only the principal can make (the format-5 migration
   note says exactly this). Their exact content-hash pins live on the
   ledger entries instead; decision format 6 exists for new decisions.
3. **Manifest entries became discharges, not sibling entries.** A
   manifest entry is the rule→decision join; migrating it as its own
   ledger decision would file the same ruling twice. Its ledger form is
   the `analyzer:<ns>/<rule>` DischargeRef on its governing decision —
   which is precisely OD-2's "their checkers are already DischargeRef
   types". The concordance still carries one row per manifest entry.
4. **Allocations were derived, not ruled per entry:** manifest-cited
   decisions → `constraint` + their analyzer refs; the risk record →
   `escaped`; everything else → `judgment` with the principal as actor;
   seams → `criterion` + `contract:` at stage `pr`. All at floor T1, no
   overrides. If any entry deserves T2 or a different allocation, the
   acceptance pass is the place to say so — each is one `revise` away.
5. **The escape's `accepted_by` transcribes the historical record**
   (the ratified risk-acceptance of 2026-08-07, required by `L002`). It
   is not a new acceptance and no `acceptances:` entry exists anywhere
   in the migration.
6. **`ddd bind` exists** — the post-hoc remedy verb CI failure demands
   (one declaration per file, signed bindings over the range's committed
   transitions). It fills facts, never judgment: fresh declarations file
   with empty `verdict_knowledge` and a warning until authored. The
   trade is stated: bulk retrofit is possible, but every declaration
   lands in the PR diff for review.
7. **The post-hoc binding path verifies against the named committed
   revision** rather than requiring a clean tree (the edit-time path
   requires both). It structurally cannot bind uncommitted state —
   `before` must equal a committed blob's hash — which is the ruling's
   substance; the difference is stated rather than hidden.
8. **`created_by` on migrated change-sets is the session's model
   identity.** The ledger permits a model author and refuses a model
   acceptor (`L006`/`L009`/`L010`) — authorship is honest, acceptance
   stays the principal's.

---

## 4. The friction delta from retiring session matching

Expected and observed:

- **Per governed edit:** one extra copy-back — the rejection demand now
  pre-fills the exact binding and the declaration must carry it. In the
  updated interception tests this is one argument passed through per
  declare call. No re-derivation: facts stay machine-authored.
- **Per-file serialization through commits:** a second governed edit to
  the same file refuses to bind until the first is committed (parent
  state must be committed — ruling 2). Multi-edit sessions on one file
  become edit→commit→edit. This is the largest behavioural change for
  agents on the governed path.
- **Formatting between signed hops breaks the chain** (the aggregate
  before→after pair no longer composes) and demands a fresh
  declaration. Strict by design: a declaration discharges the
  transition it names, nothing adjacent.
- **Measured on this milestone itself:** the M8 implementation diff
  raised **154 undischarged contract-surface events** at the phase-4
  mark (FINAL_BIND_PLACEHOLDER by the end) — every one had to be
  discharged by a signed declaration before the milestone's own CI gate
  would pass. Under session matching these would have been silently
  dischargeable by any same-session declaration touching the symbol;
  under signing they are reviewable per-file declarations with exact
  hashes.
- **Restart survival is the win purchased:** declarations now outlive
  the serve process, so the friction buys durable discharge — the
  correspondence rows link to declarations that still exist.

---

## 5. The classifier reading

The hand-labelled corpus (research protocol §3) exists under
`ddd-cli/tests/corpus/` with its measurement harness
(`cargo test -p ddd-cli --test corpus`) and reading
([`audits/classifier-corpus-2026-08.md`](audits/classifier-corpus-2026-08.md)).

CORPUS_NUMBERS_PLACEHOLDER

Labelling provenance is disclosed in the reading: labels were produced
by the implementing session against fixed policy-row semantics;
independent labelling is future work. C# and Bicep cases are the scoped
follow-up — their hosts are not available in this measurement
environment, and shipping numbers measured against the mock would be
measuring the mock.

---

## 6. The re-acceptance queue, as it now stands

`ledger verify` lists **79 entries allocated, awaiting acceptance** in
`ddd-governance` — the bounded cost the principal took (ruling 8):

- **44 decisions** (judgment or constraint per §3 item 4), including the
  seven re-typed ones with their honest typed bases carried in pinned
  content, and `dec/ddd/interceptor-not-extension` **flagged: still
  indeterminate, awaiting the F-7h ruling**;
- **1 priced escape** — the risk record, review by 2027-02-07;
- **34 seam-family entries**: 33 migrated declarations plus the filed
  ontology question (`question/ddd/watched-edge-kind`) — **the second
  item awaiting an explicit ruling**: does the watched-not-grounding
  edge become a distinct ontology kind, or retire at the F-batch?

Acceptance mechanics, for the pass: `ledger accept dec:hafeok.ddd/<ulid>`
per entry (identity from git config; `L009` requires the acceptance's
committer to be the acceptor — the principal commits their own
acceptances). `ddd why <historical-id>` shows each entry's ledger
identity; `ledger status` lists the queue.

Also awaiting rulings, carried from the audit and not consumed by this
milestone: the F-batch (F-1..F-6 falsifier repairs), and the
`DDD-method-06` status move proposed by the audit (§7 there).

---

## 7. What did not happen (anti-goals, confirmed)

No second classifier (invariant 4 tested); no second canonicalisation
(the ledger's primitives are the one law; every new payload is an
explicit field set under its own domain prefix); no MCP-routed gate; no
session-matching fallback; no auto-conversion of ratifications; no
acceptances filed by the session under any framing (asserted in the
migration test: the migration writes no `acceptances:` anywhere); no
invariant row flipped on narrative — row 5 carries its residual in the
table itself; no L3/L4, workbench, or org-track work; adapter changes
limited to what the diff path required (one `Host::open_with_text`
method; policy tables untouched).
