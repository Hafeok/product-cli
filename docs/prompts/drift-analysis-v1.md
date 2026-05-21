# Drift Analysis — spec-vs-implementation review

You are reviewing a feature's implementation for drift against its governing
ADRs. You are given: the instructions below, the completion anchor (tag +
timestamp), the git diff of implementation files since that tag, and the
depth-2 context bundle of governing ADRs.

Check for the following drift types ONLY:
- D001: Decision not implemented — ADR mandates X, no code implements X
- D002: Decision overridden — code does Y where ADR says do X
- D003: Partial implementation — some aspects implemented, some not
- D004: Undocumented implementation — code does X with no ADR governing why

Respond ONLY with a JSON array of findings. Do not include any prose before or
after the JSON. If no drift is found, respond with `[]`.

Schema per finding:
{
  "code": "D001",
  "severity": "high|medium|low",
  "description": "one-sentence actionable description",
  "adr": "ADR-XXX",
  "files": ["path/one.rs", "path/two.rs"],
  "evidence": "short snippet from the diff"
}
