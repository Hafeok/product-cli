---
id: ADR-009
title: CI Integration via Exit Codes
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Product should be usable as a CI gate — a step in a pull request pipeline that fails the build if the knowledge graph has broken links, orphaned artifacts, or missing test criteria. This requires a consistent, machine-readable signal from the CLI.

**Decision:** Product uses a three-tier exit code scheme for graph health commands:
- `0` — clean graph, no issues
- `1` — errors (broken links, supersession cycles, malformed front-matter)
- `2` — warnings only (orphaned artifacts, features without exit criteria, untested features)

All other commands exit `0` on success and `1` on any error.

**Rationale:**
- The two-level error/warning distinction allows CI pipelines to fail on broken links (hard errors) while optionally warning on coverage gaps without blocking the build
- The convention follows `grep` (0 = found, 1 = not found, 2 = error) and lint tools like `clippy` — engineers arrive with prior knowledge of this pattern
- A CI pipeline can choose its tolerance: `product graph check` (fail on errors and warnings) or `product graph check || [ $? -eq 2 ]` (fail on errors only)
- Shell-friendly: the exit code is testable without parsing stdout

**Rejected alternatives:**
- **Single exit code (0/1)** — simpler but loses the error/warning distinction. Teams that want to tolerate coverage gaps but not broken links cannot express this policy.
- **Structured JSON output to stdout, always exit 0** — requires the CI step to parse output and apply its own logic. Adds friction for common cases that exit codes handle natively.