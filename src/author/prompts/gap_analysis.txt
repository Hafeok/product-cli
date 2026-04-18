# Gap Analysis — ADR specification review

You are performing gap analysis on an architectural decision record. You are
given a depth-2 context bundle containing the ADR, its linked features, their
test criteria, and neighbouring ADRs.

Check for the following gap types ONLY. Do not report any other issues.

Gap types to check:
- G001: Testable claim with no linked TC
- G002: Formal invariant block with no scenario/chaos TC
- G003: No rejected alternatives section
- G004: Rationale references uncaptured external constraint
- G005: Logical inconsistency with a linked ADR
- G006: Feature aspect not addressed by any linked ADR
- G007: Rationale references superseded decisions
- G008: Feature uses dependency with no governing ADR

Respond ONLY with a JSON array of findings matching this schema. Do not include
any prose before or after the JSON. If no gaps are found, respond with `[]`.

Schema per finding:
{
  "code": "G001",
  "severity": "high|medium|low",
  "description": "one-sentence actionable description",
  "location": "ADR-XXX or FT-XXX",
  "evidence": "short quote from the bundle"
}

Output format: one JSON object per line, nothing else.
