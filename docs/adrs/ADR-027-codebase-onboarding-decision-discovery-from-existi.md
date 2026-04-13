---
id: ADR-027
title: Codebase Onboarding — Decision Discovery from Existing Code
status: proposed
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Proposed

**Context:** Product's migration path (ADR-017) handles a specific case: the team already has a monolithic PRD or ADR document and wants to decompose it into structured artifacts. This covers greenfield projects that wrote docs first and legacy projects that happened to keep a single architecture doc up to date.

It does not cover the most common real-world case: an existing codebase of 10k–500k lines where decisions were made over years but never written down. The architecture lives in the code, in commit messages, in the heads of the original authors, and in the consistency of patterns that were enforced by convention rather than documentation.

Naively applying an LLM to this problem produces one of three failure modes:

1. **The archaeology dump.** Scan the whole codebase, generate 40 ADRs. They are technically accurate descriptions of what the code does but contain no rationale, no rejected alternatives, and no evidence grounding. The graph is populated but useless — an agent reading these ADRs learns nothing it couldn't learn by reading the code directly.

2. **The perfectionism trap.** Require every ADR to be complete (rationale, alternatives, test criteria) before proceeding. Onboarding a 50k-line codebase takes six months. The team abandons the effort because they are always "almost done."

3. **The wrong unit.** Start from directory structure or file boundaries instead of decision boundaries. You end up with ADRs that map to modules rather than decisions. "We use a HashMap in the cache layer" is not an architectural decision. "All caches are in-process with no distributed invalidation" is.

The correct framing is not "document the architecture" but rather: **what does an agent need to know to safely modify this codebase?** Specifically: what decisions, if violated, would cause cascading failures that the violator wouldn't predict? These are the high-centrality nodes in the implicit decision graph — the load-bearing walls of the codebase.

This reframing produces a different design. Onboarding is not a documentation exercise. It is a **decision discovery** exercise that produces a minimum viable knowledge graph — just enough for `product context` bundles to protect agents from the most dangerous mistakes. The graph then grows incrementally through gap analysis (ADR-019) and normal authoring workflows (ADR-022).

**Decision:** Implement `product onboard` as a three-phase LLM-assisted decision discovery pipeline: **scan** (LLM proposes decision candidates from code signals), **triage** (team confirms, merges, or rejects candidates in a structured review), and **seed** (confirmed candidates become ADR files with stub features, producing a minimum viable graph). The LLM is used only for signal detection — it proposes *what might be a decision*. Rationale, rejected alternatives, and correctness come from the team during triage.

---

### Phase 1: Scan — Decision Candidate Extraction

```bash
product onboard scan ./src --output candidates.json
product onboard scan ./src --include "*.rs,*.toml" --exclude "target/*,tests/*"
product onboard scan ./src --max-candidates 30
```

The scan phase reads source files and produces **decision candidates** — structured hypotheses about load-bearing decisions. A candidate is not an ADR. It is a question: "this pattern appears intentional and consistent — is it a decision?"

**What the LLM looks for:**

The scan prompt instructs the model to identify patterns that suggest deliberate architectural choices. These are categorised into signal types:

| Signal | Example | Why it matters |
|---|---|---|
| **Consistency signal** | Every HTTP handler returns `Result<Json<T>, AppError>` — never raw status codes | Violating this breaks the error contract for all API consumers |
| **Boundary signal** | Module `db` is the only code that imports `sqlx` — no other module touches the database directly | Violating this creates a second data access path with no transaction guarantees |
| **Constraint signal** | All config values come from environment variables, never from files | Violating this breaks the deployment model (12-factor assumption) |
| **Convention signal** | Every struct that crosses an API boundary derives `Serialize, Deserialize` but internal structs do not | Violating this leaks internal types into the public API |
| **Absence signal** | No code uses threads directly — all concurrency is through `tokio::spawn` | Introducing `std::thread` would bypass the runtime's cooperative scheduling |
| **Dependency signal** | `Cargo.toml` pins a specific major version of a foundational crate with a comment explaining why | Upgrading past that version would break an assumption the codebase relies on |

The scan prompt explicitly instructs the model to **not** produce:
- Descriptions of what code does (that's documentation, not decisions)
- File-level or module-level summaries (wrong unit)
- Style preferences (tabs vs spaces is not load-bearing)
- Anything without a plausible "violating this would break X" consequence

**Candidate output format:**

```json
{
  "candidates": [
    {
      "id": "DC-001",
      "signal_type": "boundary",
      "title": "Database access exclusively through the repository layer",
      "observation": "All 23 database queries in the codebase are in src/repo/*.rs. No other module imports sqlx or holds a connection pool reference.",
      "evidence": [
        {"file": "src/repo/users.rs", "line": 14, "snippet": "pub async fn find_by_id(pool: &PgPool, id: Uuid) -> Result<User>"},
        {"file": "src/repo/mod.rs", "line": 1, "snippet": "//! Repository layer — sole owner of database access"}
      ],
      "hypothesised_consequence": "Adding database queries outside src/repo/ would bypass transaction boundaries and connection pool limits, potentially causing connection exhaustion under load.",
      "confidence": "high"
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

Each candidate has:
- `id` — temporary ID (`DC-NNN`), not an ADR ID. Becomes an ADR ID only after triage confirmation.
- `signal_type` — which signal category triggered it (consistency, boundary, constraint, convention, absence, dependency)
- `title` — phrased as a decision ("X through Y", "All Z via W"), not as a description
- `observation` — what the LLM observed in the code, factually
- `evidence` — file paths, line numbers, and code snippets that ground the observation. This is the critical anti-hallucination mechanism: every candidate must cite specific code.
- `hypothesised_consequence` — what would go wrong if this decision were violated. Phrased as a prediction, not a fact, because the LLM is guessing at intent.
- `confidence` — `high` (pattern is universal and enforced by types/visibility), `medium` (pattern is consistent but not enforced), `low` (pattern exists but has exceptions)

**Chunking strategy for large codebases:**

Codebases above ~50 files cannot be scanned in a single LLM context window. The scan command splits the codebase into chunks:

1. Parse the dependency/import graph to identify module clusters
2. Scan each cluster independently, producing per-cluster candidates
3. Run a deduplication pass: merge candidates that describe the same decision from different vantage points (e.g., "all DB access via repo layer" detected from both the repo module and from the absence of DB imports elsewhere)
4. Run a cross-cluster pass: look for decisions that span clusters (e.g., error handling convention that applies everywhere)

The chunking is deterministic given the same file set. File ordering within chunks is alphabetical. This ensures scan reproducibility.

---

### Phase 2: Triage — Structured Team Review

```bash
product onboard triage candidates.json
product onboard triage candidates.json --interactive
product onboard triage candidates.json --output triaged.json
```

Triage is the phase where human judgment enters. The LLM found signals; the team determines which signals represent real decisions, what the actual rationale was, and what alternatives were considered.

**Interactive triage flow:**

For each candidate, the tool presents:

```
─── DC-003 [boundary] confidence: high ────────────────────────────
Database access exclusively through the repository layer

Observation: All 23 database queries are in src/repo/*.rs. No other
module imports sqlx or holds a connection pool reference.

Evidence:
  src/repo/users.rs:14    pub async fn find_by_id(pool: &PgPool, ...
  src/repo/mod.rs:1       //! Repository layer — sole owner of database access

Hypothesised consequence: Adding queries outside src/repo/ would bypass
transaction boundaries and connection pool limits.

  [c]onfirm  [e]nrich  [m]erge with DC-XXX  [r]eject  [s]kip  [q]uit
```

Actions:
- **confirm** — accept the candidate as-is. It becomes an ADR with the observation as context, empty rationale (to be filled later), and the hypothesised consequence as a starting point.
- **enrich** — opens `$EDITOR` with a pre-filled template. The developer adds rationale ("we chose this because..."), rejected alternatives ("we considered using an ORM but..."), and corrects any inaccuracies in the observation.
- **merge** — combine this candidate with another. Common when the LLM detects the same decision from two different signals. The developer picks which title and evidence to keep.
- **reject** — this is not a decision. "We use HashMap here" → reject. Discarded permanently.
- **skip** — defer judgment. Stays in `candidates.json` for later review.

**The "enrich later" principle:**

Confirmed candidates without enrichment produce ADRs with empty rationale sections. This is intentional. An ADR with a correct title, grounded observation, and plausible consequence is already useful for agent safety — it tells the agent "don't violate this." The rationale explains *why*, which is valuable but not blocking. Gap analysis (G003) will surface the missing rationale as a medium-severity finding, which the team can address incrementally.

This is how onboarding avoids the perfectionism trap: confirm fast, enrich incrementally, let gap analysis drive completeness.

---

### Phase 3: Seed — Minimum Viable Graph

```bash
product onboard seed triaged.json
product onboard seed triaged.json --dry-run
```

The seed phase converts confirmed candidates into Product artifacts:

1. **ADR creation** — each confirmed candidate becomes an ADR file. The `id` is assigned by Product's normal ID sequence (ADR-028, ADR-029, etc.). Front-matter links are empty — no features, no tests. The body contains:
   - **Context:** the candidate's observation and evidence, framed as context
   - **Decision:** the candidate's title, rephrased as a decision statement
   - **Rationale:** developer-provided rationale from enrichment, or empty with a `<!-- TODO: add rationale -->` marker
   - **Rejected alternatives:** developer-provided alternatives from enrichment, or empty with a `<!-- TODO: add rejected alternatives -->` marker

2. **Feature stub creation** — candidates are grouped by signal type and proximity (candidates whose evidence files overlap likely belong to the same feature). Each group becomes a feature stub with a descriptive title and the grouped ADRs linked. Feature stubs have `status: planned` and empty test lists.

3. **Graph health check** — `product graph check` runs automatically after seeding. Expected output: many W001 (orphaned) and W002 (no tests) warnings, zero errors. This is the baseline.

4. **Gap analysis bootstrap** — `product gap check --all` runs against the seeded ADRs. The output becomes the initial `gaps.json` baseline. Expected: many G003 (missing rationale) and G001 (missing tests) findings. These are suppressed with reason "onboarding baseline — to be addressed incrementally" and tracked for resolution.

**The output is a starting point, not a finished product.** The seeded graph is intentionally sparse. It captures the load-bearing decisions — the things that would be dangerous to ignore. Everything else is grown through normal product workflows: authoring sessions add new ADRs, gap analysis surfaces missing coverage, `product implement` drives test criteria creation.

---

### Scan Prompt Design

The onboard scan prompt is versioned and stored at `benchmarks/prompts/onboard-scan-v{N}.md`. Referenced in `product.toml`:

```toml
[onboard]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-candidates = 30             # upper bound on candidates per scan
confidence-threshold = "low"    # minimum confidence to include in output
chunk-strategy = "import-graph" # how to split large codebases: import-graph | directory | flat
evidence-validation = true      # post-validate that cited files and lines exist
```

The prompt structure:

```markdown
You are analysing a codebase to identify load-bearing architectural decisions.

A load-bearing decision is a choice that is:
- Consistent across the codebase (not a one-off)
- Would cause cascading failures if violated by someone who didn't know about it
- Represents a deliberate choice between alternatives (not the only possible approach)

You are NOT looking for:
- Descriptions of what code does
- File-level or module-level summaries
- Style preferences or formatting choices
- Implementation details that are local to a single function
- Anything where "violating this" would be caught by the compiler

For each decision you identify, you MUST provide:
- Specific file paths and line numbers as evidence
- A concrete consequence of violation (not "things would break" — what specifically breaks)
- A confidence level based on how universal and enforced the pattern is

Signal types to scan for: [consistency, boundary, constraint, convention, absence, dependency]

Respond ONLY with JSON matching this schema: [schema]

Source files:
{CHUNKS}
```

**The anti-hallucination constraint:** Every candidate must include `evidence` with real file paths and line numbers from the provided source. The scan command post-validates evidence: if a cited file or line does not exist, the candidate is flagged with a warning. This catches hallucinated evidence before it reaches triage.

---

### Relationship to Existing Features

- **FT-020 (Migration Path)** — onboarding and migration are complementary, not overlapping. Migration handles `existing docs → structured artifacts`. Onboarding handles `existing code → decision candidates → structured artifacts`. A team might use both: migrate their existing docs, then onboard the codebase to find decisions that were never documented.
- **FT-029 (Gap Analysis)** — gap analysis is the growth engine after onboarding. The seeded graph is intentionally incomplete. Gap analysis identifies what's missing and drives the team to fill in rationale, add test criteria, and resolve contradictions incrementally.
- **FT-023 (Agent Orchestration)** — `product implement` is the consumer of the onboarded graph. The onboarding exit criterion is: can `product context FT-XXX` produce a bundle that would prevent an agent from violating a load-bearing decision?
- **FT-022 (Authoring Sessions)** — enriching onboarded ADRs with rationale and rejected alternatives is a normal authoring session. Onboarding produces the stubs; authoring sessions complete them.

---

### Exit Criterion

Onboarding is "done enough" when:

1. `product gap check --all` produces no G005 (architectural contradiction) findings — the captured decisions are internally consistent
2. Every seeded ADR has at least `confidence: high` evidence that post-validates (cited files and lines exist)
3. `product graph check` exits 0 or 2 (warnings only, no structural errors)

Coverage gaps (G001, G006) and missing rationale (G003) are expected and acceptable. They are tracked in `gaps.json` and addressed incrementally. Onboarding does not try to be comprehensive — it tries to find the load-bearing walls.

---

**Rationale:**
- The three-phase design (scan → triage → seed) separates what an LLM is good at (pattern detection across large codebases) from what it is bad at (inferring human intent and rationale). The LLM finds signals; the team provides meaning. This avoids the archaeology dump failure mode.
- Decision candidates are not ADRs. The intermediate format forces the team to make an explicit confirmation decision for every artifact that enters the graph. This prevents graph pollution from LLM hallucinations.
- The "enrich later" principle with gap analysis as the growth driver avoids the perfectionism trap. A confirmed candidate with empty rationale is still useful — it tells an agent "this pattern is intentional, don't break it." The rationale makes it *more* useful but is not blocking.
- Signal types (consistency, boundary, constraint, convention, absence, dependency) are defined to steer the LLM away from the wrong-unit failure mode. None of these signals correspond to files or directories. They correspond to decisions that manifest *across* files.
- Evidence grounding with post-validation is the primary anti-hallucination mechanism. A candidate that cites non-existent files is immediately suspect. This is more reliable than asking the LLM to self-assess confidence.
- The maximum candidate cap (`max-candidates = 30`) prevents the archaeology dump by construction. Even if the LLM finds 100 patterns, only the top 30 by consequence severity are reported. This forces prioritisation toward load-bearing decisions.

**Rejected alternatives:**
- **Fully automated onboarding (no triage phase)** — scan the codebase, generate ADRs directly, let gap analysis clean up. Rejected because it produces the archaeology dump. Without human confirmation, every LLM observation becomes an ADR, most of which describe code behaviour rather than decisions. The graph is populated but useless.
- **Fully manual onboarding (no scan phase)** — provide a questionnaire template: "What's your error handling strategy? What's your data access pattern?" The developer fills it in. Rejected because it requires the developer to already know what's load-bearing, which is precisely what they don't know in a legacy codebase. The LLM scan surfaces decisions the developer has internalised but wouldn't think to document.
- **Start from directory structure** — scan each top-level directory independently, produce one ADR per module. Rejected because it maps to the wrong unit. Architectural decisions cross module boundaries. The repository layer decision affects both `src/repo/` and every module that *doesn't* import `sqlx`. Starting from directories misses the absence signal entirely.
- **Use static analysis instead of LLM** — parse the AST, build a call graph, identify architectural patterns through graph analysis. Rejected for v1: the engineering cost of language-specific parsers for every supported language is prohibitive. An LLM reads any language. Static analysis can be added as a signal source in a future version to complement LLM detection.
- **Onboard into a separate "draft" namespace** — onboarded ADRs get a `draft-ADR-XXX` prefix and live in a separate directory until promoted. Rejected because it creates a two-tier system. Confirmed candidates have been through human triage — they deserve full ADR status. Incomplete rationale is tracked by gap analysis, not by a separate namespace.

---