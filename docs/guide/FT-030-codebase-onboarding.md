The file write was denied. Here's the complete documentation for FT-030 — Codebase Onboarding (~260 lines). You can save it to `docs/guide/FT-030-codebase-onboarding.md`:

---

## Overview

Codebase onboarding discovers load-bearing architectural decisions from an existing codebase and produces a minimum viable knowledge graph. It targets the common case where a team has 10k–500k lines of code but no formal architecture documentation — decisions live in code patterns, not in prose. The `product onboard` command runs a three-phase pipeline: **scan** (detect decision candidates from code signals), **triage** (team confirms, merges, or rejects candidates), and **seed** (confirmed candidates become ADR files and feature stubs). The LLM detects patterns; humans provide meaning. Onboarding is specified by ADR-027.

## Tutorial

### Your first onboarding run

Start by scanning a source directory. This produces decision candidates — structured hypotheses about load-bearing patterns in your code:

```bash
product onboard scan ./src --output candidates.json
```

The scan inspects source files for six signal types (consistency, boundary, constraint, convention, absence, dependency) and writes a `candidates.json` file containing the detected candidates with evidence.

### Reviewing candidates interactively

Run triage in interactive mode to review each candidate:

```bash
product onboard triage candidates.json --interactive --output triaged.json
```

For each candidate, you see the signal type, title, observation, evidence (file paths and line numbers), and a hypothesised consequence. You respond with one of:

- `c` — **confirm** the candidate as a real decision
- `r` — **reject** it (not a decision, discarded permanently)
- `m` — **merge** it with another candidate (e.g., `m` then `DC-001`)
- `s` — **skip** it for later review

You do not need to enrich every candidate immediately. Confirmed candidates with empty rationale are valid — gap analysis (FT-029) will surface missing rationale as G003 findings over time.

### Seeding the knowledge graph

Convert triaged candidates into ADR files and feature stubs:

```bash
product onboard seed triaged.json
```

This creates ADR files in `docs/adrs/` with sequential IDs and feature stubs grouped by evidence proximity. After seeding, verify the graph is structurally sound:

```bash
product graph check
```

Expect W001 (orphaned artifacts) and W002 (no tests) warnings — these are normal for a freshly onboarded graph. No E-class errors should appear.

### Checking the exit criterion

Onboarding is "done enough" when:

```bash
product gap check --all
# No G005 (architectural contradiction) findings
# G003 (missing rationale) and G001 (missing tests) are expected

product graph check
# Exit code 0 or 2 (warnings only)
```

Coverage gaps are tracked in `gaps.json` and addressed incrementally through normal workflows.

## How-to Guide

### Preview what seed would create without writing files

```bash
product onboard seed triaged.json --dry-run
```

This prints the proposed file paths and ADR IDs to stdout without creating any files. Use this to verify the output before committing to it.

### Limit the number of candidates

Prevent an overwhelming scan by capping candidates:

```bash
product onboard scan ./src --output candidates.json --max-candidates 10
```

Only the top candidates by consequence severity are kept. The default cap is 30 (configured via `[onboard] max-candidates` in `product.toml`).

### Skip evidence validation during scan

If you want to scan quickly without checking that cited files and lines exist:

```bash
product onboard scan ./src --output candidates.json --no-validate
```

By default, evidence validation is enabled — candidates citing non-existent files are flagged with warnings.

### Batch-confirm all candidates

If you trust the scan output and want to skip interactive review:

```bash
product onboard triage candidates.json --output triaged.json
```

Without the `--interactive` flag, triage runs in batch mode and auto-confirms all candidates.

### Merge duplicate candidates during triage

When two candidates describe the same decision from different angles (e.g., a boundary signal from the repository module and an absence signal from handler modules), merge them:

1. When prompted for the first candidate, enter `m`
2. Enter the target candidate ID (e.g., `DC-001`)
3. The merged candidate combines evidence from both, creating a single ADR during seed

### Combine onboarding with document migration

If your team has both existing documents and undocumented code decisions:

1. Migrate existing docs first:
   ```bash
   product migrate prd docs/architecture.md
   ```

2. Then onboard the codebase to find decisions not covered by docs:
   ```bash
   product onboard scan ./src --output candidates.json
   product onboard triage candidates.json --interactive --output triaged.json
   product onboard seed triaged.json
   ```

3. Run gap analysis to see the combined state:
   ```bash
   product gap check --all
   ```

### Enrich onboarded ADRs after seeding

Seeded ADRs have TODO markers for rationale and rejected alternatives. Use authoring sessions (FT-022) to fill them in:

1. Open the ADR file (e.g., `docs/adrs/ADR-028-database-access-through-repository.md`)
2. Replace `<!-- TODO: add rationale -->` with the actual rationale
3. Replace `<!-- TODO: add rejected alternatives -->` with considered alternatives
4. Run `product gap check ADR-028` to confirm the G003 finding is resolved

## Reference

### Commands

#### `product onboard scan`

Scans source files and produces decision candidates.

```
product onboard scan <SOURCE> [OPTIONS]
```

| Argument / Flag | Type | Default | Description |
|---|---|---|---|
| `<SOURCE>` | positional | required | Path to the source directory to scan |
| `--output` | string | `candidates.json` | Output file for candidates JSON |
| `--max-candidates` | integer | none (config default: 30) | Maximum number of candidates to produce |
| `--no-validate` | flag | false | Disable evidence post-validation |

**Exit codes:** 0 on success, non-zero on error.

#### `product onboard triage`

Reviews candidates from scan and produces triaged output.

```
product onboard triage <SOURCE> [OPTIONS]
```

| Argument / Flag | Type | Default | Description |
|---|---|---|---|
| `<SOURCE>` | positional | required | Path to `candidates.json` from scan |
| `--interactive` | flag | false | Enable interactive triage (reads actions from stdin) |
| `--output` | string | `triaged.json` | Output file for triaged candidates |

**Interactive actions:** `c` (confirm), `r` (reject), `m` (merge with target ID), `s` (skip), `q` (quit).

**Batch mode:** Without `--interactive`, all candidates are auto-confirmed.

#### `product onboard seed`

Converts confirmed candidates into ADR files and feature stubs.

```
product onboard seed <SOURCE> [OPTIONS]
```

| Argument / Flag | Type | Default | Description |
|---|---|---|---|
| `<SOURCE>` | positional | required | Path to `triaged.json` from triage |
| `--dry-run` | flag | false | Show proposed files without writing |

**Created artifacts:**
- ADR files in the configured `adrs` path (default: `docs/adrs/`)
- Feature stubs in the configured `features` path (default: `docs/features/`)

### Candidate JSON format

```json
{
  "candidates": [
    {
      "id": "DC-001",
      "signal_type": "boundary",
      "title": "Database access exclusively through the repository layer",
      "observation": "All 23 database queries are in src/repo/*.rs...",
      "evidence": [
        {
          "file": "src/repo/users.rs",
          "line": 14,
          "snippet": "pub async fn find_by_id(pool: &PgPool, ...)",
          "evidence_valid": true
        }
      ],
      "hypothesised_consequence": "Adding queries outside src/repo/ would bypass...",
      "confidence": "high",
      "warnings": []
    }
  ],
  "scan_metadata": {
    "files_scanned": 142,
    "tokens_used": 85000,
    "model": "claude-sonnet-4-6",
    "prompt_version": "onboard-scan-v1"
  }
}
```

### Signal types

| Signal | What the LLM observes | Example |
|---|---|---|
| **consistency** | Same pattern repeated across the codebase | Every handler returns `Result<Json<T>, AppError>` |
| **boundary** | Only certain modules access a resource | Only `src/repo/` imports `sqlx` |
| **constraint** | All X comes from Y, never from Z | Config only from environment variables |
| **convention** | Different treatment for different categories | API structs derive `Serialize`; internal structs do not |
| **absence** | Something is deliberately not used | No `std::thread` — all concurrency via `tokio::spawn` |
| **dependency** | A foundational dependency is pinned with explanation | Specific crate version pinned in `Cargo.toml` |

### Confidence levels

| Level | Meaning |
|---|---|
| `high` | Pattern is universal and enforced by types or visibility |
| `medium` | Pattern is consistent but not compiler-enforced |
| `low` | Pattern exists but has exceptions |

### Configuration

The `[onboard]` section in `product.toml`:

```toml
[onboard]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-candidates = 30
confidence-threshold = "low"
chunk-strategy = "import-graph"
evidence-validation = true
```

| Key | Type | Default | Description |
|---|---|---|---|
| `prompt-version` | string | `"1"` | Version of the scan prompt to use |
| `model` | string | `"claude-sonnet-4-6"` | LLM model for scan phase |
| `max-candidates` | integer | `30` | Upper bound on candidates per scan |
| `confidence-threshold` | string | `"low"` | Minimum confidence to include (`low`, `medium`, `high`) |
| `chunk-strategy` | string | `"import-graph"` | How to split large codebases: `import-graph`, `directory`, `flat` |
| `evidence-validation` | boolean | `true` | Post-validate that cited files and lines exist |

### Seeded ADR structure

Each seeded ADR file contains:

- **Front-matter:** `id`, `status: proposed`, empty `features` and `tests` arrays
- **Context section:** derived from the candidate's observation and evidence
- **Decision section:** derived from the candidate's title
- **Rationale section:** developer-provided text from enrichment, or `<!-- TODO: add rationale -->`
- **Rejected alternatives section:** developer-provided text, or `<!-- TODO: add rejected alternatives -->`

## Explanation

### Why three phases instead of one?

The scan → triage → seed pipeline separates what LLMs are good at (pattern detection across large codebases) from what they are bad at (inferring human intent and rationale). A single-phase approach that generates ADRs directly produces the "archaeology dump" — technically accurate descriptions with no rationale. The intermediate candidate format forces explicit human confirmation before anything enters the graph (ADR-027).

### Decision candidates are not ADRs

The `DC-NNN` IDs are temporary. A candidate is a question: "this pattern appears intentional — is it a decision?" Only after human confirmation does it receive a real `ADR-NNN` ID and enter the knowledge graph. This prevents graph pollution from LLM hallucinations.

### The "enrich later" principle

Onboarding avoids the perfectionism trap by allowing confirmed candidates to become ADRs with empty rationale. An ADR with a correct title and grounded evidence is already useful — it tells an agent "this pattern is intentional, don't break it." Gap analysis (FT-029) surfaces missing rationale as G003 findings, driving incremental enrichment without blocking onboarding completion.

### Evidence grounding as anti-hallucination

Every candidate must cite specific file paths and line numbers. The scan command post-validates this evidence: if a cited file or line does not exist, the candidate is flagged with a warning. This is more reliable than asking the LLM to self-assess its confidence. Use `--no-validate` to skip this check only when you know the codebase has changed since the scan.

### Signal types cross module boundaries by design

The six signal types (consistency, boundary, constraint, convention, absence, dependency) are defined to steer the LLM away from file-level or directory-level descriptions. None map to individual files — they map to decisions that manifest *across* files. This avoids the "wrong unit" failure mode where ADRs describe modules instead of decisions (ADR-027).

### How onboarding relates to migration (FT-020)

Migration converts existing documents into structured artifacts. Onboarding converts existing code into decision candidates. The inputs are fundamentally different: prose vs. patterns. A team may use both — migrate existing docs first, then onboard the codebase to find decisions that were never written down.

### Chunking for large codebases

Codebases above ~50 files cannot fit in a single LLM context window. The `chunk-strategy` configuration controls how files are split:

- **`import-graph`** (default) — parses the dependency/import graph to identify module clusters, scans each cluster independently, then deduplicates and runs a cross-cluster pass for decisions that span clusters
- **`directory`** — splits by top-level directories
- **`flat`** — no splitting, processes all files together (only for small codebases)

Chunking is deterministic given the same file set, ensuring scan reproducibility.
