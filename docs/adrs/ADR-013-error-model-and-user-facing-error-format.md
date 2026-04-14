---
id: ADR-013
title: Error Model and User-Facing Error Format
status: accepted
features: []
supersedes: []
superseded-by: []
domains: [error-handling, api]
scope: domain
content-hash: sha256:871466eb005fd371a0fc05023afc6de14305df42f23a9f2faced5693a5360049
---

**Status:** Accepted

**Context:** Product operates as a CLI tool used both interactively by developers and non-interactively in CI pipelines. Errors occur in two distinct contexts with different requirements:

- **Interactive use:** a developer runs `product context FT-001` and gets a clear, actionable message telling them exactly what is wrong and where to fix it
- **CI use:** a pipeline runs `product graph check` and needs machine-parseable output it can surface in a PR comment or test report

Additionally, there are two fundamentally different categories of failure: user errors (malformed front-matter, broken links, invalid arguments) and internal errors (bugs in Product itself). These must never be presented identically — a user should never see a Rust panic or stack trace for something they caused, and a bug should never be silently swallowed.

**Decision:** Define a four-tier error taxonomy with a consistent display format for each tier, structured stderr output for CI consumption, and a strict rule that no user action produces a Rust panic.

---

### Error Taxonomy

**Tier 1 — Parse errors:** malformed YAML front-matter, unrecognised front-matter fields that are required, invalid ID format. The artifact file is not usable.

**Tier 2 — Graph errors:** broken links (reference to non-existent artifact), dependency cycles, supersession cycles. The graph is structurally inconsistent.

**Tier 3 — Validation warnings:** orphaned artifacts, features without exit criteria, formal blocks missing on invariant/chaos tests, phase/dependency ordering disagreements. The graph is usable but incomplete.

**Tier 4 — Internal errors:** unexpected state that represents a bug in Product. Anything that would otherwise produce a Rust `panic!`.

---

### Display Format

All errors and warnings are written to **stderr**. Stdout is reserved for command output (context bundles, lists, query results). This separation ensures that `product context FT-001 > bundle.md` produces a clean file even when warnings are present.

**Interactive format (default):**
```
error[E002]: broken link
  --> docs/features/FT-003-rdf-projection.md
   |
 4 | adrs: [ADR-001, ADR-002, ADR-099]
   |                          ^^^^^^^ ADR-099 does not exist
   |
   = hint: create the file with `product adr new` or remove the reference

warning[W003]: missing exit criteria
  --> docs/features/FT-002-products-iam.md
   |
   = no test criterion of type `exit-criteria` is linked to this feature
   = hint: add one with `product test new --type exit-criteria`
```

Format mirrors rustc and clang diagnostic output — engineers arrive with prior knowledge of this style. Every message includes: error code, human description, file path, line number where applicable, the offending content, and a `hint` for remediation.

**Structured format (`--format json`, for CI):**
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
  "warnings": [...],
  "summary": { "errors": 1, "warnings": 2 }
}
```

**Internal errors (Tier 4):**
```
internal error: unexpected None in topological sort at graph/topo.rs:147
  This is a bug in Product. Please report it at https://github.com/.../issues
  with the output of `product --version` and the command you ran.
```

Internal errors always print the source location, the Product version, and a link to file an issue. They never print a Rust panic backtrace directly (though `RUST_BACKTRACE=1` enables it for debugging).

---

### Error Codes

| Code | Tier | Description |
|---|---|---|
| E001 | Parse | Malformed YAML front-matter |
| E002 | Graph | Broken link — referenced artifact does not exist |
| E003 | Graph | Dependency cycle in `depends-on` DAG |
| E004 | Graph | Supersession cycle in ADR `supersedes` chain |
| E005 | Parse | Invalid artifact ID format |
| E006 | Parse | Missing required front-matter field |
| E007 | Parse | Unknown artifact type in `type` field |
| E008 | Schema | `schema-version` in `product.toml` exceeds binary support |
| E009 | Orchestration | `product implement` blocked — unsuppressed high-severity gaps |
| E010 | Concurrency | Repository locked — another Product process holds the write lock |
| E011 | Domain | `domains-acknowledged` entry present with empty or missing reasoning |
| E012 | Domain | Domain declared in front-matter not present in `product.toml` vocabulary |
| E013 | Dependency | Dependency has no linked ADR — every dependency requires a governing decision |
| W001 | Validation | Orphaned artifact — no incoming links |
| W002 | Validation | Feature has no linked test criteria |
| W003 | Validation | Feature has no exit-criteria type test |
| W004 | Validation | Invariant/chaos test missing formal block |
| W005 | Validation | Phase label disagrees with dependency order |
| W006 | Validation | Formal block evidence `δ` below threshold (< 0.7) |
| W007 | Schema | Schema upgrade available — current version is behind binary support |
| W008 | Migration | ADR status field not found, defaulted to `proposed` |
| W009 | Migration | No test subsection found in ADR — no TC files extracted |
| W010 | Domain | Cross-cutting ADR not linked or acknowledged by a feature |
| W011 | Domain | Feature declares a domain with domain-scoped ADRs but no coverage |
| W012 | Measurement | Feature has no `bundle` block — context bundle size has never been measured |
| W013 | Dependency | Feature uses a deprecated or migrating dependency |
| W015 | Dependency | Dependency `availability-check` failed during preflight |
| I001 | Internal | Unexpected None in graph traversal |
| I002 | Internal | Assertion failure in topological sort |

---

### Implementation Rules

- `#![deny(clippy::unwrap_used)]` and `#![deny(clippy::expect_used)]` in all production code paths. Every `Option` and `Result` is handled explicitly.
- All Tier 1–3 failures return structured `Error` or `Warning` values through the call stack. No `eprintln!` in library code — only in the CLI rendering layer.
- Tier 4 errors use a dedicated `internal_error!` macro that captures file and line, formats the message, and exits with code 3. Code 3 is reserved exclusively for internal errors, distinguishing them from user-caused failures (1, 2).
- `--format json` is a global flag on all commands, not per-command. When set, all output (errors, warnings, and results) is JSON.

**Rationale:**
- The rustc-style diagnostic format is the single most important UX decision in the error model. It provides location, cause, and remediation in one message. Developers spend less time debugging Product and more time fixing their artifacts.
- Separating stderr (errors/warnings) from stdout (results) is a Unix convention that makes scripting and piping reliable.
- Structured JSON output on stderr with `--format json` enables CI tools (GitHub Actions, GitLab CI, Buildkite) to parse and annotate PRs without screen-scraping.
- The four-tier taxonomy prevents the two most common error model failures: conflating bugs with user errors, and treating all user errors identically regardless of severity.

**Rejected alternatives:**
- **Panic on internal errors** — unacceptable. A Rust panic produces a backtrace that reveals implementation details and is indistinguishable from a bug in user-controlled input parsing.
- **All errors to stdout** — breaks piping. `product context FT-001 > bundle.md` must produce a clean file.
- **Single `--verbose` flag for structured output** — conflates verbosity with machine-readability. `--format json` is explicitly about output format, not detail level.