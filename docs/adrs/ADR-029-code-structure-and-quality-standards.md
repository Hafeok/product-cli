---
id: ADR-029
title: Code Structure and Quality Standards
status: accepted
features: []
supersedes: []
superseded-by: []
domains: []
scope: domain
content-hash: sha256:71bb577274866b798c526d742fdbde6b4b34ca330ebc8b84492bd861bc782e0f
---

**Status:** Accepted

**Context:** Product is designed for LLM-driven implementation. Files with 1600+ lines of mixed concerns are already appearing before implementation is complete. This is a problem on two dimensions simultaneously.

For human contributors, large files indicate poor cohesion — the file is doing more than one thing. For LLM agents, large files are a context problem. When implementing a feature that touches `graph.rs`, the agent receives the full file — traversal algorithms, builder logic, centrality computation, impact analysis — most of which is irrelevant to the specific change. The agent makes assumptions about the whole file from the parts it can see clearly. This produces unnecessary changes, incorrect assumptions, and implementations that work in isolation but break adjacent behaviour.

The Product spec already has vocabulary for this problem in a different domain: a feature with `depth-1-adrs > 8` is a signal to split. A 1600-line source file is the implementation equivalent. The same principle applies: bounded scope enables accurate context assembly.

ADR-001 covers compilation quality (`#![deny(clippy::unwrap_used)]`). This ADR covers structural quality — how the codebase is organised and what limits are enforced.

**Decision:** Enforce four structural quality rules with measurable thresholds, checked by TC files that run on `product verify --platform`. Rules are enforced by CI scripts rather than custom lints — they must be auditable by reading the script, not by understanding a lint framework.

---

### Rule 1: File Size Limit

No Rust source file in `src/` may exceed **400 lines** (blank lines and comments included). The 400-line limit is a hard gate — CI fails. A secondary warning threshold of **300 lines** produces a warning but does not fail CI.

The limit applies to `src/**/*.rs` only. Test files in `tests/` are exempt — integration test scenarios are necessarily verbose. Benchmark files in `benches/` are exempt.

**Why 400, not 500 or 200?**

200 is too tight for Rust — a module with a substantial type definition, its `impl` blocks, and its error types legitimately reaches 200 lines. 500 is too loose — it permits files that clearly have multiple responsibilities. 400 is the point at which most single-responsibility Rust modules fit comfortably.

**Enforcement script (`scripts/checks/file-length.sh`):**

```bash
#!/usr/bin/env bash
# scripts/checks/file-length.sh
# Checks Rust source file lengths.
# Exit 0: all files within limits
# Exit 1: one or more files exceed hard limit (400 lines)
# Exit 2: one or more files exceed warning threshold (300 lines), none exceed hard limit
set -euo pipefail

HARD_LIMIT=${FILE_LENGTH_HARD:-400}
WARN_LIMIT=${FILE_LENGTH_WARN:-300}

HARD_VIOLATIONS=$(find src -name "*.rs" \
  | xargs wc -l \
  | awk -v limit="$HARD_LIMIT" '$1 > limit && $2 != "total" {print $1, $2}' \
  | sort -rn)

WARN_VIOLATIONS=$(find src -name "*.rs" \
  | xargs wc -l \
  | awk -v wl="$WARN_LIMIT" -v hl="$HARD_LIMIT" \
    '$1 > wl && $1 <= hl && $2 != "total" {print $1, $2}' \
  | sort -rn)

if [ -n "$HARD_VIOLATIONS" ]; then
  echo "ERROR: files exceeding hard limit ($HARD_LIMIT lines):"
  echo "$HARD_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (limit: $HARD_LIMIT)"
  done
  exit 1
fi

if [ -n "$WARN_VIOLATIONS" ]; then
  echo "WARNING: files approaching limit ($WARN_LIMIT–$HARD_LIMIT lines):"
  echo "$WARN_VIOLATIONS" | while read -r count file; do
    echo "  $file: $count lines (warn at: $WARN_LIMIT)"
  done
  exit 2
fi

echo "OK: all source files within limits"
exit 0
```

---

### Rule 2: Function Length Limit

No function body may exceed **40 lines** (blank lines excluded from the count — only statement lines count). Trait `impl` blocks may be longer but each individual method within them must respect the 40-line limit.

**Why 40?** A function that exceeds 40 statement lines is almost always doing more than one thing. The fix is always the same: name the sub-operation and extract it. The name is documentation. The extraction is a seam for testing.

**Enforcement script (`scripts/checks/function-length.sh`):**

```bash
#!/usr/bin/env bash
# scripts/checks/function-length.sh
# Uses ripgrep to find fn definitions, then counts statement lines until closing brace.
# Requires: rg (ripgrep), awk
set -euo pipefail

HARD_LIMIT=${FN_LENGTH_HARD:-40}
WARN_LIMIT=${FN_LENGTH_WARN:-30}
VIOLATIONS=0
WARNINGS=0

# For each .rs file, use awk to find fn blocks and count statement lines
find src -name "*.rs" | while read -r file; do
  awk -v hard="$HARD_LIMIT" -v warn="$WARN_LIMIT" -v fname="$file" '
    /^[[:space:]]*(pub |pub\(.*\) |async |pub async )*fn / {
      fn_name = $0
      fn_line = NR
      brace_depth = 0
      stmt_count = 0
      in_fn = 1
    }
    in_fn {
      # Count opening braces
      n = split($0, chars, "")
      for (i = 1; i <= n; i++) {
        if (chars[i] == "{") brace_depth++
        if (chars[i] == "}") brace_depth--
      }
      # Count non-blank, non-brace-only lines as statements
      stripped = $0
      gsub(/^[[:space:]]+/, "", stripped)
      gsub(/[[:space:]]+$/, "", stripped)
      if (length(stripped) > 0 && stripped != "{" && stripped != "}") {
        stmt_count++
      }
      if (brace_depth == 0 && fn_line != NR) {
        if (stmt_count > hard) {
          print "ERROR: " fname ":" fn_line ": function has " stmt_count \
                " statement lines (limit: " hard ")"
        } else if (stmt_count > warn) {
          print "WARN: " fname ":" fn_line ": function has " stmt_count \
                " statement lines (warn at: " warn ")"
        }
        in_fn = 0
        stmt_count = 0
      }
    }
  ' "$file"
done | tee /tmp/fn-length-results.txt

if grep -q "^ERROR:" /tmp/fn-length-results.txt; then
  exit 1
elif grep -q "^WARN:" /tmp/fn-length-results.txt; then
  exit 2
fi
exit 0
```

---

### Rule 3: Module Decomposition

The `src/` directory follows a mandatory module structure. Each module has a single stated responsibility. A file may not import from a sibling module's internal submodules — only from its public surface (`mod.rs` re-exports).

**Canonical module structure:**

```
src/
  main.rs           # CLI entry point only — no logic, only clap dispatch
  error.rs          # ProductError type and Display impl (ADR-013)
  config.rs         # product.toml parsing and ProductConfig type

  graph/            # in-memory graph: construction, traversal, algorithms
    mod.rs           # re-exports Graph, GraphBuilder
    builder.rs       # front-matter → in-memory graph
    topo.rs          # Kahn's topological sort
    bfs.rs           # BFS traversal with depth and deduplication
    centrality.rs    # Brandes' betweenness centrality
    impact.rs        # reverse-graph reachability
    coverage.rs      # feature × domain coverage matrix

  parse/            # all parsing: front-matter, formal blocks, TOML
    mod.rs
    frontmatter.rs   # YAML front-matter → typed structs
    formal.rs        # AISP formal block parser
    grammar.rs       # grammar AST types

  context/          # context bundle assembly and measurement
    mod.rs
    bundle.rs        # bundle assembly, ordering, dedup
    measure.rs       # token counting, bundle metrics
    failures.rs      # --with-failures flag: TC status → failure context

  commands/         # one file per command group, no logic — delegates to modules
    mod.rs
    feature.rs
    adr.rs
    test.rs
    graph.rs
    context.rs
    gap.rs
    drift.rs
    metrics.rs
    verify.rs
    mcp.rs
    prompts.rs
    migrate.rs
    preflight.rs

  verify/           # product verify implementation
    mod.rs
    runner.rs        # TC runner execution
    prereqs.rs       # prerequisite checking
    status.rs        # TC and feature status update

  mcp/              # MCP server: both transports, tool registry
    mod.rs
    stdio.rs
    http.rs
    registry.rs
    tools/           # one file per tool group, mirrors commands/
      mod.rs
      read.rs
      write.rs

  io/               # file system operations
    mod.rs
    write.rs         # atomic writes (ADR-015)
    lock.rs          # advisory locking (ADR-015)
```

`main.rs` must contain only: the `clap` derive macro, the top-level `match` dispatching to `commands/`, and the call to `std::process::exit`. No logic. If `main.rs` exceeds 80 lines, it is a violation.

**Enforcement script (`scripts/checks/module-structure.sh`):**

```bash
#!/usr/bin/env bash
# scripts/checks/module-structure.sh
# Checks that required top-level modules exist and main.rs is within limits.
set -euo pipefail

REQUIRED_MODULES=(graph parse context commands verify mcp io)
MISSING=()

for mod in "${REQUIRED_MODULES[@]}"; do
  if [ ! -d "src/$mod" ]; then
    MISSING+=("src/$mod/")
  fi
done

if [ ${#MISSING[@]} -gt 0 ]; then
  echo "ERROR: missing required modules:"
  for m in "${MISSING[@]}"; do echo "  $m"; done
  exit 1
fi

MAIN_LINES=$(wc -l < src/main.rs)
if [ "$MAIN_LINES" -gt 80 ]; then
  echo "ERROR: src/main.rs has $MAIN_LINES lines (limit: 80)"
  echo "  main.rs must contain only CLI dispatch — no logic."
  exit 1
fi

echo "OK: module structure valid, main.rs: $MAIN_LINES lines"
exit 0
```

---

### Rule 4: Single Responsibility Naming Contract

Each `src/` file must begin with a doc comment of exactly one sentence stating its single responsibility. The sentence must not contain "and" — if it does, the file has two responsibilities and must be split.

```rust
//! Kahn's topological sort over the feature dependency DAG.

//! AISP formal block parser — produces typed FormalBlock AST from markdown.

//! Atomic file writes and fsync discipline for all Product mutations.
```

Checked by CI script:

```bash
#!/usr/bin/env bash
# scripts/checks/single-responsibility.sh
set -euo pipefail

VIOLATIONS=()
find src -name "*.rs" ! -name "mod.rs" ! -name "main.rs" | while read -r file; do
  FIRST_LINE=$(head -1 "$file")
  if [[ ! "$FIRST_LINE" =~ ^//! ]]; then
    echo "ERROR: $file: missing single-responsibility doc comment (first line must be //! ...)"
    exit 1
  fi
  if [[ "$FIRST_LINE" =~ " and " ]]; then
    echo "ERROR: $file: responsibility doc comment contains 'and' — split this file"
    echo "  Found: $FIRST_LINE"
    exit 1
  fi
done

echo "OK: all files have single-responsibility doc comments"
exit 0
```

---

### TC Files

These TCs have `scope: cross-cutting` (see ADR-025) — they validate every feature's implementation implicitly. They run via `product verify --platform`. They use `runner: bash` pointing to the enforcement scripts.

**TC-CQ-001** — File length hard limit:
```yaml
---
id: TC-CQ-001
title: No Rust Source File Exceeds 400 Lines
type: exit-criteria
status: unimplemented
runner: bash
runner-args: ["scripts/checks/file-length.sh"]
runner-timeout: 10s
validates:
  adrs: [ADR-029]
  features: []    # cross-cutting — validated via product verify --platform
---
```

**TC-CQ-002** — File length warning:
```yaml
---
id: TC-CQ-002
title: No Rust Source File Exceeds 300 Lines (Warning)
type: invariant
status: unimplemented
runner: bash
runner-args: ["FILE_LENGTH_HARD=99999", "FILE_LENGTH_WARN=300",
              "scripts/checks/file-length.sh"]
runner-timeout: 10s
validates:
  adrs: [ADR-029]
  features: []
---
```

**TC-CQ-003** — Function length:
```yaml
---
id: TC-CQ-003
title: No Function Exceeds 40 Statement Lines
type: invariant
status: unimplemented
runner: bash
runner-args: ["scripts/checks/function-length.sh"]
runner-timeout: 30s
validates:
  adrs: [ADR-029]
  features: []
---
```

**TC-CQ-004** — Module structure:
```yaml
---
id: TC-CQ-004
title: Required Module Structure Present and main.rs Within Limits
type: exit-criteria
status: unimplemented
runner: bash
runner-args: ["scripts/checks/module-structure.sh"]
runner-timeout: 10s
validates:
  adrs: [ADR-029]
  features: []
---
```

**TC-CQ-005** — Single responsibility doc comments:
```yaml
---
id: TC-CQ-005
title: Every Source File Has a Single-Responsibility Doc Comment Without "and"
type: invariant
status: unimplemented
runner: bash
runner-args: ["scripts/checks/single-responsibility.sh"]
runner-timeout: 10s
validates:
  adrs: [ADR-029]
  features: []
---
```

---

### Integration with `product verify --platform`

TC-CQ-001 through TC-CQ-005 have empty `validates.features` — they are not linked to any specific feature. They are validated via `product verify --platform`, which runs all TCs linked to cross-cutting ADRs. ADR-029 has `scope: cross-cutting`.

This means: every time any feature is implemented and `product verify --platform` is run, the code quality checks run alongside the platform invariants. A new file that creeps past 400 lines fails the platform check, not just a code review comment.

---

**Rationale:**
- File size limits are not aesthetic. For LLM-driven development they are a context quality constraint. A 1600-line file means the implementation agent receives 1600 lines when it needs 80. The agent either truncates (missing context) or processes everything (noise drowning signal). Both outcomes produce worse implementations than a focused 200-line file.
- The single-responsibility doc comment rule is self-enforcing documentation. Writing "//! Graph traversal and centrality computation." and seeing it fail CI because of "and" is a clearer signal than a code review comment saying "this file has two responsibilities."
- Shell scripts for enforcement rather than custom lints makes the rules auditable. Any developer can read `file-length.sh` and understand what it checks. A custom clippy lint requires understanding Rust's compiler plugin API. Shell scripts are boring and correct.
- The 400-line hard limit with a 300-line warning gives two signals: "you're approaching the limit" (warning, visible in CI) and "you've exceeded it" (error, blocks CI). The warning is the more valuable signal — it's caught before the file becomes a problem.
- `TC-CQ-002` uses the trick of setting `FILE_LENGTH_HARD=99999` to disable the hard limit and only check the warning threshold. This lets `product verify` distinguish between "over the warning threshold" (exit 2) and "over the hard limit" (exit 1) using the existing three-tier exit code model.

**Rejected alternatives:**
- **Custom clippy lint for file length** — requires understanding `rustc`'s internal span API. Brittle across Rust versions. Rejected: shell script is simpler, more portable, and more readable.
- **tokei or similar line-counting tools** — dependency on an external binary. Rejected: `wc -l` and `awk` are universally available. No installation required.
- **250-line limit** — tested against the existing Product codebase. The graph module's `centrality.rs` with full Brandes' implementation legitimately reaches 280 lines. 250 would require artificial splitting of cohesive algorithms. Rejected.
- **No module structure mandate** — leaves module decomposition to the implementing agent's judgment. Agents without a defined module structure will make different choices on different features, producing inconsistent organisation that compounds over time. A defined structure eliminates this decision entirely.