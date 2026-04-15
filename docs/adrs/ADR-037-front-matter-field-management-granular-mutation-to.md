---
id: ADR-037
title: Front-Matter Field Management — Granular Mutation Tools
status: proposed
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Proposed

**Context:** The MCP server and CLI expose write tools for creating artifacts (`feature new`, `adr new`, `test new`), linking them (`feature link --adr/--test/--dep`), setting status (`feature/adr/test status`), and editing markdown bodies (`body_update`). These tools cover artifact creation and basic graph wiring. However, a significant subset of front-matter fields has no write tool:

| Artifact | Unmanaged fields |
|---|---|
| Feature | `domains`, `domains-acknowledged` |
| ADR | `domains`, `scope`, `supersedes`, `superseded-by`, `source-files` |
| TC | `runner`, `runner-args`, `runner-timeout`, `requires` |

This gap breaks the authoring flow (FT-022). During a phone-based authoring session over the HTTP MCP transport, the agent can scaffold artifacts and link them but cannot:

1. **Classify by domain** — the agent creates a feature touching networking and security but cannot set `domains: [networking, security]`. The domain coverage matrix (FT-019) remains empty. Cross-cutting ADR warnings (W010, W011) cannot be addressed.
2. **Declare supersession** — when the agent drafts a new ADR that supersedes an existing one, it cannot set `supersedes: [ADR-035]` on the new ADR or `superseded-by: [ADR-036]` on the old one. The supersession chain is invisible to the graph.
3. **Set scope** — a new cross-cutting ADR defaults to `feature-specific` and the agent cannot change it. The pre-flight system (ADR-026) never flags it.
4. **Configure TC runners** — every TC requires `runner` and `runner-args` for `product verify` to execute it (ADR-021). Without an MCP tool to set these fields, the agent creates TCs that `verify` silently skips.
5. **Declare source files** — `source-files` on ADRs drives drift detection (ADR-023). Without a tool, drift detection has no governed file mappings to check.
6. **Acknowledge domains** — `domains-acknowledged` on features requires a domain name and mandatory reasoning. Without a tool, domain gaps from `product preflight` cannot be closed from MCP.

The result is that any authoring session that uses only MCP tools produces incomplete artifacts that require manual YAML editing afterward — defeating the purpose of the authoring flow.

**Decision:** Add granular front-matter mutation tools to the CLI and MCP server. Each tool targets a specific field family, validates inputs against the schema and vocabulary, and writes atomically using the existing file safety infrastructure (ADR-015). All new tools are write tools gated behind `mcp.write = true`.

---

### New CLI Commands

**Domain management (features and ADRs):**

```bash
product feature domain FT-009 --add networking --add security
product feature domain FT-009 --remove storage
product adr domain ADR-013 --add error-handling --add api

# Domain acknowledgement (features only):
product feature acknowledge FT-009 --domain security \
  --reason "Rate limiting operates at the Resource API layer. No new trust boundaries."

# Remove an acknowledgement:
product feature acknowledge FT-009 --domain security --remove
```

Domain names are validated against the `[domains]` vocabulary in `product.toml`. Adding a domain not in the vocabulary is E012. Adding an acknowledgement without `--reason` is E011.

**ADR scope:**

```bash
product adr scope ADR-013 cross-cutting
product adr scope ADR-040 domain
product adr scope ADR-041 feature-specific
```

Values are validated against the enum: `cross-cutting`, `domain`, `feature-specific`.

**ADR supersession:**

```bash
product adr supersede ADR-036 --supersedes ADR-035
```

This command performs a bidirectional write:
1. Adds `ADR-035` to the `supersedes` list of `ADR-036`
2. Adds `ADR-036` to the `superseded-by` list of `ADR-035`
3. If `ADR-035` status is `accepted`, changes it to `superseded`

Bidirectional write is atomic — both files are written in a single lock acquisition. If either write fails, neither is committed. This prevents the graph from entering an inconsistent state where one side of the supersession link exists without the other.

Removing a supersession link:

```bash
product adr supersede ADR-036 --remove ADR-035
```

This reverses all three mutations. If `ADR-035` has no remaining `superseded-by` entries after removal, its status is not automatically changed back — status restoration is a manual decision.

**ADR source files:**

```bash
product adr source-files ADR-023 --add src/drift.rs --add src/drift/
product adr source-files ADR-023 --remove src/old_drift.rs
```

Paths are validated to exist in the repository at the time of the command. Missing paths produce a warning (W-class) rather than an error — the source file may not exist yet during authoring.

**TC runner configuration:**

```bash
product test runner TC-054 --runner cargo-test --args "tc_054_product_impact_adr_001"
product test runner TC-054 --timeout 60s
product test runner TC-054 --requires binary-compiled --requires two-node-cluster
product test runner TC-054 --remove-requires two-node-cluster
```

Runner values are validated against the supported set: `cargo-test`, `bash`, `pytest`, `custom`. Prerequisite names are validated against `[verify.prerequisites]` in `product.toml`.

---

### New MCP Tools

Each CLI command maps to an MCP tool following the existing naming convention:

| MCP Tool | CLI Equivalent | Write |
|---|---|---|
| `product_feature_domain` | `product feature domain FT-XXX --add/--remove` | Yes |
| `product_feature_acknowledge` | `product feature acknowledge FT-XXX --domain D --reason R` | Yes |
| `product_adr_domain` | `product adr domain ADR-XXX --add/--remove` | Yes |
| `product_adr_scope` | `product adr scope ADR-XXX VALUE` | Yes |
| `product_adr_supersede` | `product adr supersede ADR-XXX --supersedes ADR-YYY` | Yes |
| `product_adr_source_files` | `product adr source-files ADR-XXX --add/--remove` | Yes |
| `product_test_runner` | `product test runner TC-XXX --runner R --args A` | Yes |

MCP tool parameters:

```json
// product_feature_domain
{ "id": "FT-009", "add": ["networking"], "remove": ["storage"] }

// product_feature_acknowledge
{ "id": "FT-009", "domain": "security", "reason": "No new trust boundaries." }
// To remove: { "id": "FT-009", "domain": "security", "remove": true }

// product_adr_domain
{ "id": "ADR-013", "add": ["error-handling"], "remove": [] }

// product_adr_scope
{ "id": "ADR-013", "scope": "cross-cutting" }

// product_adr_supersede
{ "id": "ADR-036", "supersedes": "ADR-035" }
// To remove: { "id": "ADR-036", "remove": "ADR-035" }

// product_adr_source_files
{ "id": "ADR-023", "add": ["src/drift.rs"], "remove": [] }

// product_test_runner
{ "id": "TC-054", "runner": "cargo-test", "args": "tc_054_product_impact_adr_001", "timeout": "60s" }
```

---

### Validation Rules

All mutations validate before writing:

1. **Artifact existence** — the target artifact must exist. E002 if not found.
2. **Domain vocabulary** — domain names must be in `product.toml` `[domains]`. E012 if not.
3. **Scope enum** — scope values must be one of the three defined values. E001 if not.
4. **Supersession target existence** — the superseded ADR must exist. E002 if not found.
5. **Supersession cycle detection** — after adding the edge, check for cycles in the supersession graph. E004 if cycle detected.
6. **Runner enum** — runner values must be in the supported set. E001 if not.
7. **Prerequisite vocabulary** — `requires` values must be in `product.toml` `[verify.prerequisites]`. E001 if not.
8. **Acknowledgement reasoning** — `--reason` must be non-empty and non-whitespace. E011 if empty.
9. **Idempotency** — adding a domain that already exists, or removing one that doesn't, is a no-op (not an error). This makes tools safe to call repeatedly.

---

### Atomic Write Strategy

Single-file mutations (`feature domain`, `adr scope`, `test runner`) follow the existing atomic write path (ADR-015): read file → parse front-matter → modify field → write temp file → fsync → rename.

The `adr supersede` command is a two-file mutation. Both files are written under a single advisory lock acquisition:

1. Acquire lock
2. Read and parse both ADR files
3. Validate (existence, cycle detection)
4. Write first file atomically (temp + rename)
5. Write second file atomically (temp + rename)
6. If step 5 fails, re-read step 4's file from the renamed temp (it's committed) — the operation is not fully atomic across two files, but the lock prevents concurrent mutations from interleaving
7. Release lock

True two-file atomicity would require a WAL or transaction log — overkill for this use case. The advisory lock serialises all writes, so partial failure during `adr supersede` leaves the first file updated and the second unchanged. The error message instructs the user to complete the operation manually or re-run the command (which is idempotent).

---

### Authoring Prompt Updates

The `author-feature` and `author-adr` system prompts (FT-022, ADR-022) should be updated to include the new tools in the authoring flow:

**Feature authoring addition (after scaffolding):**
```
6. Set domains: call product_feature_domain to classify the feature
7. Run product preflight — address domain gaps with product_feature_acknowledge
8. Call product_graph_check and product_gap_check before ending the session
```

**ADR authoring addition (after drafting):**
```
4. Set scope: call product_adr_scope (cross-cutting, domain, or feature-specific)
5. Set domains: call product_adr_domain
6. If superseding: call product_adr_supersede
7. If source files known: call product_adr_source_files
```

---

**Rationale:**
- Granular tools (one tool per field family) over a generic `product frontmatter set` command. A generic setter would bypass validation — `product frontmatter set FT-009 domains "[networking, storage]"` requires the caller to produce valid YAML, handle list semantics (append vs replace), and know the vocabulary. Typed tools enforce validation at the API boundary, which is where MCP agents interact. An agent calling `product_feature_domain` with `"add": ["securty"]` gets E012 immediately — not a corrupt YAML file discovered later by `graph check`.
- Bidirectional supersession writes prevent the most common graph inconsistency. Manually editing one ADR's `supersedes` without updating the other's `superseded-by` creates a half-link that `graph check` catches but shouldn't exist in the first place. The tool makes it impossible to create a one-sided supersession link.
- Idempotent add/remove semantics make tools safe for retry. An MCP agent that encounters a timeout and retries the call will not create duplicates or errors.
- Source file path validation is a warning, not an error, because authoring often precedes implementation. The ADR may declare `source-files: [src/drift.rs]` before the file exists.

**Rejected alternatives:**
- **Generic `product frontmatter set ARTIFACT FIELD VALUE`** — accepts any field name and any value as a string. No type safety, no vocabulary validation, no bidirectional writes. The caller must construct valid YAML values. Rejected because it moves validation responsibility from the tool to the caller, which is backwards for an MCP interface where the caller is an LLM agent.
- **`product body_update` for front-matter changes** — `body_update` already exists for markdown body editing. Extending it to front-matter would conflate two concerns and require the caller to produce the entire front-matter block (including fields it doesn't want to change). Rejected because partial field updates are the correct granularity for individual mutations.
- **Batch mutation tool (`product frontmatter patch ARTIFACT '{"domains": [...], "scope": "..."}'`)** — JSON patch over front-matter. More flexible than granular tools but harder to validate, harder to document, and produces opaque tool calls that agents and humans both find harder to read. Rejected for MCP use; may be added later for scripting if demand exists.
- **No CLI commands, MCP-only** — the CLI and MCP share a tool surface (ADR-020). Adding MCP tools without CLI equivalents breaks the parity principle and makes the tools untestable from shell scripts. Rejected.