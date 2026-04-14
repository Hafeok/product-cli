## Overview

Validation and graph health checking ensures the knowledge graph remains structurally consistent and complete. The `product graph check` command detects broken links, dependency cycles, orphaned artifacts, missing test coverage, and domain acknowledgement gaps — reporting them in a rustc-style diagnostic format with actionable remediation hints. Exit codes follow a three-tier scheme (ADR-009) so CI pipelines can distinguish errors from warnings and set their own tolerance policy.

## Tutorial

### Your first graph health check

Run the check command against your repository:

```bash
product graph check
```

If the graph is clean, the command exits silently with code 0. If there are problems, you see diagnostics on stderr:

```
error[E002]: broken link
  --> docs/features/FT-003-rdf-projection.md
   |
 4 | adrs: [ADR-001, ADR-002, ADR-099]
   |                          ^^^^^^^ ADR-099 does not exist
   |
   = hint: create the file with `product adr new` or remove the reference

warning[W002]: missing test criteria
  --> docs/features/FT-005-context-assembly.md
   |
   = no test criteria linked to this feature
   = hint: add one with `product test new`
```

### Understanding exit codes

Check the exit code after running:

```bash
product graph check
echo $?
```

| Exit code | Meaning |
|-----------|---------|
| `0` | Clean — no issues found |
| `1` | Errors — broken links, cycles, malformed front-matter |
| `2` | Warnings only — orphans, missing coverage, phase disagreements |

### Fixing a broken link

1. Read the diagnostic — it names the file, line, and the missing artifact ID.
2. Either create the missing artifact (`product adr new` or `product test new`) or remove the stale reference from the front-matter.
3. Re-run `product graph check` to confirm the fix.

### Checking domain coverage

If your feature declares domains, the checker verifies you have considered all relevant ADRs:

```bash
product graph check
```

```
warning[W011]: domain gap without acknowledgement
  --> docs/features/FT-009-rate-limiting.md
   |
   = feature declares domain `security` but neither links security ADRs
   | nor acknowledges the domain
   = hint: add a `domains-acknowledged` block or link the relevant ADRs
```

To resolve this, either link the relevant ADRs in your feature's front-matter or add a `domains-acknowledged` entry with reasoning explaining why no link is needed.

## How-to Guide

### Run graph check in CI

Add `product graph check` as a pipeline step. Use exit codes to control strictness:

1. **Fail on any issue (errors or warnings):**
   ```bash
   product graph check
   ```

2. **Fail on errors only, tolerate warnings:**
   ```bash
   product graph check || [ $? -eq 2 ]
   ```

3. **Get structured JSON output for PR annotations:**
   ```bash
   product graph check --format json
   ```

### Find and fix dependency cycles

1. Run `product graph check` and look for E003 or E004.
2. E003 indicates a cycle in `depends-on` edges between features. E004 indicates a cycle in ADR `supersedes` chains.
3. Open the named files and remove or redirect the circular reference.
4. Re-run the check to confirm resolution.

### Acknowledge a cross-cutting ADR

When `product graph check` reports W010 (unacknowledged cross-cutting ADR):

1. Open your feature's front-matter.
2. Either add the ADR to your `adrs` list, or add a `domains-acknowledged` block:
   ```yaml
   domains-acknowledged:
     error-handling: >
       This feature surfaces errors through the standard error model.
       No new error codes required.
   ```
3. The reasoning field is mandatory — an empty value triggers E011.

### Handle feature abandonment cleanly

1. Run `product feature status FT-XXX abandoned`.
2. Product automatically removes FT-XXX from the `validates.features` list of all linked test criteria.
3. The command prints which TCs were auto-orphaned.
4. Run `product graph check` — orphaned TCs appear as W001 warnings (exit 2), not errors.
5. Decide for each orphaned TC: re-link to another feature or delete.

### Run gap analysis

Gap analysis is separate from `product graph check` and uses its own code series (G001–G007):

```bash
product gap check
```

Gap analysis output goes to stdout (not stderr). It identifies specification-level issues such as testable claims without TCs, missing rejected-alternatives sections, and logical contradictions between linked ADRs.

## Reference

### Command syntax

```
product graph check [--format <FORMAT>]
```

| Flag | Values | Default | Description |
|------|--------|---------|-------------|
| `--format` | `text`, `json` | `text` | Output format. `json` writes structured JSON to stderr for CI consumption. |

### Error codes

| Code | Tier | Severity | Description |
|------|------|----------|-------------|
| E001 | Parse | Error | Malformed YAML front-matter |
| E002 | Graph | Error | Broken link — referenced artifact does not exist |
| E003 | Graph | Error | Dependency cycle in `depends-on` DAG |
| E004 | Graph | Error | Supersession cycle in ADR `supersedes` chain |
| E008 | Schema | Error | `schema-version` in `product.toml` exceeds binary support |
| E011 | Domain | Error | `domains-acknowledged` entry with empty reasoning |
| E012 | Domain | Error | Domain in front-matter not in `product.toml` vocabulary |

### Warning codes

| Code | Tier | Description |
|------|------|-------------|
| W001 | Validation | Orphaned artifact — no incoming feature links |
| W002 | Validation | Feature has no linked test criteria |
| W003 | Validation | Feature has no test of type `exit-criteria` |
| W004 | Validation | Invariant/chaos test missing formal specification blocks |
| W005 | Validation | Phase label disagrees with topological dependency order |
| W006 | Validation | Formal block evidence `δ` below 0.7 |
| W007 | Schema | Schema upgrade available |
| W008 | Migration | ADR status field not found, defaulted to `proposed` |
| W009 | Migration | No test subsection found in ADR, no TC files extracted |
| W010 | Domain | Cross-cutting ADR not linked or acknowledged by a feature |
| W011 | Domain | Feature declares domain with domain-scoped ADRs but no coverage |

### Gap analysis codes

| Code | Severity | Description |
|------|----------|-------------|
| G001 | High | Testable claim in ADR body with no linked TC |
| G002 | High | Formal invariant block with no scenario or chaos TC |
| G003 | Medium | ADR has no rejected alternatives section |
| G004 | Medium | Rationale references undocumented external constraint |
| G005 | High | Logical contradiction between linked ADRs |
| G006 | Medium | Feature aspect not addressed by any linked ADR |
| G007 | Low | Rationale references decisions superseded by a newer ADR |

### Exit codes

| Code | Meaning | Applies to |
|------|---------|------------|
| `0` | Clean — no issues | All commands |
| `1` | Errors found | `graph check`, all commands on failure |
| `2` | Warnings only (no errors) | `graph check` |
| `3` | Internal error (bug in Product) | All commands |

### JSON output format

With `--format json`, structured output is written to stderr:

```json
{
  "errors": [
    {
      "code": "E002",
      "tier": "graph",
      "message": "broken link",
      "file": "docs/features/FT-003-rdf-projection.md",
      "line": 4,
      "context": "adrs: [ADR-001, ADR-002, ADR-099]",
      "detail": "ADR-099 does not exist",
      "hint": "create the file with `product adr new` or remove the reference"
    }
  ],
  "warnings": [],
  "summary": { "errors": 1, "warnings": 0 }
}
```

### Output channels

| Channel | Content |
|---------|---------|
| stderr | All errors, warnings, and diagnostics (both text and JSON modes) |
| stdout | Command output only (context bundles, lists, gap analysis results) |

### Domain vocabulary configuration

Domains are declared in `product.toml`:

```toml
[domains]
security    = "Authentication, authorisation, secrets, trust boundaries"
storage     = "Persistence, durability, volume, block devices, backup"
```

Feature front-matter references these domains and may acknowledge them:

```yaml
domains: [security, api]
domains-acknowledged:
  storage: >
    No persistence required. State is in-memory only.
```

## Explanation

### Why three exit codes instead of two

A binary pass/fail (0/1) cannot express the difference between "your graph has a broken link" and "your graph works but a feature lacks test coverage." CI pipelines need this distinction. A team may choose to block merges on broken links (exit 1) while tolerating coverage gaps (exit 2) during early development. The three-tier scheme from ADR-009 makes this a one-line shell expression rather than output parsing.

### Why rustc-style diagnostics

The diagnostic format — file path, line number, offending content, remediation hint — mirrors `rustc` and `clang`. Engineers already know how to read this format. Every diagnostic includes a hint so the developer knows what action to take without consulting documentation. This is the core UX decision from ADR-013.

### Errors vs. warnings: the design boundary

Errors (exit 1) represent structural problems that make the graph unreliable: broken links mean a referenced artifact is missing, cycles mean dependency ordering is impossible, malformed front-matter means an artifact cannot be parsed. These are objective failures.

Warnings (exit 2) represent completeness gaps that require human judgment: an orphaned test might be intentional (preserved for a future feature), a feature without exit criteria might be in early planning. Blocking CI on these would create false positives that erode trust in the tool.

### Auto-orphaning on feature abandonment

When a feature is abandoned (ADR-010), Product automatically removes it from the `validates.features` list of linked test criteria rather than requiring manual cleanup. This prevents a cascade of false E002 errors from stale links. The orphaned tests are preserved — they document behaviour that was specified, even if it was never built — and surface as W001 warnings so the developer can decide their fate.

### Domain coverage and cross-cutting ADRs

The domain system (ADR-025) solves a discovery problem in large graphs. Without it, finding "all security ADRs" requires reading every ADR. With domains, `product graph check` automatically verifies that features touching a domain have considered the relevant ADRs.

Cross-cutting ADRs (like ADR-013 for error handling) are enforced more strictly: every feature must either link them or explicitly acknowledge them with reasoning. The mandatory reasoning in `domains-acknowledged` is deliberate — an empty acknowledgement is indistinguishable from a checkbox ticked to silence a warning, so it triggers E011. The reasoning also serves as documentation for future authors who need to understand why a domain was scoped out.

### Separation from gap analysis

`product graph check` validates structural consistency — are the links valid, are the cycles absent, is the front-matter well-formed. `product gap check` validates specification completeness — are there testable claims without tests, are there logical contradictions between ADRs. These are distinct concerns with different audiences: graph check is a CI gate, gap analysis is a specification review tool. They use separate code series (E/W vs. G) and separate output channels (stderr vs. stdout).
