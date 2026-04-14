## Overview

Agent Orchestration automates the spec-to-implementation loop. Instead of manually assembling context, invoking an LLM agent, running tests, and updating statuses, two commands handle the entire workflow: `product implement` runs a gated five-step pipeline (gap check, drift check, context assembly, agent invocation, auto-verify), and `product verify` executes test criteria and updates artifact statuses. Together they close the loop between specification and working code with a single command.

## Tutorial

### Your first agent-driven implementation

This walkthrough takes you from a specified feature to a verified implementation using `product implement`.

1. Pick a feature to implement. List features that are not yet complete:

   ```bash
   product checklist generate
   ```

   Choose a feature marked `[ ]` (planned) or partially complete — for example, `FT-001`.

2. Check that the specification is ready. The implement pipeline will do this automatically, but you can preview:

   ```bash
   product gap check FT-001
   ```

   If high-severity gaps appear (G001, G002, G005), fix them before proceeding — `product implement` will refuse to continue.

3. Run the implementation pipeline:

   ```bash
   product implement FT-001
   ```

   You will see output like:

   ```
   ✓ Gap check: no high-severity gaps
   ✓ Drift check: no drift detected
   → Assembling context bundle (FT-001, 4 ADRs, 6 TCs, depth 2)
   → Invoking claude-code...
   [agent output streams here]
   → Running product verify FT-001...
   TC-001 binary-compiles         PASS
   TC-002 raft-leader-election    PASS
   TC-003 raft-leader-failover    FAIL
   ✗ 1 test failing. Feature status: in-progress
   ```

4. If any tests fail, fix the issue and re-verify:

   ```bash
   product verify FT-001
   ```

5. Once all TCs pass, the feature status updates to `complete` and the checklist regenerates automatically.

### Previewing the context before invoking an agent

Use `--dry-run` to inspect exactly what the agent will receive:

```bash
product implement FT-001 --dry-run
```

This runs the gap gate and drift check, assembles the context bundle, writes it to a temp file, and prints the file path — but does not invoke the agent. Open the file to review the full prompt.

## How-to Guide

### Implement a feature end-to-end

1. Run `product implement FT-XXX`.
2. The pipeline checks for gaps and drift, assembles context, invokes the agent, and verifies.
3. If tests fail, fix the code and run `product verify FT-XXX` until all TCs pass.

### Skip auto-verify after implementation

If you want to review the agent's output before running tests:

1. Run `product implement FT-XXX --no-verify`.
2. Review the changes.
3. Run `product verify FT-XXX` when ready.

### Verify a feature manually

1. Ensure each TC linked to the feature has `runner` and `runner-args` in its YAML front-matter.
2. Run `product verify FT-XXX`.
3. Check the output for pass/fail results. TC statuses and feature status are updated automatically.

### Configure TC runners

Add `runner` and `runner-args` fields to each TC's YAML front-matter:

```yaml
---
id: TC-002
type: scenario
runner: cargo-test
runner-args: ["--test", "raft_leader_election"]
runner-timeout: 60s
---
```

Without a `runner` field, the TC is reported as `unrunnable` and skipped during verification.

### Unblock an implementation blocked by gaps

If `product implement` exits with E009 (specification gaps):

1. Read the error output to identify the gaps (e.g., G001, G002).
2. Either add the missing TCs or suppress the gaps with a reason.
3. Re-run `product implement FT-XXX`.

### Use a custom agent instead of Claude Code

Configure the agent in `product.toml`:

```toml
[agent]
default = "custom"

[agent.custom]
command = "my-agent --input {context_file} --feature {feature_id}"
```

The `{context_file}` and `{feature_id}` placeholders are replaced at invocation time.

## Reference

### `product implement`

```
product implement <FEATURE_ID> [OPTIONS]
```

Runs the five-step implementation pipeline: gap gate, drift check, context assembly, agent invocation, auto-verify.

| Flag / Option | Description |
|---|---|
| `<FEATURE_ID>` | Required. The feature to implement (e.g., `FT-001`). |
| `--no-verify` | Skip the auto-verify step after agent completion. |
| `--dry-run` | Stop after context assembly. Prints the temp file path without invoking the agent. |

**Pipeline steps:**

| Step | Action | Blocking? |
|---|---|---|
| 1. Gap gate | `product gap check FT-XXX` | Yes — exits 1 on high-severity gaps (G001, G002, G005) |
| 2. Drift check | `product drift check --phase N` | No — drift is reported as a warning |
| 3. Context assembly | `product context FT-XXX --depth 2` wrapped in implementation prompt | N/A |
| 4. Agent invocation | Invokes configured agent with context file | N/A |
| 5. Auto-verify | `product verify FT-XXX` | Skipped if `--no-verify` |

**Temp files:**

- Context bundle: `$TMPDIR/product-impl-{feature_id}-{timestamp}.md`
- Agent stderr log: `$TMPDIR/product-impl-{feature_id}-{timestamp}.log`

**Exit codes:**

| Code | Meaning |
|---|---|
| 0 | Pipeline completed, all TCs pass |
| 1 | Gap gate blocked implementation (E009) |
| Non-zero | Agent or verify failure |

### `product verify`

```
product verify <FEATURE_ID>
```

Reads linked TC files, executes configured runners, and updates statuses.

| Flag / Option | Description |
|---|---|
| `<FEATURE_ID>` | Required. The feature to verify (e.g., `FT-001`). |

**Supported runners:**

| Runner | Command template |
|---|---|
| `cargo-test` | `cargo test {runner-args}` in repo root |
| `bash` | `bash {runner-args[0]}` |
| `pytest` | `pytest {runner-args}` |
| `custom` | `{runner-args[0]} {runner-args[1..]}` |

**TC runner front-matter fields:**

| Field | Type | Required | Description |
|---|---|---|---|
| `runner` | string | No | Runner type. If absent, TC is `unrunnable`. |
| `runner-args` | string or list | No | Arguments passed to the runner. |
| `runner-timeout` | duration | No | Maximum execution time (e.g., `60s`). |

**Status update rules:**

| Condition | Feature status |
|---|---|
| All runnable TCs pass | `complete` |
| Any runnable TC fails | `in-progress` |
| All TCs are `unrunnable` | Unchanged (warning emitted) |

**Fields written to TC front-matter after verify:**

| Field | Written when |
|---|---|
| `status` | Always (`passing`, `failing`, or `unrunnable`) |
| `last-run` | After any run (ISO 8601 timestamp) |
| `last-run-duration` | After any run |
| `failure-message` | After a failing run |

After updating statuses, `product checklist generate` runs automatically.

### Agent configuration in `product.toml`

```toml
[agent]
default = "claude-code"   # or "custom"

[agent.claude_code]
flags = ["--print"]       # additional flags passed to claude

[agent.custom]
command = "my-agent --input {context_file} --feature {feature_id}"
```

**Placeholders for custom agents:**

| Placeholder | Replaced with |
|---|---|
| `{context_file}` | Path to the assembled context temp file |
| `{feature_id}` | The feature ID (e.g., `FT-001`) |

## Explanation

### Why the gap gate is a hard block

The gap gate (Step 1) is intentionally a hard gate, not a warning. When an LLM agent encounters a specification with missing constraints — such as an invariant with no linked test (G001) — it will either invent the missing constraints (hallucination) or silently skip them. Neither outcome is acceptable in a specification-driven workflow. By blocking implementation until the specification is complete, the pipeline ensures the agent always works from a full contract. This is a deliberate trade-off: it adds friction when gaps exist, but eliminates an entire class of agent failure modes.

### Why drift check is advisory

Unlike the gap gate, drift detection (Step 2) does not block the pipeline. Drift means the codebase has diverged from a related ADR, which is common during active development. The implementing agent receives a drift summary in its context so it can account for existing divergences, but blocking on drift would be too aggressive — it would halt implementation whenever any related ADR has unresolved drift, which is routine in multi-phase projects.

### Why context is written to a temp file

The pipeline writes the assembled context to a temp file rather than piping it directly to the agent. This enables three capabilities: `--dry-run` inspection (developers can review exactly what the agent will see), persistent logging (the file survives the session for debugging), and compatibility with agents that accept file paths rather than stdin. See ADR-021 for the full rationale.

### The `unrunnable` TC status

TCs without a `runner` field are classified as `unrunnable` rather than `failing`. This distinction matters in early project phases where test criteria are specified before the infrastructure to run them exists. An `unrunnable` TC does not block a feature from reaching `complete` status — it represents a specification awaiting infrastructure, not a failure. This prevents early-phase features from being permanently incomplete. See ADR-021.

### Relationship to other commands

`product implement` composes several existing Product commands into a single pipeline. It calls `product gap check`, `product drift check`, `product context`, and `product verify` internally. When using `product implement`, do not also run `product context` separately — the pipeline handles context assembly automatically, and running it twice would duplicate the work.
