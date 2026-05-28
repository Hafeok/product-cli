---
id: FT-009
title: Formal Specification
phase: 1
status: complete
depends-on: []
adrs:
- ADR-005
- ADR-011
tests:
- TC-013
- TC-014
- TC-015
- TC-160
domains:
- data-model
domains-acknowledged:
  ADR-042: Pre-dates ADR-042; this feature does not define TC types or validate the type vocabulary. FT-048 owns the mechanics.
  ADR-047: Predates ADR-047; this feature does not author the functional-spec body convention. FT-055 owns the structural validator and W030 mechanics.
  ADR-048: Predates ADR-048; this feature does not author the canonical .product/ folder layout. FT-057 owns the migration command and discovery fallback.
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
  ADR-041: Pre-dates ADR-041; this feature does not author absence TCs or set removes/deprecates on ADRs. FT-047 owns the mechanics.
  ADR-043: Predates ADR-043; feature command adapters were written before the slice+adapter pattern was formalised and are not retroactively refactored.
  ADR-018: Predates the 2026-04-22 scope promotion of ADR-018 to cross-cutting. Test coverage reflects the property/session/benchmark strategy as it existed when this feature shipped; not retroactively reclassified.
---

⟦Λ:Benchmark⟧{
  baseline≜condition(none)
  target≜condition(product)
  scorer≜rubric_llm(temperature:0)
  pass≜score(product) ≥ 0.80 ∧ score(product) - score(naive) ≥ 0.15
}

⟦Ε⟧⟨δ≜0.85;φ≜80;τ≜◊?⟩
```

The evidence block fields are:
- `δ` — specification confidence (0.0–1.0)
- `φ` — coverage completeness (0–100%)
- `τ` — stability signal: `◊⁺` stable, `◊⁻` unstable, `◊?` unknown

### Repository Config (`product.toml`)

The complete canonical `product.toml`. All sections except `[paths]`, `[phases]`, and `[prefixes]` are optional and shown with their defaults.

```toml
name = "picloud"
schema-version = "1"

[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
metrics = "metrics.jsonl"
gaps = "gaps.json"
drift = "drift.json"
prompts = "benchmarks/prompts"

[phases]
1 = "Cluster Foundation"
2 = "Products and IAM"
3 = "RDF and Event Store"
4 = "Operational Maturity"

[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"

# Concern domain vocabulary — controlled by the project, not by Product
# Any domain declared in ADR or feature front-matter must appear here
[domains]
security        = "Authentication, authorisation, secrets, trust boundaries"
storage         = "Persistence, durability, volume, block devices, backup"
consensus       = "Raft, leader election, log replication, cluster membership"
networking      = "mDNS, mTLS, DNS, service discovery, port allocation"
error-handling  = "Error model, diagnostics, exit codes, panics, recovery"
observability   = "OTel, metrics, tracing, logging, telemetry"
iam             = "Identity, OIDC, tokens, RBAC, workload identity"
scheduling      = "Workload placement, resource limits, eviction"
api             = "CLI surface, MCP tools, event schema, resource language"
data-model      = "RDF, SPARQL, ontology, event sourcing, projections"

# MCP server settings (product mcp)
[mcp]
write = true                    # enable write tools over MCP
port = 7777                     # HTTP transport port
cors-origins = ["https://claude.ai"]
# token = ""                    # override with PRODUCT_MCP_TOKEN env var

# Agent invocation (product implement)
[agent]
default = "claude-code"         # claude-code | cursor | custom
auto-verify = true
gap-gate = "high"               # refuse to implement if gaps at this severity

[agent.claude-code]
flags = []

[agent.custom]
command = "./scripts/agent.sh {context_file} {feature_id}"

# Versioned system prompts for authoring sessions
[author]
feature-prompt-version = "1"
adr-prompt-version = "1"
review-prompt-version = "1"
agent = "claude-code"

# Versioned implementation prompt
[implementation-prompt]
version = "1"

# LLM gap analysis settings (product gap check)
[gap-analysis]
prompt-version = "1"
model = "claude-sonnet-4-6"
max-findings-per-adr = 10
severity-threshold = "medium"   # findings below this are informational only

# Drift detection settings (product drift check)
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20

# Architectural fitness thresholds (product metrics threshold)
[metrics]
record-on-merge = true          # automatically append to metrics.jsonl in CI

[metrics.thresholds]
spec_coverage           = { min = 0.90, severity = "error" }
test_coverage           = { min = 0.80, severity = "error" }
exit_criteria_coverage  = { min = 0.60, severity = "warning" }
phi                     = { min = 0.70, severity = "warning" }
gap_resolution_rate     = { min = 0.50, severity = "warning" }
drift_density           = { max = 0.20, severity = "warning" }
```

---

```

---

## Description

FT-009 covers two foundational specification concerns. First, the numeric artifact ID scheme (ADR-005): prefixed zero-padded integers (`FT-XXX`, `ADR-XXX`, `TC-XXX`) auto-incremented by `product feature/adr/test new`, permanently assigned on creation, and never reused when an artifact is retired. The sub-namespace extension (`TC-CQ-001`, `TC-PLT-001`) is supported for cross-cutting TCs without displacing feature-specific TC IDs. Second, the `⟦Λ:Benchmark⟧` formal block type from ADR-011: its syntax, required fields (`baseline`, `target`, `scorer`, `pass`), and the `scorer` sub-expression grammar (`rubric_llm(temperature:0)`). The file body above also provides the complete canonical `product.toml` with all sections, keys, and their defaults — this is the reference configuration that governs Product's behaviour across all commands.

## Functional Specification

### Inputs

- `product feature new "<title>"` / `product adr new "<title>"` / `product test new "<title>" --type TYPE` — scaffold commands that trigger ID auto-increment.
- The existing artifact files in the configured directories, scanned to determine the current maximum numeric suffix for each prefix type.
- For benchmark TCs: a `⟦Λ:Benchmark⟧{…}` block in the TC file body containing `baseline`, `target`, `scorer`, and `pass` fields.
- `product.toml` at the repository root, parsed according to the canonical schema shown in the file body above.

### Outputs

- **ID assignment**: a new artifact file whose `id` field is set to `PREFIX-NNN` where `NNN` is zero-padded to at least three digits and equals `max(existing numeric suffixes for this prefix) + 1`. If no existing artifacts exist for the prefix, `NNN = 001`.
- **Benchmark block AST**: a `FormalBlock::Benchmark(BenchmarkBlock)` value with `baseline`, `target`, `scorer` (kind + params), and `pass` (raw expression string) fields, stored on the `TestCriterion` struct.
- **product.toml configuration**: all config values (paths, phases, prefixes, domains, MCP settings, agent settings, gap analysis, drift, metrics, thresholds) resolved from `product.toml` with documented defaults applied for absent optional keys.

### State

Stateless between invocations. The maximum existing ID for each prefix type is determined by scanning the configured directories on every `feature/adr/test new` invocation; no ID counter is persisted outside the artifact files themselves.

### Behaviour

1. **ID auto-increment (ADR-005)**: `product feature/adr/test new` scans the configured directory, parses the `id` field from every artifact file's front-matter, extracts the numeric suffix, finds the maximum, and assigns `max + 1` to the new artifact. Gaps in the numeric sequence (retired artifacts with `status: abandoned`) are not filled — the new ID is always `max + 1`, not the first unused number (TC-014).
2. **ID conflict prevention**: before writing the new file, Product checks that the computed ID does not already exist in the graph. If it does (e.g., two concurrent `new` commands), the command exits with E005 and does not overwrite the existing file (TC-015).
3. **Sub-namespace IDs**: `TC-CQ-NNN` and `TC-PLT-NNN` are treated identically to plain `TC-NNN` by the parser and graph engine. The sub-namespace prefix is a human-readable classifier only; it does not affect ID uniqueness checks (which compare the full `id` string) or graph traversal logic (ADR-005).
4. **Benchmark block parsing (ADR-011, ADR-016)**: a `⟦Λ:Benchmark⟧` block is parsed by the formal parser. Required fields (`baseline`, `target`, `scorer`, `pass`) are extracted into the `BenchmarkBlock` struct. The `pass` expression is stored as a raw string for verbatim context bundle output. The `scorer` field is parsed into `ScorerConfig` with `kind` and `params` (list of key-value pairs).
5. **product.toml parsing**: on every invocation, `product.toml` is read and all sections are deserialized using `toml` crate. Optional sections use documented defaults when absent. The `[domains]` vocabulary table governs domain validation for feature and ADR front-matter (E012). The `[phases]` table provides human-readable phase names for status and checklist output.

### Invariants

- Every `id` in the repository is unique within its prefix type. Duplicate IDs are reported as E005 at graph build time.
- ID numeric suffixes are monotonically increasing relative to the existing maximum at the time of creation; gaps are never filled.
- Retired artifacts (status `abandoned`) retain their IDs permanently — IDs are never renumbered or reused (ADR-005).
- `product.toml` `schema-version` is an integer string; any non-integer value is a parse error.
- All `[domains]` keys in `product.toml` must be referenced by at least one ADR or Feature before their domain string is considered defined; domain strings not present in `[domains]` used in front-matter are reported as E012.

### Error handling

- No artifacts of a prefix type exist yet → first ID is `PREFIX-001`.
- Computed ID already exists in graph (concurrent creation race) → E005, command exits without writing; the existing file is unchanged (TC-015).
- `⟦Λ:Benchmark⟧` block missing a required field (`baseline`, `target`, `scorer`, `pass`) → E001 with field name and line; the block is stored as raw text for the bundle.
- `product.toml` absent → hard error on startup with hint to run `product init`.
- `product.toml` has unrecognised top-level keys → they are silently ignored (forward compatibility).
- Domain string in front-matter not present in `product.toml` `[domains]` table → E012.

### Boundaries

- ID assignment is per-prefix-type; `FT-XXX`, `ADR-XXX`, and `TC-XXX` counters are independent. Adding a new Feature does not affect the ADR counter.
- The prefix strings (`FT`, `ADR`, `TC`) are configurable in `product.toml` `[prefixes]`; the binary uses whatever prefix is declared there, not hardcoded strings.
- The full `product.toml` schema shown in the file body above is the canonical reference; no section other than `[paths]`, `[phases]`, and `[prefixes]` is required for Product to start.

## Out of scope

- Semantic versioning of artifact IDs — IDs are opaque numeric references, not version identifiers (ADR-005).
- Renumbering or compacting the ID space after artifact retirement.
- The full `product.toml` configuration section for authoring sessions, implementation prompts, and LLM model selection — those are consumed by the agent orchestration pipeline (FT-030) and gap analysis (FT-019), not by the core ID and schema machinery covered here.
- Execution of benchmark TCs against external rubric files — the `⟦Λ:Benchmark⟧` block is parsed and preserved; benchmark execution is external to Product.
