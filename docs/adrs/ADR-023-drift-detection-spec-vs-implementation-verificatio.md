---
id: ADR-023
title: Drift Detection — Spec vs. Implementation Verification
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Gap analysis (ADR-019) checks specification completeness. It validates that ADRs are internally consistent and well-covered by test criteria. It does not check whether the codebase matches the ADRs. An ADR can be complete, well-tested, and fully gap-free — and the implementation can still contradict it. This divergence is invisible to all current Product checks because they operate on the documentation graph, not the code.

Drift detection closes this gap by giving an LLM both the ADR context bundle and the relevant source files and asking it to identify where the code diverges from the decisions.

**Decision:** `product drift check` provides LLM-driven spec-vs-implementation verification. The LLM receives the ADR's depth-2 context bundle and the source files associated with it (resolved via configurable path patterns). It checks for four drift types (D001–D004). Findings follow the same baseline/suppression model as gap findings (`drift.json`). `product drift scan` reverses the direction: given a source path, identify which ADRs govern it.

---

### Source File Association

Product resolves source files for an ADR via two mechanisms:

**Pattern-based (configured):**
```toml
# product.toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20        # cap to keep context bundle size bounded
```

For each ADR, Product searches `source-roots` for files whose path or content contains the ADR's ID or any of its linked feature IDs. This is a heuristic — it will miss files with no explicit reference. The `--files` flag overrides for precision:

```bash
product drift check ADR-002 --files src/consensus/raft.rs src/consensus/leader.rs
```

**`source-files` in ADR front-matter (explicit):**
```yaml
---
id: ADR-002
source-files:
  - src/consensus/raft.rs
  - src/consensus/leader.rs
---
```

Explicit `source-files` in front-matter always override pattern-based discovery. This is the recommended approach for ADRs governing specific, known files.

---

### Drift Types

| Code | Severity | Description |
|---|---|---|
| D001 | high | Decision not implemented — ADR mandates X, no code implements X |
| D002 | high | Decision overridden — code does Y where ADR says do X |
| D003 | medium | Partial implementation — some aspects implemented, some not |
| D004 | low | Undocumented implementation — code does X with no ADR governing why |

D004 is the "code ahead of spec" case. It is a low-severity finding that suggests an ADR should be written, not that something is wrong.

---

### `product drift scan`

Reverse direction: given a source path, find the ADRs that govern it.

```
product drift scan src/consensus/raft.rs
  → ADR-002: openraft for cluster consensus
  → ADR-006: Oxigraph for RDF projection (via raft log → projection pipeline)
  → ADR-001: Rust as implementation language
```

The scan loads the file, asks the LLM to identify which ADRs from the full graph are relevant to this code, and returns them ranked by relevance. This is "ADR archaeology" — understanding the decisions behind an unfamiliar file without reading the entire spec.

---

### `drift.json` Baseline

Same structure as `gaps.json`. Suppressions reference `DRIFT-{ADR_ID}-{CODE}-{HASH}`. The same suppression lifecycle applies: new findings fail CI, suppressed findings pass, resolved findings are recorded.

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "DRIFT-ADR002-D003-f4a1",
      "reason": "Partial implementation is intentional — full openraft storage layer in phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ]
}
```

---

**Rationale:**
- Drift detection requires source code access, which makes it qualitatively different from gap analysis. Gap analysis operates entirely within the docs graph. Drift analysis crosses the docs/code boundary. This distinction justifies a separate command, separate finding codes, and a separate baseline file.
- `source-files` in ADR front-matter is the high-precision path. For ADRs governing specific subsystems (consensus, storage, IAM), the author knows exactly which files implement the decision. For cross-cutting ADRs (ADR-001 Rust), pattern-based discovery is appropriate.
- D004 (undocumented implementation) is valuable during active development phases when code is written faster than specs. It prompts the developer to write the ADR that should govern the code they just wrote. It is low severity — not a failure, a reminder.