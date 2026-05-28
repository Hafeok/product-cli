---
id: FT-030
title: Codebase Onboarding
phase: 5
status: complete
depends-on: []
adrs:
- ADR-022
- ADR-027
tests:
- TC-168
- TC-169
- TC-170
- TC-171
- TC-172
- TC-173
- TC-174
- TC-175
- TC-176
- TC-177
- TC-178
- TC-356
- TC-357
- TC-358
- TC-359
- TC-360
- TC-361
- TC-362
- TC-363
- TC-364
- TC-365
- TC-366
- TC-367
- TC-368
domains:
- api
- data-model
domains-acknowledged:
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
  api: The onboard command adds CLI subcommands but the API contract is fully specified in ADR-027 (transitive TC link inference) which is already linked. No separate API-domain ADR is required.
---

Codebase onboarding discovers load-bearing architectural decisions from an existing codebase and produces a minimum viable knowledge graph. See ADR-027 for the full specification.

### The Problem

Most codebases have no formal architecture documentation. The decisions are baked into patterns — error handling conventions, module boundaries, dependency choices — that were made over years but never written down. An agent (or new engineer) modifying this codebase has no way to know which patterns are load-bearing and which are incidental.

### The Three Failure Modes

Naive approaches to onboarding fail in predictable ways:

1. **The archaeology dump** — LLM scans the codebase, generates 40 ADRs with no rationale, no rejected alternatives, no evidence. The graph is populated but useless. *Avoided by: LLM proposes candidates, not ADRs. Human triage is required.*

2. **The perfectionism trap** — every ADR must be complete before proceeding. Onboarding takes six months. *Avoided by: the "enrich later" principle. Confirmed candidates with empty rationale are valid. Gap analysis drives incremental enrichment.*

3. **The wrong unit** — starting from directory structure produces ADRs that map to files, not decisions. *Avoided by: signal types that cross module boundaries by design.*

### The Three Phases

```bash
# Phase 1: Scan — LLM detects decision candidates from code patterns
product onboard scan ./src --output candidates.json

# Phase 2: Triage — team confirms, enriches, merges, or rejects
product onboard triage candidates.json --interactive

# Phase 3: Seed — confirmed candidates become ADR files + feature stubs
product onboard seed triaged.json

# Post-onboarding: gap analysis drives incremental growth
product gap check --all
```

### Signal Types

The scan prompt looks for six signal types — patterns that suggest deliberate architectural choices:

| Signal | What the LLM observes | Why it's load-bearing |
|---|---|---|
| **Consistency** | Same pattern repeated across the codebase | Violating it breaks an implicit contract |
| **Boundary** | Only certain modules access a resource | Violating it bypasses safety guarantees |
| **Constraint** | All X comes from Y, never from Z | Violating it breaks deployment/runtime assumptions |
| **Convention** | Different treatment for different categories | Violating it leaks internals or breaks APIs |
| **Absence** | Something is deliberately *not* used | Introducing it would conflict with the chosen approach |
| **Dependency** | A foundational dependency is pinned with explanation | Upgrading it would break an assumption |

None of these map to files or directories. They map to decisions that manifest *across* files.

### How It Differs from Migration (FT-020)

Migration converts **existing documents** (PRDs, ADR docs) into structured artifacts. The input is already prose that describes decisions.

Onboarding converts **existing code** into structured artifacts. The input is source files where decisions are implicit in patterns, not stated in prose. The LLM detects signals; the team provides meaning.

A team may use both: migrate existing docs with `product migrate`, then onboard the codebase with `product onboard` to find decisions that were never documented.

### Configuration

```toml
[onboard]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-candidates = 30             # upper bound — prevents archaeology dump
confidence-threshold = "low"    # include everything, let triage filter
chunk-strategy = "import-graph" # split large codebases by module clusters
evidence-validation = true      # post-validate cited files and lines exist
```

### Design Principles

1. **Find load-bearing walls, not everything.** The question is "what would break if an agent didn't know about it?" — not "document the architecture."
2. **LLM detects, humans decide.** The LLM proposes decision candidates with evidence. The team confirms and enriches. No ADR enters the graph without human triage.
3. **Good enough beats complete.** Confirmed candidates with empty rationale are valid. Gap analysis (FT-029) drives incremental enrichment. Onboarding that finishes in a day beats onboarding that takes six months.
4. **Evidence-grounded, not hallucinated.** Every candidate must cite specific files and line numbers. Post-validation catches fabricated evidence before triage.

### Exit Criterion

Onboarding is "done enough" when:

1. `product gap check --all` produces no G005 (architectural contradiction) — captured decisions are internally consistent
2. Every seeded ADR has evidence that post-validates (cited files and lines exist)
3. `product graph check` exits 0 or 2 (warnings only)

Coverage gaps are expected — they're tracked in `gaps.json` and addressed over time. **Onboarding finds the load-bearing walls. Gap analysis fills in the rest.**

---

---

## Description

Codebase onboarding discovers load-bearing architectural decisions implicit in an existing codebase (where no formal ADRs exist) and produces a minimum viable knowledge graph. It is a three-phase pipeline: scan (`product onboard scan`) asks an LLM to detect decision candidates from source code patterns; triage (`product onboard triage`) lets the team confirm, enrich, merge, or reject each candidate interactively; seed (`product onboard seed`) creates ADR files and feature stubs from confirmed candidates. The LLM detects; humans decide. No ADR enters the graph without human triage. Post-onboarding gap analysis (`product gap check --all`) drives incremental enrichment of the resulting graph (ADR-027 and ADR-022).

## Functional Specification

### Inputs

- **`product onboard scan PATH [--output candidates.json]`**: source directory to scan; output path for the candidates JSON file
- **`product onboard triage candidates.json [--interactive]`**: candidates file from scan phase; `--interactive` enables per-candidate prompt/confirm flow
- **`product onboard seed triaged.json`**: triaged candidates file; seeds ADR and feature files
- **`product.toml` `[onboard]`**: `prompt-version`, `model`, `max-candidates` (default 30), `confidence-threshold`, `chunk-strategy`, `evidence-validation` (default true)
- **Source files in PATH**: read by the scan LLM; content is chunked per `chunk-strategy` (e.g. `import-graph` clusters by module)
- **Existing product graph**: read during seed phase to avoid ID collisions with existing ADRs and features

### Outputs

- **`candidates.json`**: structured JSON array from the scan phase; each candidate has a signal type, a description, evidence (file path + line number citations), and a confidence score
- **`triaged.json`**: candidates JSON with per-candidate disposition: `confirmed`, `merged`, `rejected`; confirmed candidates may have human-enriched rationale
- **ADR files** (from seed): one `docs/adrs/ADR-XXX-*.md` per confirmed candidate with YAML front-matter and candidate content as body
- **Feature stub files** (from seed): one `docs/features/FT-XXX-*.md` per confirmed candidate that maps to a feature boundary, with YAML front-matter and `status: planned`
- **Post-validation report** (when `evidence-validation = true`): each cited file path and line number is checked to exist; invalid evidence is flagged before triage

### State

- `candidates.json` and `triaged.json` are intermediate files written by scan and triage respectively; they are not committed to the repository unless the operator chooses to retain them.
- Seeded ADR and feature files are written atomically (ADR-015) and become part of the knowledge graph immediately.
- The existing graph is read (not modified) during scan and triage. The seed phase is the only phase that writes to the graph.

### Behaviour

1. **Scan**: the LLM receives source files (chunked by `chunk-strategy`) and a prompt instructing it to identify six signal types: Consistency, Boundary, Constraint, Convention, Absence, Dependency. It is instructed to produce at most `max-candidates` candidates, each with specific file + line evidence. Candidates with fabricated evidence paths are rejected by post-validation before triage.
2. **Evidence validation**: when `evidence-validation = true`, each candidate's cited file paths and line numbers are checked to exist on disk after the scan completes. Candidates with invalid evidence are flagged in `candidates.json` with a `evidence-invalid` flag; the triage phase surfaces these prominently.
3. **Triage**: the interactive triage command presents each candidate with its description, signal type, and evidence. The developer confirms (possibly adding rationale), merges into another candidate, or rejects. Confirmed candidates with empty rationale are valid — the "enrich later" principle applies.
4. **Seed**: confirmed candidates are written as ADR files with the next available ADR-XXX ID and as feature stubs with the next available FT-XXX ID. The ADR body contains the candidate description and evidence as initial content. IDs are assigned atomically to avoid collisions with concurrent writes.
5. **Exit criterion check**: onboarding is complete when `product gap check --all` produces no G005 (architectural contradiction) and `product graph check` exits 0 or 2. Coverage gaps are expected and tracked in `gaps.json`.
6. **`max-candidates` cap**: the scan prompt is instructed to produce at most `max-candidates` candidates. This prevents the "archaeology dump" failure mode where the LLM generates 40 low-value candidates with superficial descriptions.

### Invariants

- Every seeded ADR must have at least one evidence entry. Candidates with no evidence are not seeded — they represent speculation, not detection.
- Evidence-validation is enabled by default. Disabling it (`evidence-validation = false`) is an escape hatch for environments where the tool is run against a read-only copy of the source tree with different paths.
- No ADR enters the graph without a human triage decision. The scan produces candidates; the seed produces ADRs; triage is the mandatory gate between them.
- The scan never modifies source files. It is read-only.

### Error handling

- **LLM returns more than `max-candidates`**: excess candidates are truncated and a warning is emitted. The hard cap enforces the design principle.
- **Evidence validation failures**: candidates with invalid evidence are flagged (not rejected automatically) so the triage step can surface them. A candidate where all evidence is invalid may still be confirmed by a developer who knows the pattern is real — the evidence annotation is simply wrong.
- **Seed ID collision**: if an ADR-XXX or FT-XXX ID would conflict with an existing file, the seed command exits 1 with an error listing the conflict. The developer resolves manually.
- **Triage file not from current scan** (version mismatch): if `triaged.json` references candidates that do not match the current `candidates.json` (different hashes), the seed command warns but proceeds with the confirmed set.

### Boundaries

- Onboarding converts existing code into structured artifacts. It does not convert existing prose documents — that is `product migrate`'s domain.
- The LLM detects signal candidates from code patterns. Human triage is required before any candidate becomes a graph artifact. Fully automated onboarding (no triage) is not supported.
- Onboarding finds load-bearing decisions, not comprehensive architecture documentation. Post-onboarding gap analysis drives incremental enrichment; complete coverage at onboarding time is neither expected nor required.
- The scan produces at most `max-candidates` candidates per run. For large codebases, multiple scan runs with different source subsets may be needed.

## Out of scope

- Automatic ADR creation without human triage
- Converting existing prose documentation to structured artifacts (that is `product migrate`)
- Onboarding from compiled binaries, build artifacts, or non-source files
- Continuous background scanning for new architectural decisions as the codebase evolves
- Automatic linking of onboarded ADRs to existing features (post-seed graph inference via `product graph infer` is the recommended next step)
