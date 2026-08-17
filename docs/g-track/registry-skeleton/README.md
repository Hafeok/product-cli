# The ground registry

The canonical store of the organisation's ratified ground: entities, relations, axes, trust
decisions. This repository is the **authority**; every SPARQL endpoint built from it is a
projection pinned to a ref, never a source. The design is the G-track PRD's §3
(`product-cli/docs/g-track/prd-ground-as-ontology.md`) — this page states the mechanics and cites
it rather than restating it.

## Structure

| Path | Holds |
|---|---|
| `graphs/<graph>/` | one named graph per directory; **one assertion per file** (the per-claim file pattern applied to triples — PRD §3, authority row) |
| `graphs/canonical/` | the ratified graph |
| `shapes/` | SHACL shapes; CI runs them on every change |
| `scripts/build-projection.sh` | ref in, store build out (stub until G0) |

There is no `proposals/` directory, deliberately. **Proposals are branches, not directories**: a
proposal graph (an extraction run, a session's proposed Readings) lands as per-triple files on a
branch, named-graph directory included, `prov:wasAttributedTo` its producer and pinned to its
source ref.

## Ratification

1. A proposal branch is opened; its files are the proposed triples, nothing else.
2. Review is **per triple, never per run** — each file is a line in the PR diff, accepted or
   rejected individually (PRD §5.1, §5.6: wholesale acceptance is manufactured ground).
3. **Emil merges.** The merge is the ratification act; supersession is a new file plus a
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
- **Structural well-formedness**: every data file parses, and carries exactly one assertion.

## Ownership

**OPEN — resolved at G-1 Gate 3.** The repository initialises under the org Emil names at the
gate. That naming answers the PRD's open item 7: the owning organisation is the
accountable-principal field of every trust decision that references this registry (PRD §4.4 —
connecting is a trust decision, Q27).
