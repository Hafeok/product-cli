## Overview

Test Criteria are the formal specification layer of the Product knowledge graph. Each test criterion (TC-XXX) defines a verifiable assertion — a constraint, scenario, or exit condition — that a feature or ADR must satisfy. Product supports a hybrid format: YAML front-matter for graph metadata, optional AISP-influenced formal blocks for machine-precise constraints, and prose for human-readable descriptions. Formal blocks eliminate interpretation variance when LLM agents consume context bundles, ensuring that invariants like "exactly one leader" are expressed unambiguously rather than left to natural language parsing.

## Tutorial

### Step 1: Inspect an existing test criterion

Open a test criterion file to see the hybrid format in action:

```bash
cat docs/tests/TC-002-raft-leader-election.md
```

You will see three layers:

1. **YAML front-matter** — graph metadata (`id`, `type`, `status`, `validates`)
2. **Prose description** — human-readable explanation
3. **Formal blocks** — typed, symbolic specification consumed by agents

### Step 2: Create a new test criterion

Scaffold a new criterion linked to a feature:

```bash
product test new --id TC-100 --title "My new scenario" --type scenario --feature FT-015
```

This creates a file in `docs/tests/` with front-matter stubs and empty formal block placeholders.

### Step 3: Add formal blocks

Edit the generated file and add a formal specification section. For a scenario-type criterion, use the `given/when/then` pattern:

```markdown
## Formal Specification

⟦Σ:Types⟧{
  Input≜String
  Output≜Result|Error
}

⟦Λ:Scenario⟧{
  given≜valid_input("hello")
  when≜process(input)
  then≜output = Result ∧ output.value = "HELLO"
}

⟦Ε⟧⟨δ≜0.90;φ≜100;τ≜◊⁺⟩
```

### Step 4: Validate the formal blocks

Run the graph checker to confirm your blocks parse correctly:

```bash
product graph check
```

If a block has a syntax error, you will see an E001 error with the file path and line number. If you left a block body empty, you will see a W004 warning.

### Step 5: Verify the criterion passes

Add `runner` and `runner-args` to the criterion's front-matter so `product verify` can execute it:

```yaml
runner: cargo-test
runner-args: "tc_100_my_new_scenario"
```

Then run:

```bash
product verify FT-015
```

### Step 6: Check formal coverage

View how many invariant and chaos criteria have formal blocks:

```bash
product graph stats
```

The `phi` (φ) metric reports the percentage of criteria with formal blocks present.

## How-to Guide

### Add formal blocks to an existing prose-only criterion

1. Open the TC file in `docs/tests/`.
2. Add a `## Formal Specification` section after the prose description.
3. Add the appropriate block types for the criterion's type:
   - **invariant/chaos**: `⟦Σ:Types⟧` and `⟦Γ:Invariants⟧` are required.
   - **scenario**: `⟦Λ:Scenario⟧` with `given/when/then` fields is recommended.
   - **exit-criteria**: `⟦Λ:ExitCriteria⟧` with measurable thresholds is recommended.
4. Add an evidence block `⟦Ε⟧` with confidence, coverage, and stability values.
5. Run `product graph check` to validate.

### Configure a TC for automated verification

1. Write the integration test function in `tests/integration.rs`:
   ```rust
   #[test]
   fn tc_054_product_impact_adr_001() {
       // test body
   }
   ```
2. Add runner fields to the TC's YAML front-matter:
   ```yaml
   runner: cargo-test
   runner-args: "tc_054_product_impact_adr_001"
   ```
3. The `runner-args` value must match the test function name exactly.
4. Run `product verify FT-XXX` to execute the runner and update status.

### Check for missing formal blocks

```bash
product graph check
```

Criteria of type `invariant` or `chaos` without formal blocks produce a warning (exit code 2, not an error). Use this to find criteria that need formal specification added.

### Inspect how formal blocks appear in context bundles

```bash
product context FT-015 --depth 2
```

Formal blocks are preserved verbatim in the bundle output. YAML front-matter is stripped, but the symbolic notation passes through byte-for-byte so agents receive the exact specification the author wrote.

## Reference

### Test criterion types

| Type | Description | Required formal blocks |
|------|-------------|----------------------|
| `scenario` | Given/when/then test flow | `⟦Λ:Scenario⟧` (optional) |
| `invariant` | Constraint that must always hold | `⟦Σ:Types⟧`, `⟦Γ:Invariants⟧` (required) |
| `chaos` | Fault-injection resilience test | `⟦Σ:Types⟧`, `⟦Γ:Invariants⟧` (required) |
| `exit-criteria` | Measurable pass/fail thresholds | `⟦Λ:ExitCriteria⟧` (optional) |

### Formal block types

| Block | Symbol | Purpose |
|-------|--------|---------|
| `⟦Σ:Types⟧` | Type definitions | Name domain types used in rules |
| `⟦Γ:Invariants⟧` | Constraint rules | Formal assertions that must hold |
| `⟦Λ:Scenario⟧` | Given/when/then | Structured test flow |
| `⟦Λ:ExitCriteria⟧` | Measurable thresholds | Numeric pass/fail bounds |
| `⟦Λ:Benchmark⟧` | Quality measurement | Conditions, scorer, pass threshold |
| `⟦Ε⟧` | Evidence block | Confidence, coverage, stability |

### Evidence block fields

| Field | Symbol | Meaning | Range |
|-------|--------|---------|-------|
| Delta | `δ` | Specification confidence | 0.0–1.0 |
| Phi | `φ` | Coverage completeness | 0–100 (%) |
| Tau | `τ` | Stability signal | `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown) |

### Symbol subset

| Symbol | Meaning |
|--------|---------|
| `≜` | Definition ("is defined as") |
| `≔` | Assignment |
| `∀` | For all (universal quantifier) |
| `∃` | There exists (existential quantifier) |
| `∧` | Logical AND |
| `∨` | Logical OR |
| `→` | Function type or implication |
| `⟨⟩` | Tuple or record delimiters |
| `\|` | Union type separator |
| `⟦⟧` | Block delimiters |

### TC front-matter fields

```yaml
---
id: TC-XXX
title: descriptive title
type: scenario | invariant | chaos | exit-criteria
status: unimplemented | passing | failing
validates:
  features: [FT-XXX]
  adrs: [ADR-XXX]
phase: 1
runner: cargo-test
runner-args: "tc_xxx_snake_case_title"
---
```

### Error and warning codes

| Code | Trigger | Severity |
|------|---------|----------|
| E001 | Malformed formal block (unclosed delimiter, bad expression, unrecognised block type, evidence value out of range) | Error |
| W004 | Empty block body (e.g., `⟦Γ:Invariants⟧{}`) | Warning |
| W004 | `invariant` or `chaos` criterion missing formal blocks | Warning |

### Parse behaviour

- Unclosed `⟦` or unrecognised block type: E001, file cannot be processed further.
- Malformed content inside a block: E001 on the specific line; subsequent blocks in the same file are still parsed.
- Empty block body: W004 warning, no error.
- Evidence `δ` outside [0.0, 1.0] or `φ` outside [0, 100]: E001.
- `Invariant.raw` is preserved byte-for-byte, including whitespace.

## Explanation

### Why formal notation instead of prose?

Test criteria are consumed primarily by LLM implementation agents via context bundles. Natural language descriptions of constraints — such as "exactly one node holds the Leader role" — have measurable interpretation variance across agent invocations. The same prose can lead to subtly different invariant checks in generated code. AISP-influenced formal blocks (`∀s:ClusterState: |{n | roles(n)=Leader}| = 1`) eliminate this ambiguity by encoding constraints as typed symbolic expressions with formal semantics.

### Why a hybrid format?

The formal blocks are additive, not a replacement for prose. Human readers enter through the prose description; agent consumers use the formal blocks. This avoids forcing contributors to learn the full notation just to understand what a test criterion checks, while giving agents the precision they need. See ADR-011 for the full rationale.

### Why a minimal symbol subset?

Product uses only the symbols needed for the constraint patterns that actually appear in the project (quantifiers, logical connectives, type definitions, tuple notation). Full AISP 5.1 includes category theory constructs and tri-vector decomposition that exceed what test criteria require. The minimal subset keeps files readable to contributors unfamiliar with the full AISP spec. See ADR-011.

### How are formal blocks parsed?

A hand-written recursive descent parser produces a typed AST (defined in ADR-016). The parser validates structure (delimiter matching, field presence, evidence ranges) but is intentionally permissive on expressions — it stores unparseable expressions as raw strings rather than rejecting them. This reflects Product's role as a context assembly tool, not a formal verifier. The raw text is preserved byte-for-byte alongside the AST so that context bundle output matches exactly what the author wrote.

### How does formal coverage tracking work?

The evidence block's `φ` field and `product graph stats` together track formal specification coverage. When `product context` assembles a bundle, the top-level evidence block computes `φ` as the percentage of linked criteria that have formal blocks present. This gives agents receiving the bundle a quick quality signal before reading the full content. A bundle with `φ≜41` tells the agent that only 41% of linked criteria have formal specifications — the rest are prose-only.

### Relationship to testing strategy

Formal blocks define *what* must be verified. The testing strategy (ADR-018) defines *how* verification is executed across three complementary approaches: property-based tests for algorithmic correctness, integration tests for CLI behaviour, and LLM benchmarks for context bundle quality. Test criteria with formal blocks feed into all three — property tests can encode formal invariants directly, integration tests verify CLI handling of formal block parsing, and benchmark rubrics derive their criteria from formal exit conditions.
