## Overview

Codebase onboarding discovers load-bearing architectural decisions from an existing codebase and produces a minimum viable knowledge graph. Rather than documenting everything, it focuses on decisions that would cause cascading failures if violated by an agent or new engineer who didn't know about them. The pipeline has three phases — **scan** (LLM detects decision candidates from code patterns), **triage** (team confirms, enriches, merges, or rejects candidates), and **seed** (confirmed candidates become ADR files and feature stubs). See ADR-027 for the full specification.

## Tutorial

This tutorial walks you through onboarding a small codebase. You will scan for architectural decisions, review what the LLM found, and seed a starter knowledge graph.

### Step 1: Scan the codebase

Point the scan at your source directory. The LLM reads the code and proposes decision candidates — patterns that look like deliberate architectural choices.

```bash
product onboard scan ./src --output candidates.json
```

This produces `candidates.json` containing structured hypotheses. Each candidate has a title, observation, evidence (file paths and line numbers), and a hypothesised consequence of violation.

### Step 2: Review the candidates

Open `candidates.json` to see what was found. Each candidate looks like this:

```json
{
  "id": "DC-001",
  "signal_type": "boundary",
  "title": "Database access exclusively through the repository layer",
  "observation": "All 23 database queries are in src/repo/*.rs...",
  "evidence": [
    {"file": "src/repo/users.rs", "line": 14, "snippet": "pub async fn find_by_id(...)"}
  ],
  "hypothesised_consequence": "Adding queries outside src/repo/ would bypass transaction boundaries...",
  "confidence": "high"
}
```

Candidates are not ADRs yet. They are questions: "is this pattern a real decision?"

### Step 3: Triage the candidates

Run interactive triage to confirm, reject, or merge candidates:

```bash
product onboard triage candidates.json --interactive --output triaged.json
```

For each candidate, you will see a summary and be prompted to act:

- Press `c` to **confirm** — accept as a real decision
- Press `e` to **enrich** — opens your editor to add rationale and rejected alternatives
- Press `m` to **merge** — combine with another candidate that describes the same decision
- Press `r` to **reject** — discard (not a real decision)
- Press `s` to **skip** — defer judgment for later

You do not need to enrich every candidate. Confirming without enrichment is fine — gap analysis will track the missing rationale.

### Step 4: Seed the knowledge graph

Convert confirmed candidates into ADR files and feature stubs:

```bash
product onboard seed triaged.json
```

This creates ADR files in `docs/adrs/` and feature stubs in `docs/features/`, with proper front-matter and cross-links.

### Step 5: Check the result

Verify that the seeded graph is structurally valid:

```bash
product graph check
product gap check --all
```

Expect warnings (orphaned artifacts, missing tests, missing rationale) but no structural errors. These warnings are your backlog — address them incrementally.

## How-to Guide

### Onboard a large codebase (50+ files)

1. Use include/exclude filters to focus the scan:
   ```bash
   product onboard scan ./src --include "*.rs,*.toml" --exclude "target/*,tests/*" --output candidates.json
   ```
2. Set a candidate cap to prevent overwhelming triage:
   ```bash
   product onboard scan ./src --max-candidates 20 --output candidates.json
   ```
3. The scan automatically chunks large codebases using the import graph. Each module cluster is scanned independently, then candidates are deduplicated.

### Preview what seed would create without writing files

Run seed with the `--dry-run` flag to see proposed file paths and ADR IDs:

```bash
product onboard seed triaged.json --dry-run
```

No files are written. Stdout shows what would be created. Run again without `--dry-run` when satisfied.

### Combine onboarding with document migration

If you have both existing architecture documents and undocumented code decisions:

1. Migrate existing docs first:
   ```bash
   product migrate import architecture.md
   ```
2. Then onboard the codebase to find decisions that were never documented:
   ```bash
   product onboard scan ./src --output candidates.json
   product onboard triage candidates.json --interactive --output triaged.json
   product onboard seed triaged.json
   ```

### Merge duplicate candidates during triage

When the LLM detects the same decision from two different signals (e.g., a boundary signal from the module that owns a resource and an absence signal from modules that don't import it):

1. Start interactive triage:
   ```bash
   product onboard triage candidates.json --interactive --output triaged.json
   ```
2. When you see the duplicate, press `m` to merge
3. Enter the target candidate ID (e.g., `DC-001`)
4. The merged candidate keeps the target's title and combines evidence from both

### Incrementally enrich onboarded ADRs

After onboarding, use gap analysis to find ADRs that need rationale:

```bash
product gap check --all
```

G003 findings indicate ADRs with missing rationale. Address them through authoring sessions:

```bash
product author start ADR-028
```

### Determine when onboarding is "done enough"

Check the three exit criteria:

1. No architectural contradictions:
   ```bash
   product gap check --all  # no G005 findings
   ```
2. All evidence post-validates (cited files and lines exist)
3. No structural errors:
   ```bash
   product graph check  # exit code 0 or 2
   ```

Coverage gaps (G001, G006) and missing rationale (G003) are expected and acceptable after onboarding.

## Reference

### Commands

#### `product onboard scan`

Scans source files and produces decision candidates.

```bash
product onboard scan <PATH> [OPTIONS]
```

| Flag | Description | Default |
|---|---|---|
| `--output <FILE>` | Output path for `candidates.json` | `candidates.json` |
| `--include <GLOBS>` | Comma-separated file globs to include | all files |
| `--exclude <GLOBS>` | Comma-separated file globs to exclude | none |
| `--max-candidates <N>` | Upper bound on candidates produced | 30 (from config) |

**Output format:** JSON file with a `candidates` array and `scan_metadata` object.

Each candidate contains:

| Field | Type | Description |
|---|---|---|
| `id` | string | Temporary ID (`DC-NNN`), not an ADR ID |
| `signal_type` | string | One of: `consistency`, `boundary`, `constraint`, `convention`, `absence`, `dependency` |
| `title` | string | Decision phrased as a choice ("X through Y") |
| `observation` | string | What the LLM observed in the code |
| `evidence` | array | File paths, line numbers, and code snippets |
| `hypothesised_consequence` | string | What would break if this decision were violated |
| `confidence` | string | `high`, `medium`, or `low` |

`scan_metadata` contains: `files_scanned`, `tokens_used`, `model`, `prompt_version`.

#### `product onboard triage`

Structured review of decision candidates.

```bash
product onboard triage <CANDIDATES_FILE> [OPTIONS]
```

| Flag | Description | Default |
|---|---|---|
| `--interactive` | Present candidates one by one for review | false |
| `--output <FILE>` | Output path for triaged results | `triaged.json` |

**Interactive actions:**

| Key | Action | Effect |
|---|---|---|
| `c` | Confirm | Accept candidate as-is |
| `e` | Enrich | Opens `$EDITOR` to add rationale and alternatives |
| `m` | Merge | Combine with another candidate by ID |
| `r` | Reject | Discard permanently |
| `s` | Skip | Defer for later review |
| `q` | Quit | Stop triage, save progress |

#### `product onboard seed`

Converts confirmed candidates into ADR files and feature stubs.

```bash
product onboard seed <TRIAGED_FILE> [OPTIONS]
```

| Flag | Description | Default |
|---|---|---|
| `--dry-run` | Show what would be created without writing files | false |

**Behaviour:**
- ADR files are created in the configured ADRs directory with the next available sequence number
- ADRs have `status: proposed` and contain Context, Decision, and placeholder Rationale sections
- Feature stubs are grouped by signal proximity (overlapping evidence files)
- Feature stubs have `status: planned` and empty test lists
- Runs `product graph check` automatically after seeding
- Runs `product gap check --all` to establish the baseline `gaps.json`

### Configuration

In `product.toml`:

```toml
[onboard]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-candidates = 30
confidence-threshold = "low"
chunk-strategy = "import-graph"
evidence-validation = true
```

| Key | Values | Description |
|---|---|---|
| `prompt-version` | version string | Scan prompt version (stored in `benchmarks/prompts/`) |
| `model` | model ID | LLM model used for scanning |
| `max-candidates` | integer | Upper bound on candidates per scan |
| `confidence-threshold` | `low`, `medium`, `high` | Minimum confidence to include in output |
| `chunk-strategy` | `import-graph`, `directory`, `flat` | How to split large codebases for scanning |
| `evidence-validation` | boolean | Post-validate that cited files and lines exist |

### Signal types

| Signal | What the LLM looks for | Example |
|---|---|---|
| **Consistency** | Same pattern repeated across the codebase | Every handler returns `Result<Json<T>, AppError>` |
| **Boundary** | Only certain modules access a resource | Only `src/repo/` imports `sqlx` |
| **Constraint** | All X comes from Y, never from Z | Config from env vars only, never files |
| **Convention** | Different treatment for different categories | API structs derive `Serialize`; internal structs do not |
| **Absence** | Something deliberately not used | No direct `std::thread` usage; all concurrency via `tokio::spawn` |
| **Dependency** | Foundational dependency pinned with explanation | Specific crate version pinned in `Cargo.toml` |

### Exit codes

`product graph check` after seeding:
- **0** — clean graph, no errors or warnings
- **2** — warnings only (W001 orphaned, W002 no tests) — expected after onboarding

`product gap check --all` after seeding:
- **0** — no findings (unlikely after onboarding)
- **1** — findings present (expected: G001, G003)

## Explanation

### Why three phases instead of one?

The scan-triage-seed pipeline separates what LLMs are good at (pattern detection across large codebases) from what they are bad at (inferring human intent and rationale). The LLM finds signals; the team provides meaning. Collapsing these phases produces the "archaeology dump" — technically accurate descriptions of code with no architectural insight. ADR-027 documents this design decision in detail.

### Why candidates instead of direct ADR generation?

Decision candidates (`DC-NNN`) are an intermediate format that forces explicit human confirmation before anything enters the knowledge graph. This prevents graph pollution from LLM hallucinations. A candidate that looks like a real decision might actually be an incidental pattern — only the team knows the difference.

### The "enrich later" principle

Confirmed candidates with empty rationale sections produce valid, useful ADRs. An ADR that says "database access goes through the repository layer" already protects an agent from creating a second data access path. The *why* makes it more useful, but is not blocking. Gap analysis (FT-029) surfaces missing rationale as G003 findings, which the team addresses incrementally. This avoids the perfectionism trap where onboarding never finishes because every ADR needs to be complete.

### Evidence grounding as anti-hallucination

Every candidate must cite specific file paths and line numbers. The scan command post-validates these citations — if a cited file or line does not exist, the candidate is flagged. This is more reliable than asking the LLM to self-assess its confidence, because fabricated evidence is objectively detectable while fabricated confidence is not.

### How onboarding differs from migration (FT-020)

Migration converts existing prose documents (PRDs, ADR docs) into structured artifacts. The input already describes decisions in words. Onboarding converts existing code into structured artifacts. The input is source files where decisions are implicit in patterns. The LLM detects the patterns; the team confirms whether they represent real decisions. A team with both existing docs and undocumented code decisions should use both tools.

### Relationship to gap analysis (FT-029)

Onboarding produces an intentionally sparse graph — just the load-bearing decisions. Gap analysis is the growth engine that drives the graph toward completeness over time. After onboarding, `product gap check --all` produces a baseline of expected gaps (missing rationale, missing tests, missing coverage). These are tracked in `gaps.json` and addressed through normal workflows. Onboarding finds the load-bearing walls; gap analysis fills in the rest.
