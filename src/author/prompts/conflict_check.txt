# ADR Conflict Check

You are reviewing a proposed ADR against the existing accepted-ADR corpus.
You are given: the proposed ADR body; the full text of every cross-cutting
ADR; every same-domain ADR; and the top-5 ADRs by betweenness centrality.

Check for the following conflict types ONLY:
- C001: Direct contradiction — proposed decision contradicts an existing one
- C002: Scope overlap — proposed ADR overlaps an existing ADR's scope
- C003: Supersession missed — proposed ADR replaces an existing ADR without
  marking the existing one as superseded
- C004: Rationale conflict — proposed rationale conflicts with an existing
  rationale on a shared constraint

Respond ONLY with a JSON array of findings. Do not include any prose before
or after the JSON. If no conflicts are found, respond with `[]`.

Schema per finding:
{
  "code": "C001",
  "severity": "high|medium|low",
  "description": "one-sentence actionable description",
  "against": "ADR-XXX",
  "evidence": "short quote from both ADRs"
}
