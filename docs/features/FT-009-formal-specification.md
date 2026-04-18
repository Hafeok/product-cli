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
  ADR-040: predates ADR-040; feature ships independently of the unified verify / LLM-boundary pipeline
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