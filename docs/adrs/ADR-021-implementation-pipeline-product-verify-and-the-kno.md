---
id: ADR-021
title: Implementation Pipeline — `product verify` and the Knowledge Boundary
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
---

**Status:** Accepted

**Context:** Earlier versions of this ADR described `product implement FT-XXX` as a command that assembled context, invoked an agent, and ran tests — a full orchestration pipeline. During design review, this was identified as a violation of Product's core responsibility boundary.

Product is a knowledge tool. Its responsibility is to expose everything an agent needs to work on this codebase accurately and safely: the graph, the context bundles, the validation checks, and the verification of outcomes. How agents are invoked, which agent is used, how the context is passed to it, and what happens between context assembly and test verification — these are the developer's choice and the harness's responsibility.

`product implement` as an orchestration command conflates two concerns: knowledge provision (Product's job) and agent lifecycle management (not Product's job). A clean boundary makes Product useful to any agent, any workflow, any harness — including ones that don't exist yet.

**Decision:** Product provides all the knowledge primitives an agent needs. It does not invoke agents. The implementation pipeline is expressed as a sequence of Product commands that a harness calls. `product verify FT-XXX` is the one pipeline command Product owns — it runs test criteria, updates status, and regenerates the checklist. Everything before `product verify` (preflight, gap check, context assembly) is a knowledge command a harness invokes directly. Everything that calls an agent is the harness's responsibility, not Product's.

---

### The Knowledge Boundary

Product's complete implementation-side responsibility:

```bash
# What a harness calls before invoking an agent:
product preflight FT-001              # domain coverage clean?
product gap check FT-001 --severity high  # no blocking spec gaps?
product drift check --phase 1         # no unacknowledged drift?
product context FT-001 --depth 2 --measure  # assemble context bundle

# What a harness calls after the agent completes:
product verify FT-001                 # run TCs, update status, regenerate checklist
product graph check                   # graph still healthy?
```

These commands produce markdown, JSON, or exit codes. Any harness, any agent, any CI system can call them. Product has no knowledge of what happens between `product context` and `product verify`.

---

### `product verify FT-XXX`

`product verify` is the one orchestration-adjacent command Product owns because it is purely a knowledge operation: read TC front-matter, execute test runners, write results back to front-matter, regenerate checklist. No agent involvement.

**The runner boundary.** Product's responsibility in `product verify` is exactly: call the configured command, wait for exit, record the result. Everything inside that command — setup, teardown, fixture management, test ordering, environment variables, database state, cluster initialisation — is the test suite's responsibility. Product never models test infrastructure. The moment Product starts answering "what must be true before this test runs?" it is building a test framework. That is a different product.

The escape hatch for any TC that requires environment preparation is a wrapper script. The wrapper handles setup and teardown internally and exits with the test result. Product calls the wrapper as it would call any command:

```yaml
---
id: TC-002
type: scenario
runner: bash
runner-args: ["scripts/test-harness/raft_leader_election.sh"]
runner-timeout: 120s
---
```

```bash
#!/usr/bin/env bash
# scripts/test-harness/raft_leader_election.sh
# Setup, test, teardown — entirely this script's responsibility.
set -euo pipefail

# Setup
./scripts/cluster-init.sh 2-nodes
trap './scripts/cluster-teardown.sh' EXIT

# Test
cargo test --test raft_leader_election

# Teardown runs via trap
```

Product calls `bash scripts/test-harness/raft_leader_election.sh`, waits, reads the exit code. It knows nothing about what happens inside.

TC front-matter fields:

```yaml
---
id: TC-002
type: scenario
runner: cargo-test           # cargo-test | bash | pytest | custom
runner-args: ["--test", "raft_leader_election"]
runner-timeout: 60s          # optional, default 30s
requires: [binary-compiled]  # optional — declarative prerequisites (see below)
---
```

Supported runners:

| Runner | Command template |
|---|---|
| `cargo-test` | `cargo test {runner-args}` in repo root |
| `bash` | `bash {runner-args[0]} {runner-args[1..]}` |
| `pytest` | `pytest {runner-args}` |
| `custom` | `{runner-args[0]} {runner-args[1..]}` |

**The `requires` field — declarative prerequisites.**

Some TCs are not runnable until something else is true: the binary compiles, a two-node cluster is available, a particular phase is complete. The `requires` field declares this as a signal — it does not make the prerequisite true. Product reads `requires` to determine whether a TC is runnable in the current context and reports it as `unrunnable` with a reason if the prerequisite is not met.

```yaml
requires: [binary-compiled, two-node-cluster]
```

Prerequisites are declared in `product.toml` as checkable conditions:

```toml
[verify.prerequisites]
binary-compiled    = "test -f target/release/picloud"
two-node-cluster   = "product graph query 'ASK { ?n a picloud:Node } HAVING COUNT(?n) >= 2'"
raft-leader-elected = "product graph query 'ASK { ?n picloud:hasRole picloud:Leader }'"
```

Each prerequisite is a shell command. Exit code 0 = satisfied. Exit code non-zero = not satisfied. Product evaluates prerequisites before attempting to run the TC. If any prerequisite fails, the TC is marked `unrunnable` with the prerequisite name in `failure-message`:

```yaml
status: unrunnable
failure-message: "prerequisite 'two-node-cluster' not satisfied"
```

This is the entire scope of Product's involvement with test infrastructure: check a declared condition, report the result. Never satisfy the condition. Never manage state. Never set up or tear down anything.

TCs without a `runner` field are always `unrunnable`. TCs with a `runner` but failing `requires` are `unrunnable`. Both are counted separately and do not block feature completion.

**Status update rules:**
- All runnable TCs pass → feature status → `complete`
- Any runnable TC fails → feature status → `in-progress`
- All TCs unrunnable → feature status unchanged, W-class warning

After status updates, `product checklist generate` runs automatically.

**TC status fields written by verify:**

```yaml
status: passing
last-run: 2026-04-11T09:14:22Z
last-run-duration: 4.2s

# On failure:
status: failing
last-run: 2026-04-11T09:14:22Z
failure-message: "thread 'raft_leader_election' panicked at..."

# On unrunnable:
status: unrunnable
failure-message: "prerequisite 'two-node-cluster' not satisfied"
```

**`product verify --platform`** runs all TCs linked to cross-cutting ADRs, regardless of feature association.

---

### Example Harness Scripts

Product ships example shell scripts in `scripts/harness/`. These are not part of the CLI — they are reference implementations a developer can copy, modify, or discard. They demonstrate how the knowledge commands compose into a complete implementation flow.

**`scripts/harness/implement.sh`:**

```bash
#!/usr/bin/env bash
# Example implementation harness. Copy and modify for your workflow.
# Product is a knowledge tool — this script is not part of Product.
set -euo pipefail

FEATURE=${1:?Usage: implement.sh FT-XXX}

echo "=== Pre-flight ==="
product preflight "$FEATURE" || {
  echo "Pre-flight failed. Run: product preflight $FEATURE"
  exit 1
}

echo "=== Gap check ==="
product gap check "$FEATURE" --severity high --format json | tee /tmp/gaps.json
if jq -e '.findings | length > 0' /tmp/gaps.json > /dev/null; then
  echo "High-severity gaps found. Resolve before implementing."
  exit 1
fi

echo "=== Drift check ==="
product drift check --phase "$(product feature show "$FEATURE" --field phase)"
# Drift is advisory — continue regardless

echo "=== Context bundle ==="
BUNDLE_FILE=$(mktemp /tmp/product-context-XXXX.md)
product context "$FEATURE" --depth 2 --measure > "$BUNDLE_FILE"
echo "Bundle written to: $BUNDLE_FILE"

echo "=== Agent invocation ==="
# Replace this with your agent of choice:
#   claude --print --context-file "$BUNDLE_FILE"
#   cursor --context "$BUNDLE_FILE"
#   cat "$BUNDLE_FILE" | your-agent
echo "Pass $BUNDLE_FILE to your agent, then run:"
echo "  product verify $FEATURE"
```

**`scripts/harness/author.sh`:**

```bash
#!/usr/bin/env bash
# Example authoring harness. Copy and modify for your workflow.
# Loads the appropriate system prompt and starts Product MCP.
set -euo pipefail

SESSION_TYPE=${1:?Usage: author.sh feature|adr|review}
PROMPTS_DIR=${PRODUCT_PROMPTS_DIR:-"$(product config get paths.prompts)"}
PROMPT_FILE="$PROMPTS_DIR/author-${SESSION_TYPE}-v1.md"

if [ ! -f "$PROMPT_FILE" ]; then
  echo "Prompt file not found: $PROMPT_FILE"
  echo "Run: product prompts init"
  exit 1
fi

echo "System prompt: $PROMPT_FILE"
echo "Product MCP: stdio (Claude Code will connect automatically)"
echo ""
echo "Open Claude Code in this directory. The .mcp.json will load Product MCP."
echo "Paste the contents of $PROMPT_FILE as your first message or system prompt."
echo ""
echo "When complete, run: product graph check && product gap check --changed"
```

These scripts make the composition explicit and learnable without Product owning the composition.

---

**Rationale:**
- The runner boundary is the critical design decision for `product verify`. CI runners (GitHub Actions, Jenkins, CircleCI) don't manage test fixtures — they call commands and read exit codes. Product's verify command has exactly the same responsibility. The moment Product models setup/teardown, it becomes a test framework. The wrapper script pattern preserves the boundary: the developer writes a script that handles environment management internally; Product calls it as a black box.
- The `requires` field exists because without it, TCs that genuinely cannot run in the current environment produce false failures rather than honest `unrunnable` status. A TC that requires a two-node cluster cannot pass in a single-binary CI build. Marking it as `unrunnable` with a clear reason is more honest than a misleading failure. Crucially, Product evaluates `requires` conditions but never satisfies them — evaluation is a read operation; satisfaction would be infrastructure management.
- Prerequisites as shell commands in `product.toml` are the right model. They are: declarative (the developer describes what must be true), checkable (Product can evaluate them with a subprocess call), and external (the shell command can call any tool the developer controls). Product does not need to understand what the prerequisite means — only whether it is satisfied.
- The boundary is `product verify`. Everything before it (preflight, gap check, context assembly) is graph knowledge. Everything after it (agent work) is the harness's domain. `product verify` is on the Product side because it writes back to the graph — TC status, feature status, checklist — which are Product-owned artifacts.
- Example harness scripts in `scripts/harness/` solve the discoverability problem without coupling. A developer opening the repo for the first time can read `implement.sh` and immediately understand how the commands compose. Product's correctness does not depend on the scripts.

**Rejected alternatives:**
- **`product implement` as an orchestration command** — conflates knowledge provision with agent lifecycle. Rejected (see context above).
- **Product manages test setup/teardown** — as soon as Product tries to satisfy prerequisites rather than just check them, it needs environment management, state machines, rollback on failure, and test isolation. That is a test framework. Rejected: wrapper scripts are the correct escape hatch.
- **`requires` as imperative setup steps** — `requires: [run: "./cluster-init.sh"]` style. Product would execute these before running the TC. This is Product doing setup. Rejected: declarative condition checks only.
- **No `requires` field** — TCs that cannot run in the current environment fail or are skipped arbitrarily. The `unrunnable` status with a named prerequisite produces honest, debuggable output. Rejected as insufficient.
- **No example scripts** — leaves developers without guidance on how commands compose. Rejected as insufficient.
- **Scripts inside the CLI binary** — harness logic in the binary. Rejected: scripts belong in the repo, not the binary.