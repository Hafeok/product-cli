# DDD Specification Platform — umbrella

**Working name:** `ddd` (final name open)
**Author:** Emil, Context&
**Status:** Umbrella index; the implementable contract lives in the
documents below (split per `dec/ddd/prd-split`, 2026-08)

## Thesis

A CLI and MCP server that make a repository's governing decisions
explicit, versioned, and checkable. The graph under `.ddd/` stores
predicates, closure claims, decisions, analyzer/linter manifests, pattern
declarations, and seam declarations; the tool checks the code against
them. **Curation, not mining: the graph is the source of truth and the
code must conform to it.**

The platform is the seam between Context& engagements: determinations
resolved once are filed as graph entries and inherited by subsequent
projects. Prompt rules are exhortation an agent can drift past; a tool in
the edit loop is a policy-level commitment — and its honest boundary is
stated in the spec: interception governs the governed path; CI governs
the repository.

## The documents

| Document | Owns |
|---|---|
| [`ddd-v1-spec.md`](ddd-v1-spec.md) | The implementable contract for what exists (M1–M7 as built): invariants, architecture, adapter capabilities, graph store, commands, interception semantics, operational requirements, success criteria |
| [`ddd-adrs.md`](ddd-adrs.md) | The settled architecture decisions, each citing its graph entry |
| [`ddd-research-protocol.md`](ddd-research-protocol.md) | The correspondence dataset, the adapter-cost experiment (predicted/observed separated), the classifier corpus |
| [`ddd-roadmap.md`](ddd-roadmap.md) | M8 (enforcement closure) and the filed follow-ups |
| [`reviews/ddd-cli-prd-review-2026-08.md`](reviews/ddd-cli-prd-review-2026-08.md) | The review that forced the split — the checker event, filed in the graph (`DDD-arch-08`, `DDD-arch-09`, `DDD-method-06`) |

## Ontology and companion references

- Predicates and claims: `predicate-format.md`, claim formats, migration
  record in [`ddd-format-migrations.md`](ddd-format-migrations.md)
- Process layer: [`way-of-working-decision-allocated-delivery.md`](way-of-working-decision-allocated-delivery.md)
  (DAD — what an engagement runs; this tool is its enforcement substrate)
- Record substrate: [`decision-ledger-prd.md`](decision-ledger-prd.md)
  (canonical for the decision record; `ledger-format-v1.md` is what an
  outside implementation imports). The M8 migration (2026-08) made the
  ledger the record of `.ddd` governance: the `.ddd` files remain as
  pinned content artifacts, and `.ddd/concordance.yaml` maps the ids
  permanently.

## History note

M1 (graph store, `validate`, `why`), M2 (SARIF diff, `report escapes`,
basis pins), M2.5 (`render`), M3 (MCP language tools over LSP hosts), M4
(the `apply_edit` interceptor), M5 (dogfood curation — the DAD checklists
seeded ~70 claims), M6 (Rust adapter; the adapter-cost experiment), and
M7 (the HTML+CSS pair; the experiment's falsifier) shipped 2026-08,
each with its report or graph record. The 2026-08 review found the PRD
overstating the enforcement boundary and conflating five documents; the
three resulting rulings (typed basis, M8 enforcement closure, this split)
are filed as `dec/ddd/typed-basis`, `dec/ddd/m8-enforcement-closure`,
`dec/ddd/prd-split`. This umbrella introduces nothing; every normative
statement lives in exactly one of the documents above.

**Section concordance for historical references.** Code comments and
graph entries citing "PRD §N" cite the pre-split PRD's numbering, kept
stable here: §5 architecture → spec §3; §6 graph store and ontology
rules (rule 6 = basis pins) → spec §5; §7 command surface and finding
taxonomy → spec §6; §8 MCP surface and interception semantics → spec §7;
§9 adapters → spec §4; §13 decisions embodied → the ADRs; §14 open
questions → spec §10 and the ADRs' settled table.
