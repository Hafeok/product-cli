# {{DISPLAY_NAME}} — a ground registry

The canonical store of {{OWNER_ORG}}'s ratified ground: entities, relations, axes, trust
decisions. This repository is the **authority**; every SPARQL endpoint built from it is a
projection pinned to a ref, never a source. The design is the G-track PRD's §3
(`product-cli/docs/g-track/prd-ground-as-ontology.md`) — this page states the mechanics and cites
it rather than restating it. This instance was generated from the registry template; its birth
provenance (template version, parameters, generated-by, date) is `GENERATION.ttl`.

## Structure

| Path | Holds |
|---|---|
| `graphs/<graph>/` | one named graph per directory; **one assertion per file, in Turtle** (the per-claim file pattern applied to triples — the pattern transfers: per-assertion files, PR review, supersession; G-1 Gate 3 ruled the format) |
| `graphs/canonical/` | the ratified graph |
| `graphs/canonical/founding-decision.ttl` | **the founding-decision slot** — a path, not a shipped file: {{OWNER_ORG}}'s decision to keep a registry, filed by {{RATIFIER}} as the first ratified content, creates it. The form is `shapes/decision.ttl`, with a conforming example in `graphs/canonical/_exemplar-decision.ttl` |
| `shapes/` | SHACL shapes; CI runs them on every change |
| `scripts/build-projection.sh` | ref in, store build out (stub until G0) |
| `GENERATION.ttl` | this instance's birth provenance |

There is no `proposals/` directory, deliberately. **Proposals are branches, not directories**: a
proposal graph (an extraction run, a session's proposed Readings) lands as per-triple files on a
branch, named-graph directory included, `prov:wasAttributedTo` its producer and pinned to its
source ref.

## Ratification

1. A proposal branch is opened; its files are the proposed triples, nothing else.
2. Review is **per triple, never per run** — each file is a line in the PR diff, accepted or
   rejected individually (PRD §5.1, §5.6: wholesale acceptance is manufactured ground).
3. **{{RATIFIER}} merges.** The merge is the ratification act; supersession is a new file plus a
   retraction marker, never a rewrite of history.
4. The endpoint rebuilds from the merge ref (`scripts/build-projection.sh`); every Reading it
   serves carries that ref (PRD §4.1 — as-of semantics for free).

## Validation

CI runs SHACL on every change (`.github/workflows/validate.yml`). The shapes start minimal and
grow at G0:

- **Reading tuple** (`shapes/reading.ttl`): the PRD §4.1 constraint — a Reading carries value,
  as-of, provenance, assurance; `institutional` provenance **requires** a `trust_decision`
  reference; the provenance value set is track vocabulary per G-track decision `g-dec-01`
  (superseded the moment canon files provenance typing).
- **Structural well-formedness** (`shapes/structural.ttl`): every data file parses; an assertion
  names exactly one subject, predicate, object.
- **The decision shape** (`shapes/decision.ttl`): a decision carries title, resolution, region,
  falsifier, ratifier, status, date. `basis`, `acceptedCost` and `revisitIf` are deliberately not
  required — a decision may honestly have none, and demanding them produces filler.
- **One assertion or one decision per file**, across every `graphs/**/*.ttl`. One rule, no filename
  exemptions.

## Ownership

Ownership is a **generation parameter** of this instance: {{OWNER_ORG}} owns this registry, and
that ownership is the accountable-principal field of every trust decision that references it
(PRD §4.4 — connecting is a trust decision, Q27).

The base IRI is the one parameter with no supersession path: a changed IRI orphans identifiers
rather than superseding them. It was settled at generation by one of the two routes the template
documents — a host {{OWNER_ORG}} controls durably, or location-independent minting (`tag:`/`urn:`)
with the resolvable HTTP form produced later as a projection. Which route this instance took, and
why, is a decision filed in this registry, not a fact recoverable from the IRI alone.
