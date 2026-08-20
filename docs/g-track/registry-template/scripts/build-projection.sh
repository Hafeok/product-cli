#!/usr/bin/env bash
# Projection build — takes a ref, emits a store build. STUB until G0.
#
# The endpoint is a projection of this repository, never the source (PRD §3,
# Q30's diagnostic applied to our own database). Every Reading the endpoint
# serves carries the ref this build was made from (PRD §4.1).
set -euo pipefail

REF="${1:?usage: build-projection.sh <ref> [out-dir]}"
OUT="${2:-build/${REF}}"

# TODO(G0), in order:
#   1. Materialise the tree at $REF (git worktree/archive — never the working tree).
#   2. Load graphs/<graph>/*.ttl into an Oxigraph store build at $OUT,
#      one named graph per directory; skip `_`-prefixed exemplars.
#   2b. Join the other two layers (template 0.3.0):
#      - runs/*.ttl — per-run extraction evidence, keyed by assertion id. The
#        authority repo keeps assertion and evidence apart; the projection is
#        where a consumer reads them together, so derivation is queryable here
#        without either layer borrowing the other's status.
#      - vocabulary/reg.ttl — the registry's own terms, including the
#        `reg:assuranceGrade owl:equivalentProperty reg:derivationConfidence`
#        axiom that lets one query answer over both generations of assertion.
#      The Rust form of this join is product_core::ground::project, whose
#      fixtures validate the projection rather than the repo alone: a shape
#      keyed on predicate presence passes pre-entailment and fails here.
#   3. Inference hook — ruled at G-1 Gate 2, recorded as g-dec-02:
#      CONSTRUCT-to-fixpoint on Oxigraph (the pf::sparql_rules shape; no new
#      dependency), `reasonable` crate as named fallback; revisit_if: fixpoint
#      wall-time exceeds the projection-build budget at G0 scale. Every entailed
#      triple carries prov:wasDerivedFrom (asserted triples + rule id) and
#      assurance = the rule. Entailments never enter this repository.
#   4. Stamp the build with $REF; the endpoint serves nothing unstamped.

echo "stub: projection build for ref '${REF}' -> '${OUT}' — not implemented before G0" >&2
exit 64
