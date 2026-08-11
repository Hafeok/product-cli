# DDD Architecture Decisions — the settled set

Extracted from the PRD's §13 and the `.ddd/` graph per `dec/ddd/prd-split`.
**This document records; it decides nothing.** Every entry cites its graph
id — `ddd why <id>` resolves the full chain (rationale, principal, typed
bases with pin state). Where a decision's basis has moved, the graph entry
carries the re-affirmation note; this document does not restate it.

## The load-bearing six

### 1. Workspace delivery — `dec/ddd/workspace-member-delivery`

Deliver `ddd` as workspace members of `product-cli` (`ddd-core` /
`ddd-lsp` / `ddd-mcp` / `ddd-cli`), reusing the store conventions, the
validation engine, and the MCP plumbing through public crate surfaces
only. The DDD ontology is its own namespace and store; it does not extend
What/How. Basis: `DDD-arch-01`.

### 2. Declarative validation on the shared SPARQL engine (SHACL-style) — `dec/ddd/workspace-member-delivery`, ontology rules in `ddd-core/src/rules.rs`

The §-6 ontology rules are declarative shapes — `sh:sparql`-style SELECTs
run by `product_core::pf::sparql_rules` over a Turtle projection —
rather than hand-rolled graph walks. A consequence of the reuse decision,
recorded separately because it is load-bearing: adding an ontology rule
is adding a query, not a traversal.

### 3. SARIF unification — `dec/ddd/sarif-unification`

One ingestion module; per-language invocation adapters know only how to
produce a SARIF file and which rule-id namespace it carries. Basis:
`DDD-detect-01` (both toolchains emit SARIF; one structured format beats
two console parsers).

### 4. LSP-protocol-only seam — `dec/ddd/lsp-as-seam`

The LSP protocol is the seam to language intelligence; the core never
touches Roslyn or Bicep APIs. Basis: `DDD-arch-02`. Corollary:
`dec/ddd/interceptor-not-extension` — seam detection is an edit-flow
interceptor consuming LSP events, not an LSP server extension.

### 5. Adapter capabilities — `dec/ddd/adapter-policy-tables`, split forced by `DDD-adapter-04`

Contract surface is defined per-language in adapter policy tables, and
tables are falsifiable claims (wrong rows die in the table, not the
core). The M6/M7 experiments split the adapter into capabilities: the
policy-table cost claim held for the classifier (`DDD-adapter-01`,
established at M7), while host lifecycle proved bespoke per server
(`DDD-adapter-04` — the evidence that forced the host / diagnostic /
config / classifier / detector decomposition in the spec §4), and
producer shapes are priced at the serve layer (`DDD-adapter-05`).

### 6. Curation, not mining — `dec/ddd/curation-over-mining`

Detection is conformance checking against declarations; the tool never
infers commitments from code. Basis: `DDD-method-01`/`DDD-method-02`
territory: the graph is the source of truth.

## The rest of the settled set

| Decision | Graph id |
|---|---|
| v1 scope: local repo, C# + Bicep, build-output detection first | `dec/ddd/v1-scope` |
| Predicates carry no status; enforcement structural | `dec/ddd/predicates-carry-no-status` |
| Basis pins land at M2, not deferred | `dec/ddd/pins-at-m2` |
| Tests run against committed SARIF fixtures; no .NET in workspace CI | `dec/ddd/fixtures-not-sdk` |
| `internal` is not contract surface by default (app-repo posture) | `dec/ddd/internal-not-surface` |
| Rejection returns facts pre-filled, judgment blank (PRD §14 q3) | `dec/ddd/rejection-facts-prefilled` |
| M5 seed lands in `claims/`, not `shared/` (PRD §14 q5) | `dec/ddd/seed-lands-in-claims-not-shared` |
| Enforce-mode matching tightens to the symbol | `dec/ddd/enforce-matching-tightens-to-symbol` |
| The tool's own Rust class is enforced here | `dec/ddd/rust-class-enforced-here` |
| Rust host is real (rust-analyzer child, not a stub) | `dec/ddd/rust-host-is-real` |
| M6 proceeds; the M6/M7 flip condition did not fire | `dec/ddd/m6-proceeds-no-flip` |
| HTML+CSS governed as a pair; plain-pair scope; enforce mode | `dec/web/plain-pair-scope`, `dec/web/htmlcss-enforce` |
| `why` resolves three ways (unfiled ≠ unknown) | `dec/ddd/why-resolves-three-ways` |
| What boundaries priced, not paid | `dec/ddd/what-boundaries-priced-not-paid` |
| The What policy table + published qualifier | `dec/ddd/what-policy-table`, `dec/ddd/what-published-qualifier` |
| A decision's basis is typed; non-claim bases first-class | `dec/ddd/typed-basis` |
| M8 expands to enforcement closure | `dec/ddd/m8-enforcement-closure` |
| The PRD splits | `dec/ddd/prd-split` |

The ledger relationship (record substrate, `dec:` migration at M8) is
owned by [`decision-ledger-prd.md`](decision-ledger-prd.md) OD-2 and the
roadmap; the process layer (DAD) by
[`way-of-working-decision-allocated-delivery.md`](way-of-working-decision-allocated-delivery.md).
