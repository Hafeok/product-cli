---
id: ADR-019
title: Continuous Gap Analysis — LLM-Driven Specification Review in CI
status: accepted
features: []
supersedes: []
superseded-by: []
domains: [api, observability]
scope: domain
content-hash: sha256:f1d5b706ef1879b88c04acfe832052e5b44eb01f4488f387c1fdf8912fbb28c4
---

**Status:** Accepted

**Context:** The existing testing strategy (ADR-018) validates that Product works correctly and that context bundles improve LLM implementation quality. It does not validate that the *specifications themselves* are complete and internally consistent. A repository can have zero structural errors (`product graph check` exits 0), passing property tests, and passing integration tests — and still contain ADRs that make untested claims, reference undocumented constraints, or contradict each other. These specification gaps are invisible to structural tooling because they require semantic understanding of the content.

An LLM is precisely the right tool for semantic review of structured documents. The gap analysis capability runs an LLM against each ADR's full context bundle and asks it to identify specific, enumerated gap types. The result is a structured set of findings that can be tracked over time, baselined, suppressed, and resolved — giving gap analysis the same CI lifecycle as static analysis.

The key design constraint is CI reliability. LLM output is non-deterministic. A gap analysis that produces different results on two identical repository states would make CI unstable and unusable. This ADR specifies the three mechanisms that make gap analysis deterministic enough for CI: structured output schema, temperature=0, and run-twice intersection for high-severity findings.

**Decision:** Implement `product gap check` as a continuous LLM-driven specification review command. It analyses ADRs using depth-2 context bundles, checks for eight defined gap types (G001–G008), produces deterministic structured findings, and integrates with a `gaps.json` baseline for suppression and resolution tracking. The `--changed` flag scopes CI analysis to the affected ADR subgraph.

---

### Gap Type Specification

Seven gap types are checked. The set is fixed — the prompt is instructed to check only these types and ignore general quality issues. This constraint is critical for determinism: an open-ended "find any problems" prompt produces unbounded, incomparable findings across runs.

| Code | Severity | Trigger condition |
|---|---|---|
| G001 | high | ADR body contains a testable claim (performance threshold, behavioural invariant, correctness property) with no linked TC that exercises it |
| G002 | high | A `⟦Γ:Invariants⟧` formal block is present but no linked TC of type `scenario` or `chaos` references the ADR and addresses the invariant |
| G003 | medium | ADR has no **Rejected alternatives** section, or the section is empty |
| G004 | medium | ADR rationale references an external constraint, library behaviour, or assumption not captured in any linked artifact (feature, ADR, or TC) |
| G005 | high | This ADR makes a claim that is logically inconsistent with a claim in a linked ADR (shared feature or within depth-2 context) |
| G006 | medium | A feature linked to this ADR has aspects — stated in the feature body — not addressed by any of the feature's linked ADRs |
| G007 | low | ADR rationale references decisions, constraints, or rationale that have been superseded by a more recent ADR in the context bundle |

---

### Context Bundle for Gap Analysis

Gap analysis uses `product context ADR-XXX --depth 2`. This produces:

- The ADR under analysis
- All features linked to it
- All test criteria linked to those features
- All other ADRs linked to those features (the 1-hop ADR neighbourhood)
- Their test criteria

This is the same bundle an implementation agent would receive. Gap analysis validates specification completeness from the agent's perspective — if the agent cannot find the information it needs in this bundle, that is a gap.

---

### Prompt Specification

The gap analysis prompt is versioned and stored at `benchmarks/prompts/gap-analysis-v{N}.md`. The version is referenced in `product.toml` under `[gap-analysis]`. Changing the prompt increments the version — findings from different prompt versions are not comparable and must not be merged in `gaps.json`.

**Prompt version upgrade protocol:** When `prompt-version` is incremented in `product.toml`, `gaps.json` suppressions from the previous version are retained but tagged with the version they were created under. On the first run with the new prompt version, all existing suppressions are treated as provisional — they are not cleared, but `product gap check` emits W-class warnings for each one: "suppression GAP-ADR002-G001-a3f9 was created under prompt-version 1, re-verify with prompt-version 2." The developer reviews and either re-confirms (`product gap suppress --re-confirm`) or removes the suppression. This prevents stale suppressions from masking gaps that the new prompt detects differently.

The prompt structure:

```markdown
You are reviewing an architectural specification for completeness and consistency.
You will be given a context bundle containing an ADR and related artifacts.

Check ONLY for the following gap types. Do not report any other issues.

[Gap type table with codes, descriptions, and examples]

Respond ONLY with a JSON array of findings matching this schema exactly.
Do not include any prose before or after the JSON.
If you find no gaps, respond with an empty array: []

Schema:
{
  "id": "GAP-{ADR_ID}-{CODE}-{HASH}",   // deterministic ID per finding
  "code": "G001",
  "severity": "high",
  "description": "...",                  // one sentence, specific and actionable
  "affected_artifacts": ["ADR-002"],
  "suggested_action": "...",             // one sentence
  "evidence": "..."                      // quote from the context bundle that triggered this
}

Context bundle:
{BUNDLE}
```

The `evidence` field is required for G005 (contradiction) — it must quote the specific conflicting claim from each ADR. This forces the model to ground its finding in actual text rather than hallucinating a contradiction.

---

### Gap ID Derivation

Gap IDs are deterministic. The short hash is derived from: `sha256(adr_id + gap_code + sorted(affected_artifact_ids) + finding_description)[0:4]`. Same logical finding → same ID across runs. This is what makes suppression stable.

```rust
fn gap_id(adr_id: &str, code: &str, artifacts: &[&str], description: &str) -> String {
    let mut sorted = artifacts.to_vec();
    sorted.sort();
    let input = format!("{}{}{}{}", adr_id, code, sorted.join(","), description);
    let hash = &sha256(input.as_bytes())[..4];
    format!("GAP-{}-{}-{}", adr_id, code, hex(hash))
}
```

In practice, the description field introduces some variance (the model may phrase the same gap differently between runs). The run-twice intersection (for high severity) handles this — two runs that describe the same gap in slightly different words will produce different IDs and thus not intersect, suppressing the finding. This is conservative: it means some real gaps may not be reported. The alternative (fuzzy matching on descriptions) introduces its own instability. Conservative is correct for CI.

---

### `--changed` Scoping

In CI, gap analysis must not run against every ADR on every commit. The scoping algorithm:

1. `git diff --name-only HEAD~1` to identify changed files
2. Filter to files under the `adrs/` directory matching the ADR prefix
3. For each changed ADR, traverse the reverse graph to find all features it is linked to
4. For each of those features, traverse forward to find all other ADRs linked to that feature
5. The analysis set = changed ADRs ∪ their 1-hop ADR neighbours

This scoping ensures that a change to ADR-002 also analyses ADR-005 if they share a feature — because the change to ADR-002 may now contradict ADR-005 (G005). Without this expansion, G005 would never be caught by `--changed` mode.

The analysis set is bounded: `|changed_adrs| × |avg_adr_neighbours|`. For a well-structured repository with average ADR fan-out of 3, a PR changing 2 ADRs analyses at most ~8 ADRs. At ~10 seconds per ADR analysis, CI time is proportional and predictable.

---

### `gaps.json` Lifecycle

```
Initial state:   gaps.json does not exist → all findings are new
First run:       findings reported, developer suppresses known/expected ones
Subsequent runs: new findings (not in suppressions) → exit code 1
                 suppressed findings → exit code 0 (logged as informational)
                 resolved findings (were suppressed, now not detected) → logged, moved to resolved
```

`gaps.json` is committed to the repository. It is the shared team baseline. A suppression added by one developer is respected by all CI runs and all teammates.

`product gap suppress` mutates `gaps.json` atomically (ADR-015). The suppression records the gap ID, the reason, the suppressing commit, and the timestamp. This creates an audit trail of deliberate decisions to accept known gaps.

---

### Handling Model Errors

If the model call fails (network error, timeout, invalid JSON response), `product gap check` reports the error on stderr and exits 2 (analysis warning). It does not exit 1 (new gaps found). A transient model error never fails CI — it only produces a warning that the analysis was incomplete.

If the model returns JSON that does not match the finding schema, the malformed findings are discarded individually. Valid findings from the same response are retained. A log line on stderr records each discarded finding.

This asymmetry (model errors are warnings, not failures) is intentional. CI is not the right place to retry flaky API calls. The operator can re-run the analysis manually, or the next commit will trigger another run.

---

**Rationale:**
- Gap analysis belongs in CI, not as an ad-hoc command, because specification gaps compound over time. An unchecked gap in ADR-002 today becomes a missing test, then a misunderstood invariant, then a production bug six months later. Continuous analysis catches gaps when they are introduced, not when they manifest.
- The fixed set of seven gap types is essential for CI reliability. An open-ended prompt produces incomparable results across runs and across prompt versions. Enumerating the gap types converts an unbounded quality review into a bounded, checkable specification.
- Run-twice intersection for high-severity findings is a conservative but correct approach to the non-determinism problem. The alternative — accepting any single-run finding — produces false positives that pollute `gaps.json` with hallucinated contradictions. The cost is that some real gaps require two consistent runs to surface; the benefit is that CI never fails on a hallucination.
- `--changed` scoping with 1-hop expansion is the only CI-viable approach. Full-repository analysis on every commit is prohibitively expensive. Analysing only changed ADRs without expansion misses cross-ADR contradictions introduced by the change. The 1-hop expansion is the minimum scope that catches G005.
- `gaps.json` suppression follows the `cargo audit` model because it is already well-understood by the Rust community Product targets. Operators know how to work with it: audit, suppress known issues, fail on new ones.

**Rejected alternatives:**
- **PR comments only, no CI gate** — analysis results are informational only, no build failure. Rejected because informational findings are routinely ignored. A CI gate is the only mechanism that ensures gaps are addressed.
- **Run on every ADR every commit** — complete coverage, no scoping complexity. Rejected on cost and latency grounds. A repository with 30 ADRs at 10s per analysis adds 5 minutes to every CI run. At $0.01/1K tokens per ADR analysis, it adds non-trivial cost per commit.
- **Semantic similarity for gap ID matching** — use embedding similarity to match a suppressed gap to a re-detected gap even if the description changed. Rejected because embedding similarity requires a model call, adds complexity, and the threshold is tunable in a way that creates fragility. Exact hash matching is brittle but predictable.
- **Store findings in artifact front-matter** — gap findings annotate the ADR file directly rather than a separate `gaps.json`. Rejected because it creates noise in git history (every CI run would potentially produce a commit), contaminates the ADR content with tooling metadata, and makes it impossible to suppress a finding without modifying the artifact under analysis.