---
id: ADR-021
title: Agent Orchestration — `product implement` and `product verify`
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** The implementation loop — assemble context, invoke agent, run tests, update status — is currently manual. A developer runs `product context`, copies the output, opens Claude Code, pastes context, invokes the agent, runs tests manually, and updates front-matter by hand. Each step is error-prone: wrong context depth, forgotten test runs, status not updated, checklist not regenerated. The loop works but it accumulates friction that compounds across every feature.

`product implement` makes the loop a single command. `product verify` makes test-driven status updates automatic.

**Decision:** `product implement FT-XXX` runs a five-step gated pipeline: gap gate, drift check, context assembly, agent invocation, auto-verify. `product verify FT-XXX` reads TC runner configuration from front-matter, executes tests, updates status, and regenerates the checklist. Both commands use the error model from ADR-013. Both write atomically (ADR-015).

---

### `product implement` Pipeline

**Step 1 — Gap gate**

Runs `product gap check FT-XXX --severity high`. If any high-severity findings (G001, G002, G005) are unsuppressed, the command exits 1:

```
error[E009]: implementation blocked by specification gaps
  feature: FT-001 — Cluster Foundation
  
  gap[G001]: "Exactly one leader at all times" has no linked chaos test
  gap[G002]: ⟦Γ:Invariants⟧ in ADR-002 has no scenario TC

  suppress gaps or add TCs before implementing.
  run: product gap check FT-001 --format json
```

The developer must either fix the gaps or suppress them with a reason. This gate enforces that no implementation agent operates on an incomplete specification.

**Step 2 — Drift check**

Runs `product drift check --phase N` where N is the feature's phase. Drift findings are reported as warnings — they do not block implementation. The agent receives a drift summary in the implementation prompt so it is aware of existing divergences.

**Step 3 — Context assembly**

Assembles `product context FT-XXX --depth 2`. Wraps in the versioned implementation prompt. The prompt includes: the context bundle, the current TC status table, hard constraints derived from ADRs, and the `product verify` instruction.

The context bundle and prompt are written to a temp file: `$TMPDIR/product-impl-{feature_id}-{timestamp}.md`. The path is printed to stdout. On `--dry-run`, the pipeline stops here and prints the file path — the developer can inspect the full input before invoking the agent.

**Step 4 — Agent invocation**

```rust
match config.agent.default.as_str() {
    "claude-code" => {
        Command::new("claude")
            .args(["--print", "--context-file", &context_file])
            .args(&config.agent.claude_code.flags)
            .spawn()?
    }
    "custom" => {
        let cmd = config.agent.custom.command
            .replace("{context_file}", &context_file)
            .replace("{feature_id}", feature_id);
        Command::new("sh").args(["-c", &cmd]).spawn()?
    }
    _ => return Err(ProductError::UnknownAgent(...))
}
```

Agent stdout streams directly to the terminal. Agent stderr is captured and written to `$TMPDIR/product-impl-{feature_id}-{timestamp}.log`.

**Step 5 — Auto-verify**

On agent exit (any exit code), runs `product verify FT-XXX`. Pass `--no-verify` to skip.

---

### `product verify` — TC Runner Protocol

TC front-matter includes optional runner configuration:

```yaml
---
id: TC-002
type: scenario
runner: cargo-test
runner-args: ["--test", "raft_leader_election"]
runner-timeout: 60s
---
```

Supported runners:

| Runner | Command template |
|---|---|
| `cargo-test` | `cargo test {runner-args}` in repo root |
| `bash` | `bash {runner-args[0]}` |
| `pytest` | `pytest {runner-args}` |
| `custom` | `{runner-args[0]} {runner-args[1..]}` |

TCs without a `runner` field are reported as `unrunnable` — they are counted separately and do not block a feature from becoming `complete`.

**Status update rules:**
- If all runnable TCs pass → feature status → `complete`
- If any runnable TC fails → feature status → `in-progress` (not regressed to `planned`)
- If all TCs are `unrunnable` → feature status unchanged, warning emitted

After status updates, `product checklist generate` runs automatically.

---

### TC Status Front-Matter

TC status is updated in-place by `product verify`. The `status` field is the only field mutated:

```yaml
# Before verify:
status: unimplemented

# After a passing run:
status: passing
last-run: 2026-04-11T09:14:22Z
last-run-duration: 4.2s

# After a failing run:
status: failing
last-run: 2026-04-11T09:14:22Z
last-run-duration: 1.1s
failure-message: "thread 'raft_leader_election' panicked at..."
```

The `last-run` and `failure-message` fields are added by verify. They are preserved on subsequent writes (unknown fields are never stripped, per ADR-014).

---

**Rationale:**
- The gap gate is a hard gate, not a warning. An agent implementing a feature with G001 gaps will either make up the missing constraints (hallucination) or silently skip them. Neither outcome is acceptable. The gate enforces that the specification is complete before anyone acts on it.
- Drift check is advisory, not a gate. Drift means the existing codebase has diverged from some ADR — but this is information the implementing agent needs, not a reason to block. The agent should be aware of drift and address it.
- Writing context to a temp file rather than piping to the agent enables `--dry-run` inspection, logging (the file persists after the session), and compatibility with agents that accept file paths rather than stdin.
- The `unrunnable` TC status is important for phase 1 when tests are specified but infrastructure to run them does not exist yet. A TC with no runner is not a failure — it is a specification awaiting infrastructure.

**Rejected alternatives:**
- **Pipe context bundle directly to agent via stdin** — no `--dry-run` capability, no log file. Rejected.
- **Gate on drift as well as gaps** — would block implementation whenever any related ADR has drift. Too aggressive: drift is common in active development. It is information, not a blocker.
- **Block feature `complete` if any TC is `unrunnable`** — would make phase 1 features permanently incomplete. The `unrunnable` status exists precisely to handle this case without blocking status progression.