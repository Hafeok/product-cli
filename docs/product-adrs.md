# Product — Architecture Decision Records

> **Status:** Draft
> **Version:** 0.1
> **Companion:** See `product-prd.md` for full product requirements

---

## ADR-001: Rust as Implementation Language

**Status:** Accepted

**Context:** Product must ship as a single binary with no runtime dependencies. It needs to run on developer laptops (ARM64 Apple Silicon, x86_64 Linux), in CI pipelines, and eventually on ARM64 Raspberry Pi nodes alongside PiCloud itself. The tool parses files, builds an in-memory graph, and assembles markdown output — none of which are exotic requirements, but the deployment constraint (single binary, no installer, no runtime) is strict.

Additionally, Product is a companion tool for PiCloud, which is written in Rust. Shared language means shared tooling, shared CI patterns, and the ability to eventually share libraries (particularly the Oxigraph integration).

**Decision:** Implement Product in Rust.

**Rationale:**
- Single binary compilation to ARM64, x86_64, and Apple Silicon with no runtime
- Cargo cross-compilation is well-understood; CI matrix builds across targets are straightforward
- `clap` provides a production-quality CLI argument parser with shell completion generation
- `oxigraph` for embedded SPARQL is a Rust-native library — no FFI required
- `gray_matter` and `pulldown-cmark` handle YAML front-matter and markdown parsing
- Alignment with PiCloud's toolchain — one language, one formatter, one linter across the project
- LLMs produce high-quality Rust when given explicit type contracts and clear architectural context

**Rejected alternatives:**
- **TypeScript/Node** — natural first choice for a CLI that iterates fast; `gray-matter` is excellent. Rejected because it requires Node.js to be installed, which violates the single-binary constraint. `pkg`/`bun` can bundle Node apps, but the result is a large binary with bundled runtime, not a native binary.
- **Go** — would satisfy the single-binary constraint and has good CLI tooling (`cobra`). Rejected because it would fragment the toolchain from PiCloud. The development overhead of maintaining two language ecosystems on a small project is not justified.
- **Python** — fast iteration, good parsing libraries. Rejected due to runtime dependency and the absence of a clean single-binary story.

**Test coverage:**

Scenario tests:
- `binary_compiles_arm64.rs` — `cargo build --release --target aarch64-unknown-linux-gnu` completes with zero errors and zero warnings.
- `binary_compiles_x86.rs` — `cargo build --release --target x86_64-unknown-linux-musl` completes with zero errors and zero warnings.
- `binary_no_deps.sh` — `ldd product` on the Linux binary reports no dynamic dependencies beyond `libc`. Any other line is a test failure.

Exit criteria:
- Binary size < 20 MB (stripped) on all targets.
- `cargo build --release` completes in < 5 minutes on a Raspberry Pi 5 (cold cache).
- Zero `unwrap()` calls reachable from production code paths (enforced via `#![deny(clippy::unwrap_used)]`).

---

## ADR-002: YAML Front-Matter as the Graph Source of Truth

**Status:** Accepted

**Context:** The knowledge graph linking features, ADRs, and test criteria must be maintained somehow. The options are: (a) a separate graph file hand-maintained alongside the markdown documents, (b) inline declarations within the markdown prose, or (c) YAML front-matter in each document that declares its identity and outgoing edges.

Option (a) creates a synchronisation problem — the graph file and the document files diverge. Option (b) is ambiguous to parse and fragile as document content changes. Option (c) keeps each document self-describing. The front-matter is the contract between the document and the graph.

**Decision:** YAML front-matter in every artifact file is the sole source of truth for graph relationships. The graph is always derived from front-matter on demand; there is no persistent graph store.

**Rationale:**
- Each file is self-describing — open any file and immediately understand its place in the graph
- Git diffs on front-matter are clean and reviewable: adding a link to an ADR is a one-line change
- No synchronisation problem: the graph cannot drift from the documents because it is always recomputed from them
- YAML front-matter is a well-understood convention (Jekyll, Hugo, Obsidian, academic tools); contributors arrive with prior knowledge
- The `serde_yaml` crate deserialises front-matter into typed Rust structs in one call
- Front-matter fields are strictly typed and validated on parse — `product graph check` reports malformed declarations

**Rejected alternatives:**
- **Separate `links.toml` graph file** — clean separation of concerns, but introduces a synchronisation requirement. Every time a new artifact is added, two files must be updated. In practice, contributors update the document and forget the graph file.
- **RDF/Turtle as the primary source** — philosophically consistent with PiCloud, but Turtle is not a natural authoring format for humans writing markdown documents. It would require a separate editor workflow or tooling that does not exist yet.
- **Inline markdown annotations** — `<!-- links: ADR-002, ADR-003 -->` style comments. Fragile, non-standard, and invisible in rendered output. Harder to validate programmatically.

**Test coverage:**

Scenario tests:
- `frontmatter_parse_feature.rs` — parse a well-formed feature file. Assert all fields deserialise correctly into the `Feature` struct. Assert `adrs` and `tests` vectors contain the expected IDs.
- `frontmatter_parse_adr.rs` — parse a well-formed ADR file. Assert `features`, `supersedes`, `superseded-by` deserialise correctly.
- `frontmatter_invalid_id.rs` — parse a feature file where `adrs` references a non-existent ID. Assert `graph check` reports the broken link and exits with code 1.
- `frontmatter_missing_required.rs` — parse a feature file with no `id` field. Assert the parser returns a structured error with the file path and field name.

Invariants:
- Every artifact file that is syntactically valid YAML front-matter must parse without panic. Any file that causes a panic is a bug, not a validation error.
- Front-matter fields not recognised by the schema are ignored with a warning, never an error (forward compatibility).

---

## ADR-003: Derived Graph — No Persistent Graph Store

**Status:** Accepted

**Context:** The knowledge graph must be queryable by the CLI. The choices are: persist the graph to a file (SQLite, RDF store, TOML index), regenerate it on every command invocation, or keep it in a daemon process.

**Decision:** Rebuild the in-memory graph from front-matter on every command invocation. The graph is never persisted. `index.ttl` is an export artefact for external tooling, never read by Product.

**Rationale:**
- A developer repository for a project like PiCloud will have on the order of 50–200 artifact files. Reading and parsing all of them takes < 50ms on any modern hardware. There is no performance case for caching.
- A persistent graph store introduces a synchronisation invariant: the graph must always match the files. This invariant is impossible to enforce perfectly (files can be edited outside Product, git operations change files without invoking the CLI). A derived graph is always correct by construction.
- No migration strategy is needed when the schema changes. Old front-matter that Product can no longer parse is reported as a warning; it does not corrupt a stored graph.
- The `index.ttl` export is a snapshot. If it is stale, `product graph rebuild` regenerates it. The CLI never depends on it being fresh.

**Rejected alternatives:**
- **SQLite index** — fast random access, good for large repositories. Rejected because the target scale (< 200 files) does not justify the added complexity of cache invalidation, migration, and the possibility of a corrupted or stale index.
- **Daemon process** — the graph stays hot in memory; file watching keeps it current. Rejected as massively over-engineered for a developer CLI tool. Daemons have startup costs, crash modes, and version skew problems.
- **`index.ttl` as read source** — `product graph rebuild` generates it; CLI reads from it. Rejected because stale `index.ttl` would silently produce wrong answers. The graph must always reflect the current file state.

**Test coverage:**

Scenario tests:
- `graph_rebuild_from_scratch.rs` — start with a directory of 10 feature files, 8 ADR files, and 15 test files. Invoke any CLI command. Assert the graph contains the correct node and edge counts without any prior `graph rebuild` having been run.
- `graph_stale_ttl.rs` — generate `index.ttl`, then add a new feature file. Invoke `product feature list`. Assert the new feature appears in the list (graph was rebuilt from files, not from stale TTL).

Invariants:
- Parse time for a repository of 200 files must be < 200ms on a 2020-era laptop. Measured in the benchmark suite on every release.

---

## ADR-004: Markdown as the Document Format

**Status:** Accepted

**Context:** Artifact files must be human-readable, diffable in git, renderable on GitHub and GitLab, and directly injectable into LLM context windows without transformation. The format choice affects authoring ergonomics, tooling availability, and the cost of the context bundle assembly step.

**Decision:** All artifact files are CommonMark markdown with YAML front-matter. No other format is supported.

**Rationale:**
- Markdown renders natively on every git hosting platform — no separate documentation pipeline required
- Markdown is the native input format for LLM context injection; no conversion step needed in context bundle assembly
- `pulldown-cmark` provides a robust, spec-compliant Rust parser
- GitHub Copilot, Cursor, and most LLM-assisted editors have first-class markdown support
- Front-matter stripping (removing the `---` block before injection) is a trivial string operation
- Code blocks, tables, and headings are all expressible in markdown — sufficient for the content patterns in features, ADRs, and test criteria

**Rejected alternatives:**
- **AsciiDoc** — more expressive than markdown, better tooling for long documents. Rejected because it does not render on GitHub by default, and LLM context injection requires an extra conversion step.
- **TOML/structured data** — fully machine-readable, no parsing ambiguity. Rejected because ADRs and features contain substantial prose (rationale, context, rejected alternatives) that is not natural to express in structured data.
- **Org-mode** — excellent for Emacs users. Rejected due to minimal tooling outside Emacs and no native renderer on git hosting platforms.

**Test coverage:**

Scenario tests:
- `markdown_front_matter_strip.rs` — read a markdown file with front-matter. Assert the context bundle output contains no `---` delimiters and no YAML fields.
- `markdown_passthrough.rs` — a markdown file with code blocks, tables, and nested lists. Assert the context bundle output preserves these structures verbatim.

---

## ADR-005: Numeric ID Scheme (FT-XXX, ADR-XXX, TC-XXX)

**Status:** Accepted

**Context:** Artifacts need stable, human-readable, machine-parseable identifiers. These IDs appear in front-matter links, CLI commands, filenames, and LLM context bundles. They must be: short enough to type, unambiguous, sortable, and stable after assignment.

**Decision:** Use prefixed zero-padded numeric IDs: `FT-001`, `ADR-001`, `TC-001`, `DEP-001`. IDs are assigned sequentially by `product feature/adr/test/dep new`. Once assigned, IDs are permanent — artifacts are never renumbered. Retired artifacts are marked `status: abandoned`, not deleted.

**Sub-namespace extension:** Cross-cutting TCs that validate platform-wide properties rather than specific features use a sub-namespace suffix: `TC-CQ-001` (code quality), `TC-PLT-001` (platform invariants). The sub-namespace is a human-readable classifier only — it does not affect Product's parsing, storage, or graph logic. All TC IDs are treated identically by the system regardless of suffix. The sub-namespace prevents numeric collision when cross-cutting TCs are added without displacing feature-specific TC IDs.

**Rationale:**
- Sequential numeric IDs are common convention in engineering (JIRA, ADR numbering, RFC numbering) — contributors arrive with prior knowledge
- Prefixes (`FT`, `ADR`, `TC`) make the artifact type visible in any context where the ID appears
- Zero-padding ensures correct alphabetical sort in file listings and git diffs
- Permanent IDs mean that external references (comments in code, commit messages, slack messages) remain valid indefinitely
- The prefix is configurable in `product.toml` — teams that prefer `FEAT`, `DEC`, `TEST` can use those instead

**Rejected alternatives:**
- **Slug-based IDs** (e.g., `cluster-foundation`) — human-readable but not stable if the title changes. Two artifacts with similar titles produce collision-prone slugs.
- **UUIDs** — globally unique, collision-free. Rejected because UUIDs are unreadable in context. `FT-001` in a commit message is meaningful; `3f2504e0-4f89-11d3-9a0c-0305e82c3301` is not.
- **Semantic versioning** — expressive for API-like artifacts. Rejected because it implies a release lifecycle that does not map cleanly to features and decisions.

**Test coverage:**

Scenario tests:
- `id_auto_increment.rs` — create three features in sequence. Assert their IDs are `FT-001`, `FT-002`, `FT-003`.
- `id_gap_fill.rs` — create features `FT-001` and `FT-003` manually. Run `product feature new`. Assert the new feature is assigned `FT-004` (gaps are not filled — next ID is always `max(existing) + 1`).
- `id_conflict.rs` — attempt to create a feature with an ID that already exists. Assert the CLI returns an error and does not overwrite the existing file.

---

## ADR-006: Context Bundle as the Primary LLM Interface

**Status:** Accepted

**Context:** The primary use case for Product is to give LLM agents precisely the context they need for implementation tasks. The question is: what is the right unit of context, and what format should it take?

A naive approach is to dump the entire repository into the LLM context. This fails at scale: a project with 40 features, 30 ADRs, and 80 test criteria produces a context document of 200,000+ tokens — past the practical window of most models and past the point where signal-to-noise is useful.

**Decision:** The context bundle — a feature, its linked ADRs, and its linked test criteria — is the primary output of Product and the primary input to LLM agents. Bundles are assembled deterministically and formatted as markdown. The context command is a first-class citizen of the CLI, not an afterthought.

**Rationale:**
- A single feature with its linked ADRs and test criteria typically produces 3,000–8,000 tokens — well within any current LLM's practical working window
- The relational structure means nothing relevant is omitted (every ADR that applies is included) and nothing irrelevant is included (ADRs for unrelated features are excluded)
- Deterministic assembly order means two invocations of `product context FT-001` produce identical output — cacheable, auditable, reproducible
- The header block (feature ID, phase, status, linked artifact IDs) is machine-parseable by the receiving agent without requiring it to read the full bundle
- Superseded ADRs are included with a `[SUPERSEDED by ADR-XXX]` annotation — the agent has the full decision history, not just the current state

**Rejected alternatives:**
- **Full repository dump** — complete context, no relevance filtering. Rejected because 200K tokens of mixed context produces demonstrably worse agent outputs than 5K tokens of targeted context. Empirically validated.
- **Feature file only** — minimal context. Rejected because the agent needs the rationale (ADRs) and the success criteria (tests) to implement correctly. A feature description without its decisions is ambiguous.
- **Streaming / agentic retrieval** — the agent calls Product as a tool to fetch context as needed. Possible, but requires the agent to be running in a tool-use environment. The bundle approach works in any context window — a terminal paste, a system prompt, a file attachment.
- **Token budget flag (`--max-tokens`)** — considered adding truncation logic to `product context` to fit a target context window. Rejected: token budget management is the agent's responsibility. Product's job is to assemble a complete and accurate bundle. Truncation decisions require knowledge of the model, the task, and the surrounding prompt — none of which Product has. An agent that needs to fit a window should request a narrower scope (single feature, ADRs-only) rather than rely on Product to guess what to drop.

**Supersession behaviour:** When a context bundle is assembled, superseded ADRs are replaced by their successors. The superseded ADR does not appear in the bundle. This keeps the bundle actionable — an agent receiving it sees only the current, accepted set of decisions. The supersession chain is recorded in the ADR's own front-matter and is queryable via `product adr show`, but it does not propagate into context bundles.

**Test coverage:**

Scenario tests:
- `context_bundle_feature.rs` — call `product context FT-001` on a repository with FT-001 linked to ADR-001, ADR-002, and TC-001. Assert the output contains the feature content, both ADR contents, and the test criterion content, in the correct order.
- `context_bundle_no_frontmatter.rs` — assert the context bundle output contains no YAML front-matter blocks (front-matter is stripped from all sections).
- `context_bundle_header.rs` — assert the context bundle header block contains the correct feature ID, phase, status, and linked artifact ID lists.
- `context_bundle_superseded_adr.rs` — link a superseded ADR to a feature. Assert it appears in the bundle with a `[SUPERSEDED by ADR-XXX]` annotation.
- `context_measure_updates_frontmatter.rs` — run `product context FT-001 --measure`. Assert feature front-matter `bundle` block is written with correct `depth-1-adrs`, `tcs`, `domains`, `tokens-approx`, and `measured-at` fields.
- `context_measure_appends_metrics.rs` — run `product context FT-001 --measure`. Assert an entry is appended to `metrics.jsonl` containing the feature ID and bundle dimensions.
- `context_measure_idempotent.rs` — run `product context FT-001 --measure` twice. Assert `metrics.jsonl` has two entries (one per invocation). Assert front-matter `bundle` block reflects the most recent measurement only.

Exit criteria:
- `product context FT-001` for a feature with 4 ADRs and 6 test criteria completes in < 100ms.
- `product context FT-001 --measure` completes in < 150ms (measurement adds token counting, not an LLM call).
- Context bundle output is valid CommonMark. Verified by `pulldown-cmark` parse with zero errors.

**`--measure` flag specification:**

`product context FT-XXX --measure` assembles the bundle normally, then computes and records:

| Field | Description |
|---|---|
| `depth-1-adrs` | Count of ADRs directly linked to the feature |
| `depth-2-adrs` | Count of additional ADRs reachable at depth 2 |
| `tcs` | Count of test criteria linked to the feature |
| `domains` | Count of distinct domains across linked ADRs |
| `tokens-approx` | Approximate token count using cl100k_base tokeniser (tiktoken) |
| `measured-at` | ISO 8601 timestamp of the measurement |

Token counting uses the cl100k_base tokeniser (same encoding as GPT-4o and claude-sonnet). The count is approximate — it measures the assembled markdown bytes, not a model-specific encoding. The `tokens-approx` label makes the approximation explicit.

Measurements are written to two places atomically: the feature's front-matter `bundle` block (latest measurement only) and `metrics.jsonl` (append-only history). The front-matter value is used for threshold checks and `product graph stats`. The `metrics.jsonl` entries are used for trend analysis.

---

## ADR-007: Checklist is Generated, Never Hand-Edited

**Status:** Accepted

**Context:** The original workflow used `checklist.md` as the source of truth for implementation status — developers ticked boxes to mark work complete. This design had a divergence problem: front-matter and checklist could disagree. Since then, the Product toolchain has matured: `product verify` updates TC and feature status directly in front-matter, `product status` renders phase gate state and exit criteria progress in the terminal, `product feature next` uses topological sort to determine what to implement next, and agents call `product_feature_list` rather than reading a file. Implementation status now lives entirely in front-matter. Agents no longer need checklist.md.

**Decision:** `checklist.md` is a generated human-readable view for stakeholders and GitHub rendering. It is not a data source, not an agent input, and not a source of truth. Implementation status is owned exclusively by feature and TC front-matter. `product checklist generate` produces `checklist.md` on demand. The file is listed in `.gitignore` by default — it is a local rendering, not a committed artifact, unless the project explicitly chooses to commit it for GitHub visibility.

**Rationale:**
- Front-matter is the single source of truth. Checklist.md is a projection of that truth, not a parallel record.
- Agents use `product_feature_list`, `product status`, and `product feature next` — none of these require checklist.md to exist. Removing checklist.md from the committed repository eliminates a file that can silently go stale.
- The legitimate remaining use case — "show a stakeholder what's been built without requiring Product to be installed" — is served by generating the file on demand and either sharing it or committing it deliberately. The default is not to commit it.
- GitHub renders markdown checkboxes natively. For projects that want GitHub visibility of implementation status, committing checklist.md remains valid — the project sets `checklist-in-gitignore = false` in `product.toml`.

**Migration note:** The existing `checklist.md` in PiCloud's repository should be treated as the initial status snapshot. During migration, `product migrate` reads checked boxes in the existing checklist and populates `status` fields in the scaffolded feature files accordingly. After migration, checklist.md is redundant as a data source.

**Rejected alternatives:**
- **Checklist as source of truth, front-matter derived** — reverses the ownership. Markdown checkbox state is harder to parse programmatically than a YAML enum field. Checklist entries cannot express the distinction between `planned`, `in-progress`, `complete`, and `abandoned`.
- **Both are sources of truth (sync on conflict)** — any two-source-of-truth design requires a merge strategy. Merge strategies for status fields have no correct answer when they diverge. Reject this entire class of design.
- **Remove checklist.md entirely** — loses the legitimate stakeholder and GitHub rendering use case. The file is genuinely useful as an occasional generated snapshot. Keeping it as an optional view rather than a required artifact is the right balance.

**Test coverage:**

Scenario tests:
- `checklist_generate.rs` — set three features to `in-progress`, `complete`, `planned`. Run `product checklist generate`. Assert the checklist contains the correct status markers and no YAML front-matter.
- `checklist_no_manual_edit_warning.rs` — assert the generated checklist begins with a comment block warning against manual editing and stating that front-matter is the source of truth.
- `checklist_roundtrip.rs` — generate checklist, change a feature status, regenerate. Assert the checklist reflects the updated status.
- `checklist_gitignore_default.rs` — run `product init` on a new repository. Assert `checklist.md` appears in `.gitignore` by default.
- `checklist_gitignore_opt_out.rs` — set `checklist-in-gitignore = false` in `product.toml`. Assert `checklist.md` does NOT appear in `.gitignore`.

---

## ADR-008: Embedded Oxigraph for SPARQL Queries

**Status:** Accepted

**Context:** `product graph query` must execute SPARQL 1.1 queries over the derived knowledge graph. The options are: an embedded in-process RDF store, an external SPARQL endpoint, or a custom query language.

**Decision:** Use `oxigraph` as the embedded in-process SPARQL 1.1 store. The graph is loaded from the in-memory representation on each `graph query` invocation. Oxigraph is a dependency, not a service.

**Rationale:**
- Oxigraph is a Rust-native SPARQL 1.1 implementation — no FFI, compiles cleanly to all target architectures
- PiCloud already uses Oxigraph for cluster state projection. Product using the same library maintains toolchain consistency and reduces the total dependency surface
- In-memory mode (no persistent storage) is fully supported by Oxigraph — the graph is loaded from the in-memory `GraphModel` and queries execute over it without touching disk
- SPARQL 1.1 SELECT, CONSTRUCT, ASK, and DESCRIBE are all supported — the full query vocabulary is available
- No external service to start, no port to configure, no version to manage

**Rejected alternatives:**
- **Custom query language** — a simpler subset designed specifically for Product's use cases. Rejected because SPARQL is a standard with existing tooling, documentation, and user knowledge. A bespoke query language would require Product to own documentation and training for a capability that SPARQL already covers.
- **External SPARQL endpoint (Fuseki, Stardog)** — full SPARQL server with persistent storage. Rejected because it requires an external service to be running — violates the single-binary, no-external-dependencies constraint.
- **SQL over SQLite** — relational model is familiar, SQLite is embeddable. Rejected because the data model is a graph with typed triples. Mapping graph traversals to SQL JOIN chains produces verbose, fragile queries. SPARQL graph patterns are a natural fit for the data model.

**Test coverage:**

Scenario tests:
- `sparql_select_feature_adrs.rs` — load a graph with FT-001 linked to ADR-001 and ADR-002. Execute `SELECT ?adr WHERE { ft:FT-001 pm:implementedBy ?adr }`. Assert the result set contains exactly `adr:ADR-001` and `adr:ADR-002`.
- `sparql_untested_features.rs` — load a graph where FT-002 has no `pm:validatedBy` triples. Execute a query for features with no test criteria. Assert FT-002 appears in the result and FT-001 (which has tests) does not.
- `sparql_phase_filter.rs` — execute a query filtering features by `pm:phase 1`. Assert only phase-1 features appear in the result.

Exit criteria:
- Any SPARQL query over a graph of 200 nodes and 800 edges completes in < 500ms.

---

## ADR-009: CI Integration via Exit Codes

**Status:** Accepted

**Context:** Product should be usable as a CI gate — a step in a pull request pipeline that fails the build if the knowledge graph has broken links, orphaned artifacts, or missing test criteria. This requires a consistent, machine-readable signal from the CLI.

**Decision:** Product uses a three-tier exit code scheme for graph health commands:
- `0` — clean graph, no issues
- `1` — errors (broken links, supersession cycles, malformed front-matter)
- `2` — warnings only (orphaned artifacts, features without exit criteria, untested features)

All other commands exit `0` on success and `1` on any error.

**Rationale:**
- The two-level error/warning distinction allows CI pipelines to fail on broken links (hard errors) while optionally warning on coverage gaps without blocking the build
- The convention follows `grep` (0 = found, 1 = not found, 2 = error) and lint tools like `clippy` — engineers arrive with prior knowledge of this pattern
- A CI pipeline can choose its tolerance: `product graph check` (fail on errors and warnings) or `product graph check || [ $? -eq 2 ]` (fail on errors only)
- Shell-friendly: the exit code is testable without parsing stdout

**Rejected alternatives:**
- **Single exit code (0/1)** — simpler but loses the error/warning distinction. Teams that want to tolerate coverage gaps but not broken links cannot express this policy.
- **Structured JSON output to stdout, always exit 0** — requires the CI step to parse output and apply its own logic. Adds friction for common cases that exit codes handle natively.

**Test coverage:**

Scenario tests:
- `exit_code_clean.rs` — run `product graph check` on a fully consistent repository. Assert exit code 0.
- `exit_code_broken_link.rs` — add a feature that references a non-existent ADR. Assert exit code 1.
- `exit_code_warnings_only.rs` — create an ADR with no feature links (orphan). Assert exit code 2.
- `exit_code_ci_pipeline.sh` — shell script that runs `product graph check` and asserts the pipeline step fails on exit code 1 but passes on exit code 2 with the correct conditional.

---

## ADR-010: Auto-Orphan Test Criteria on Feature Abandonment

**Status:** Accepted

**Context:** Test criteria are linked to features via `validates.features` in their front-matter. When a feature is marked `abandoned`, the tests that validated it have no active feature to belong to. The question is whether Product should require the developer to manually clean up those links, or handle it automatically.

**Decision:** When a feature's status is set to `abandoned` (via `product feature status FT-XXX abandoned`), Product automatically removes that feature's ID from the `validates.features` list of all linked test criteria. Test criteria that end up with an empty `validates.features` list are orphaned. `product graph check` reports them as warnings (exit code 2). No test criteria are deleted.

**Rationale:**
- Requiring manual cleanup is friction that will routinely be skipped. Orphaned tests with stale feature links produce silent graph inconsistencies — `product graph check` reports the link as broken (exit code 1), blocking CI, for a situation that is not actually an error
- Auto-orphaning on abandonment is the less surprising behaviour: the developer said the feature is gone; Product cleans up the edges
- Tests are not deleted because they may still be useful: they can be re-linked to a successor feature, or they document behaviour that was specified but not built
- Orphaned tests surface as warnings, not errors. A warning prompts the developer to decide: re-link, or explicitly delete. An error would block CI for something that requires a judgment call
- The mutation is logged to stdout during the command so the developer sees exactly what was changed

**Rejected alternatives:**
- **Require explicit `product test unlink TC-XXX --feature FT-001`** — correct but creates friction. Abandoned features often have several linked tests. Requiring individual unlinking is a multi-step cleanup that will be deferred or forgotten.
- **Delete tests automatically** — too destructive. A test criterion represents specified behaviour. Deleting it erases the record that the behaviour was ever intended. Orphaning preserves the history.
- **No action — leave stale links** — stale links produce broken-link errors in `product graph check`. This would cause CI failures for abandoned features, which is a false positive. Not acceptable.

**Test coverage:**

Scenario tests:
- `abandon_feature_orphans_tests.rs` — create FT-001 linked to TC-001 and TC-002. Set FT-001 to `abandoned`. Assert TC-001 and TC-002 have FT-001 removed from their `validates.features`. Assert both tests appear in `product test untested`.
- `abandon_feature_exit_code.rs` — after abandoning a feature with linked tests, run `product graph check`. Assert exit code 2 (warning) not 1 (error).
- `abandon_feature_stdout.rs` — assert the abandonment command prints the list of test criteria that were auto-orphaned.
- `abandon_feature_tests_preserved.rs` — assert test criterion files are not deleted during abandonment, only their feature links are removed.

---

## ADR-011: AISP-Influenced Formal Notation for Test Criteria

**Status:** Accepted

**Context:** Test criteria files currently express constraints, invariants, and assertions in natural language prose. When an LLM implementation agent receives a context bundle, it must interpret prose like "exactly one node holds the Leader role at all times" and infer the precise semantics. Two agents, or the same agent on two invocations, may interpret this differently — producing implementations with subtly different invariant checks.

AISP (AI Symbolic Protocol) is a formal notation language designed to reduce LLM interpretation variance. Its key insight is that symbolic, typed expressions with formal semantics have near-zero ambiguity, whereas natural language descriptions of the same constraints have 40–65% interpretation variance. Rather than adopting AISP wholesale, we evaluated where its notation patterns deliver the most value in Product's artifact model.

Test criteria are the highest-value target for formal notation because:
- They express assertions that must be verified, not explained
- Ambiguity in a constraint definition leads directly to incorrect implementations or missed test cases
- They are consumed primarily by agents, not humans reading for understanding

ADR prose (context, rationale, rejected alternatives) is explicitly excluded from this decision — that content is argumentative and explanatory, where prose is the correct medium.

**Decision:** Test criterion files use a hybrid format: YAML front-matter for graph metadata, AISP-influenced formal blocks for constraints and invariants, and plain prose for the human-readable description only. The formal blocks are mandatory for `invariant` and `chaos` type test criteria. They are optional but encouraged for `scenario` and `exit-criteria` types.

**Format:**

```markdown
---
id: TC-002
title: Raft Leader Election
type: scenario
status: unimplemented
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
---

## Description

Bootstrap a two-node cluster. Assert that exactly one node is elected leader
within 10 seconds, and that the leader identity is reflected in the RDF graph.

## Formal Specification

⟦Σ:Types⟧{
  Node≜IRI
  Role≜Leader|Follower|Learner
  ClusterState≜⟨nodes:Node+, roles:Node→Role⟩
}

⟦Γ:Invariants⟧{
  ∀s:ClusterState: |{n∈s.nodes | s.roles(n)=Leader}| = 1
}

⟦Λ:Scenario⟧{
  given≜cluster_init(nodes:2)
  when≜elapsed(10s)
  then≜∃n∈nodes: roles(n)=Leader
       ∧ graph_contains(n, picloud:hasRole, picloud:Leader)
}

⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩
```

**Block semantics:**

| Block | Symbol | Purpose | Required for type |
|---|---|---|---|
| `⟦Σ:Types⟧` | Type definitions | Name the domain types used in rules | invariant, chaos |
| `⟦Γ:Invariants⟧` | Constraint rules | Formal assertions that must hold | invariant, chaos |
| `⟦Λ:Scenario⟧` | Given/when/then | Structured test flow | scenario |
| `⟦Λ:ExitCriteria⟧` | Measurable thresholds | Numeric pass/fail bounds | exit-criteria |
| `⟦Λ:Benchmark⟧` | Quality measurement | Conditions, scorer, pass threshold | benchmark |
| `⟦Ε⟧` | Evidence block | Confidence, coverage, stability | all types |

**Evidence block fields:**

| Field | Meaning | Range |
|---|---|---|
| `δ` | Specification confidence | 0.0–1.0 |
| `φ` | Coverage completeness (%) | 0–100 |
| `τ` | Stability signal | `◊⁺` (stable), `◊⁻` (unstable), `◊?` (unknown) |

**Symbol subset in use:**

Product uses a minimal AISP symbol subset, not the full specification. Only these symbols appear in Product test criteria:

| Symbol | Meaning |
|---|---|
| `≜` | Definition ("is defined as") |
| `≔` | Assignment |
| `∀` | For all |
| `∃` | There exists |
| `∧` | Logical and |
| `∨` | Logical or |
| `→` | Function type or implication |
| `⟨⟩` | Tuple or record |
| `\|` | Union type (in type definitions) |
| `⟦⟧` | Block delimiter |

This subset is sufficient for all constraint and invariant patterns encountered in the PiCloud ADRs. Full AISP notation (category theory operators, tri-vector decomposition, ghost intent search) is not adopted — it exceeds what is needed and would make files unreadable to contributors unfamiliar with the full spec.

**Rationale:**
- The formal blocks are consumed by LLM agents receiving context bundles. Replacing prose invariants with typed, symbolic expressions eliminates interpretation decisions at the agent side — the constraint is unambiguous
- The hybrid approach preserves human readability: the prose description remains the primary entry point for a human reading the file. The formal blocks are additive, not a replacement
- `⟦Γ:Invariants⟧` maps exactly to the invariant patterns already present in the PiCloud ADRs ("exactly one leader", "log index is strictly monotonically increasing") — this is not a new concept, it is a more precise notation for concepts already being expressed
- The `⟦Λ:Scenario⟧` given/when/then pattern is equivalent to Gherkin (BDD) but typed — agents familiar with either convention recognise it immediately
- The evidence block `⟦Ε⟧` makes specification confidence explicit and queryable. `product graph stats` can report aggregate confidence across all test criteria
- The symbol subset is stable: every symbol used is in Unicode's standard mathematical operators block, renders correctly in any markdown viewer, and is representable in all major editors without custom font configuration

**Rejected alternatives:**
- **Full AISP adoption** — the complete AISP 5.1 spec includes category theory constructs, tri-vector signal decomposition, and proof-by-layers that go well beyond what test criteria need. Full adoption would make files unreadable to contributors not trained in the spec. Rejected: overhead exceeds benefit.
- **Gherkin (BDD) format** — `Given/When/Then` in plain English. More familiar to many engineers, good tooling. Rejected because it still relies on natural language for the assertion content — `"Then exactly one leader exists"` has the same interpretation problem as prose. Gherkin structures the test but does not eliminate ambiguity in the assertion.
- **JSON Schema / OpenAPI assertions** — machine-readable, well-tooled. Rejected because JSON is not a natural fit for logical quantifiers (`∀`, `∃`) and temporal assertions (`within 10s`). The resulting schemas are verbose and hard to scan.
- **Keep prose only** — minimal friction for authors. Rejected because the context bundle's primary consumer is an LLM agent, and prose invariants demonstrably require interpretation decisions that formal notation eliminates.

**Migration:**

Existing test criteria extracted from the PiCloud ADRs are in prose. Migration is incremental:
1. `product test new` scaffolds new criteria with formal block stubs
2. Existing criteria get prose descriptions only — the formal blocks are absent, not malformed
3. `product graph check` reports criteria with missing formal blocks as warnings (not errors) when the criterion type is `invariant` or `chaos`
4. `product graph stats` reports `φ` (formal coverage) — the percentage of invariant/chaos criteria that have formal blocks — so coverage is visible without being a hard gate

**Test coverage:**

Scenario tests:
- `formal_block_parse_types.rs` — parse a test criterion file with a `⟦Σ:Types⟧` block. Assert all type definitions deserialise into the `TypeDef` struct with correct names and variants.
- `formal_block_parse_invariants.rs` — parse a `⟦Γ:Invariants⟧` block with a universal quantifier. Assert the parsed expression tree matches the expected structure.
- `formal_block_parse_scenario.rs` — parse a `⟦Λ:Scenario⟧` block with `given/when/then` fields. Assert all three fields are present and non-empty.
- `formal_block_evidence.rs` — parse `⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩`. Assert `delta=0.95`, `phi=100`, `tau=Stable`.
- `formal_block_missing_invariant_warning.rs` — create an `invariant` type test criterion with no `⟦Γ⟧` block. Run `product graph check`. Assert exit code 2 (warning, not error).
- `context_bundle_formal_blocks_preserved.rs` — assert that formal blocks in test criteria are preserved verbatim in the context bundle output, not stripped like front-matter.

Invariants:
- The evidence block `δ` value must be in range [0.0, 1.0]. Values outside this range are a parse error.
- The evidence block `φ` value must be in range [0, 100]. Values outside this range are a parse error.
- A test criterion file with a malformed formal block (unclosed `⟦`, unknown block type) is a parse error, reported with file path and line number.

---

## ADR-012: Graph Theory Foundations for Navigation, Context, and Impact Analysis

**Status:** Accepted

**Context:** The current graph model supports only fixed 1-hop traversals: a feature's direct ADRs, a feature's direct tests, an ADR's direct features. This is sufficient for simple lookups but fails for four real problems:

1. **Implementation ordering** — `product feature next` uses phase labels to determine what to implement next. Phase labels are human-assigned approximations of dependency order. A feature in phase 2 may depend on an incomplete feature in phase 1, but phase ordering cannot express or detect this. The correct implementation order is determined by the *dependency structure* of the feature graph, not by human-assigned integers.

2. **Context depth** — context bundles are assembled at exactly 1 hop from the seed feature. An agent implementing a feature that shares foundational ADRs with adjacent features has no way to discover that adjacency without querying each feature individually. Transitive context — the ADRs and tests of features this feature depends on — is often relevant but is currently invisible.

3. **Decision importance** — all ADRs in a context bundle are presented as equal. ADR-001 (Rust) is structurally foundational — it is linked to every feature. ADR-007 (checklist generation) is peripheral. An agent or engineer has no signal about which decisions to read first. This signal is latent in the graph structure but not surfaced.

4. **Change impact** — superseding or modifying an ADR has downstream consequences: features that must be re-evaluated, tests that may be invalidated, implementation work that may need to be revisited. Today the developer discovers these consequences by reading every linked file. A graph-reachability traversal can compute the full impact set in one operation.

**Decision:** Extend the graph model with four graph-theoretic capabilities:

1. **Topological sort** on a `depends-on` DAG of feature nodes — used for `product feature next` and dependency validation
2. **BFS to configurable depth** — used for `product context --depth N` to surface transitive context
3. **Betweenness centrality** on ADR nodes — used for `product graph central` to rank architectural decisions by structural importance
4. **Reverse-graph reachability** — used for `product impact` to compute the full affected set of any change

---

### Capability 1: Topological Sort, Feature Dependencies, and Phase Gates

**New edge type:** `depends-on` between Feature nodes. Declared in feature front-matter:

```yaml
---
id: FT-003
title: RDF Projection
depends-on: [FT-001, FT-002]
---
```

This edge means FT-003 cannot be correctly implemented until FT-001 and FT-002 are complete.

**Graph construction:** Feature nodes plus `depends-on` edges form a directed acyclic graph (DAG). Product validates this DAG on every invocation. A cycle (FT-001 depends-on FT-003 depends-on FT-001) is a hard error — exit code 1. Cycles represent contradictory dependency claims and cannot be resolved automatically.

**Topological sort:** Kahn's algorithm over the feature DAG produces a partial order of valid implementation sequences. `product feature next` applies a two-level gate to select the next feature:

```
for each feature F in topological order:
    if F.status == complete:                              skip
    if any depends_on predecessor is not complete:        skip
    if F.phase > 1 AND NOT phase_gate_satisfied(F.phase - 1):  skip
    return F   ← next feature to implement
```

**Phase gate (`phase_gate_satisfied(N)`):**

A phase gate is satisfied when all test criteria of type `exit-criteria` linked to features in phase N have `status: passing`. Not all features in the phase need to be complete — only the exit criteria must pass. This reflects the spec's definition of phase completion: a phase is done when its measurable exit conditions are met, not when every feature in it is perfect.

```rust
fn phase_gate_satisfied(phase: u32, graph: &Graph) -> bool {
    graph.features_in_phase(phase)
        .flat_map(|f| graph.tests_for_feature(f))
        .filter(|tc| tc.tc_type == TcType::ExitCriteria)
        .all(|tc| tc.status == TcStatus::Passing)
}
```

If no exit-criteria TCs exist for a phase, the gate is considered satisfied — a phase with no defined exit criteria is always open. This ensures backward compatibility during migration when TCs haven't been written yet.

**What `product feature next` reports when a phase gate blocks:**

```
product feature next

  Next candidate: FT-009 — Rate Limiting  [phase 2, planned]
  ✗ Phase 2 locked — Phase 1 exit criteria not all passing:

    TC-001  Binary compiles               [passing  ✓]
    TC-004  Two-node cluster forms        [passing  ✓]
    TC-007  Workload survives restart     [failing  ✗]
    TC-012  Volume allocation end-to-end  [unimplemented]

  Fix TC-007 and TC-012 to unlock Phase 2.
  To skip the gate:  product feature next --ignore-phase-gate
  To work on FT-009 directly:  product preflight FT-009
```

The `--ignore-phase-gate` flag bypasses the phase gate for the current invocation only. It does not suppress the warning. Explicit feature invocations (`product preflight FT-009`, `product context FT-009`) are always available regardless of phase gate state — the gate only applies to the automated `next` selection.

**Topological order vs. phase labels:** Phase labels carry human intent about grouping and milestones. Topological order carries structural truth about explicit dependency. The phase gate adds a third signal: phase completion readiness. All three are used together in `product feature next`. When they disagree (a phase-1 feature depends-on a phase-2 feature), `product graph check` reports W005.

**New command:** `product feature deps FT-003` — prints the full transitive dependency tree for a feature.

**`product status` with phase gate display:**

```
product status

Phase 1 — Cluster Foundation  [OPEN — exit criteria: 2/4 passing]
  FT-001  Cluster Foundation     complete
  FT-002  mTLS Node Comms        complete
  FT-003  Raft Consensus         in-progress
  FT-004  Block Storage          planned

Phase 2 — Products and IAM  [LOCKED — Phase 1 exit criteria: TC-007, TC-012 not passing]
  FT-005  Product Resource       planned
  FT-006  OIDC Provider          planned

Phase 3 — RDF and Event Store  [LOCKED — Phase 2 not yet open]
  FT-007  RDF Store              planned
```

`product status --phase 1` shows the full exit criteria detail for a single phase including which TCs are passing, failing, and unimplemented.

---

### Capability 2: BFS Context Assembly

**Current behaviour:** `product context FT-001` performs exactly 1-hop traversal:
```
FT-001 → {ADR-001, ADR-002} → (stop)
FT-001 → {TC-001, TC-002}   → (stop)
```

**New behaviour:** `product context FT-001 --depth N` performs BFS to depth N from the seed node, following all edge types in the traversal direction. Default depth is 1 (preserves current behaviour).

**Depth semantics:**

```
depth 1 (default):
  FT-001 → direct ADRs, direct tests

depth 2:
  FT-001 → direct ADRs → other features those ADRs apply to
  FT-001 → depends-on features → their ADRs and tests
  FT-001 → direct tests → (no outbound edges from tests)

depth 3:
  depth-2 nodes → their ADRs, tests, and dependencies
```

**Deduplication:** A node that appears multiple times in a BFS traversal (reachable via multiple paths) is included once in the bundle, at its first-encountered position. The bundle header `⟦Ω:Bundle⟧` lists all included artifact IDs so the agent sees the full manifest before reading content.

**Practical limit:** Depth ≥ 3 on a well-connected graph risks pulling in most of the repository. `product context --depth 3` emits a warning to stderr if the resulting bundle exceeds 50 nodes: "Bundle contains N artifacts at depth 3. Consider narrowing scope." The bundle is still produced — the warning does not block output.

**New flag on context command:**
```
product context FT-001 --depth 2     # transitive context
product context FT-001 --depth 1     # direct only (default)
```

---

### Capability 3: Betweenness Centrality

**Definition:** The betweenness centrality of a node v is the fraction of shortest paths between all pairs of nodes in the graph that pass through v. A node with high betweenness is a structural bridge — many other nodes depend on it to connect to each other.

**Application to ADRs:** ADRs that are linked to many features, and whose features are otherwise loosely connected, have high betweenness. These are the foundational decisions an engineer or agent must understand before working on any feature. ADRs that apply to a single isolated feature have low betweenness regardless of how important they feel to the author.

**Algorithm:** Brandes' algorithm. O(V·E) time complexity. On a repository with 200 nodes and 800 edges this completes in < 50ms.

**New command:**
```
product graph central                # top-10 ADRs by betweenness centrality
product graph central --top 5        # configurable N
product graph central --all          # full ranked list
```

**Output format:**
```
Rank  ID       Centrality  Title
1     ADR-001  0.847       Rust as Implementation Language
2     ADR-002  0.731       openraft for Cluster Consensus
3     ADR-006  0.612       Oxigraph for RDF Projection
4     ADR-003  0.445       Event Log Schema
5     ADR-009  0.201       CI Exit Codes
```

**Integration with context bundles:** When `--depth 1` (default), ADRs in the bundle are ordered by betweenness centrality descending, not by ID ascending. An agent reading the bundle top-to-bottom encounters the most structurally important decisions first. ID-ascending order is available via `--order id`.

**`product graph stats` output** is extended with:
```
ADR centrality: mean=0.41, max=0.847 (ADR-001), min=0.003 (ADR-007)
Structural hubs (centrality > 0.5): ADR-001, ADR-002, ADR-006
```

---

### Capability 4: Reverse-Graph Reachability (Impact Analysis)

**Reverse graph:** For every directed edge A → B in the knowledge graph, the reverse graph contains edge B → A. BFS on the reverse graph from any node returns all nodes that have a path *to* that node in the forward graph — i.e., everything that depends on it.

**`product impact` command:**
```
product impact ADR-002               # what is affected if ADR-002 changes
product impact TC-003                # what depends on this test criterion
product impact FT-001                # what depends on this feature completing
```

**Impact set composition for an ADR:**

Starting from ADR-002 in the reverse graph:
- Features that `implementedBy` ADR-002 — must be re-evaluated
- Test criteria that `validates` ADR-002 — may be invalidated
- Features that `depends-on` features linked to ADR-002 — transitively affected

**Output:**
```
Impact analysis: ADR-002 — openraft for Cluster Consensus

Direct dependents:
  Features:  FT-001 (in-progress), FT-004 (planned)
  Tests:     TC-002 (unimplemented), TC-003 (unimplemented), TC-007 (passing)

Transitive dependents (via feature dependencies):
  Features:  FT-007 (planned) — depends-on FT-001
  Tests:     TC-011 (unimplemented) — validates FT-007

Summary: 3 features, 4 tests affected. 1 passing test may be invalidated.
```

The summary line highlights passing tests that may be invalidated — these are the highest-urgency items when superseding a decision.

**Integration with ADR supersession:** When `product adr status ADR-002 superseded --by ADR-013` is run, Product automatically runs impact analysis and prints the impact summary before completing the status change. The developer sees the full blast radius before committing.

---

### Graph Model Update

The full edge type set after this ADR:

| Edge | From | To | Direction | Description |
|---|---|---|---|---|
| `implementedBy` | Feature | ADR | forward | Feature is governed by this decision |
| `validatedBy` | Feature | TestCriterion | forward | Feature is verified by this test |
| `testedBy` | ADR | TestCriterion | forward | Decision is verified by this test |
| `supersedes` | ADR | ADR | forward | This decision replaces another |
| `depends-on` | Feature | Feature | forward | Implementation dependency |

The reverse of every edge is implicit and traversed by impact analysis.

---

**Rationale:**
- Topological sort is the only correct solution to implementation ordering in a system with explicit dependencies. Phase labels cannot express partial order — two features in the same phase may have a dependency between them that phase numbers cannot represent
- BFS depth generalises context assembly without changing the default behaviour — existing workflows are unaffected unless `--depth N` is explicitly passed
- Betweenness centrality requires no human curation — the structural importance ranking falls out of the graph that already exists. It does not add any new maintenance burden
- Reverse-graph reachability is O(V+E) and trivially derived from the forward graph already in memory. The implementation cost is near zero; the operational value (knowing the blast radius of a change before making it) is high
- All four algorithms operate on graphs of the scale Product manages (< 500 nodes) in well under 100ms. There is no performance argument against any of them

**Rejected alternatives:**
- **PageRank for ADR importance** — PageRank models random-walk importance, which assumes edges represent influence or endorsement. Our edges are structural dependencies, not endorsements. Betweenness centrality correctly models structural bridging, which is the property we want.
- **Manual importance tagging on ADRs** — `importance: foundational | standard | peripheral` in front-matter. Requires human judgment and drifts over time as the graph evolves. Centrality is computed, not declared — it cannot drift.
- **Depth-limited context as default** — making depth-2 the default for `product context`. Rejected because depth-2 bundles are significantly larger and the use case (transitive context for an agent implementing a complex feature) is not the common case. Default depth-1 preserves current behaviour; opt-in depth-2 covers the complex case.
- **Full graph dump with relevance scoring** — send the entire graph to an LLM and let it select relevant nodes. Rejected because it defeats the purpose of Product: the whole point is to assemble targeted context cheaply and deterministically, not to add another LLM call to the pipeline.

**Test coverage:**

Scenario tests:
- `topo_sort_simple.rs` — three features: FT-001, FT-002 depends-on FT-001, FT-003 depends-on FT-002. Assert topological order is [FT-001, FT-002, FT-003].
- `topo_sort_parallel.rs` — FT-002 and FT-003 both depend-on FT-001, no dependency between FT-002 and FT-003. Assert FT-001 appears before both; FT-002 and FT-003 order is unspecified.
- `topo_sort_cycle.rs` — FT-001 depends-on FT-002, FT-002 depends-on FT-001. Assert `product graph check` exits with code 1 and names both features in the error message.
- `feature_next_uses_topo.rs` — FT-001 complete, FT-002 depends-on FT-001 (in-progress), FT-003 no dependencies (planned). Assert `product feature next` returns FT-002, not FT-003.
- `feature_next_phase_gate_blocks.rs` — Phase 1 has TC-007 (exit-criteria, failing). FT-005 is phase 2. Assert `product feature next` skips FT-005 and reports the phase gate with TC-007 named. Assert it returns a remaining phase-1 feature instead.
- `feature_next_phase_gate_satisfied.rs` — all phase-1 exit-criteria TCs are passing. Assert `product feature next` returns the first eligible phase-2 feature.
- `feature_next_phase_gate_no_exit_criteria.rs` — phase 1 has no exit-criteria TCs. Assert phase gate is treated as satisfied and phase-2 features are returned normally.
- `feature_next_ignore_gate.rs` — phase-1 exit criteria failing. Run `product feature next --ignore-phase-gate`. Assert a phase-2 feature is returned. Assert a warning is emitted to stderr.
- `feature_next_gate_partial.rs` — phase 1 has 4 exit-criteria TCs: 3 passing, 1 failing. Assert phase gate is NOT satisfied (all must pass). Assert stderr names only the failing TC.
- `status_shows_phase_gate.rs` — run `product status`. Assert each phase shows its gate state: `[OPEN]`, `[LOCKED]`. Assert LOCKED phases name the failing exit-criteria TCs.
- `status_phase_detail.rs` — run `product status --phase 1`. Assert output lists all exit-criteria TCs for phase 1 with their individual pass/fail status.
- `context_depth_2.rs` — FT-001 linked to ADR-002; ADR-002 also linked to FT-004; FT-004 linked to TC-009. Assert `product context FT-001 --depth 2` includes TC-009 and FT-004. Assert `product context FT-001 --depth 1` does not.
- `context_depth_dedup.rs` — two paths from FT-001 to ADR-002 (via direct link and via depends-on chain). Assert ADR-002 appears exactly once in the bundle.
- `context_bundle_adr_order_centrality.rs` — feature linked to ADR-001 (high centrality) and ADR-007 (low centrality). Assert ADR-001 appears before ADR-007 in the default bundle output.
- `centrality_computation.rs` — load a graph with known topology. Assert betweenness centrality values match hand-computed expected values within ±0.001.
- `centrality_top_n.rs` — assert `product graph central --top 3` returns exactly 3 ADRs in descending centrality order.
- `impact_direct.rs` — ADR-002 linked to FT-001 and FT-004. Assert `product impact ADR-002` reports both features in direct dependents.
- `impact_transitive.rs` — FT-007 depends-on FT-001; FT-001 linked to ADR-002. Assert `product impact ADR-002` includes FT-007 in transitive dependents.
- `impact_on_supersede.rs` — run `product adr status ADR-002 superseded --by ADR-013`. Assert impact summary is printed to stdout before the status change is committed.

Invariants:
- Topological sort must complete in O(V+E) time. Any repository with < 500 feature nodes must sort in < 10ms.
- Betweenness centrality scores must be in range [0.0, 1.0]. Any value outside this range is a computation error.
- BFS deduplication: a node ID must appear at most once in any context bundle, regardless of how many paths reach it.
- Phase gate evaluation: a phase with no exit-criteria TCs is always open. Never blocked by absence of tests.

Exit criteria:
- `product graph central` on a graph of 200 ADR nodes and 800 edges completes in < 100ms.
- `product impact ADR-001` on the full PiCloud repository completes in < 50ms.
- Topological sort on 100 features with 150 dependency edges completes in < 5ms.
- `product feature next` on the migrated PiCloud repository returns a phase-1 feature when phase-1 exit criteria are not all passing.

---

## ADR-013: Error Model and User-Facing Error Format

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

**Test coverage:**

Scenario tests:
- `error_broken_link_format.rs` — parse a feature with a broken ADR reference. Assert stderr contains the file path, line number, offending content, and a hint. Assert stdout is empty. Assert exit code 1.
- `error_json_format.rs` — run `product graph check --format json` on a repo with one error and one warning. Assert stderr is valid JSON matching the schema above. Assert the `errors` array has length 1 and `warnings` has length 1.
- `error_no_panic_on_bad_yaml.rs` — feed a file with completely invalid YAML as front-matter. Assert exit code 1, structured error on stderr, no panic.
- `error_internal_tier4.rs` — trigger a Tier 4 path via an injected fault. Assert exit code 3 and the internal error message format.
- `error_stdout_clean.rs` — run any command that produces warnings but no errors. Assert stdout contains only the command's normal output. Assert warnings are on stderr only.

Invariants:
- No user-supplied input produces a Rust `panic!`. Enforced by running the full test suite with `RUST_BACKTRACE=1` and asserting zero panics in test output.
- Every error has a code in the `E001`–`E007` / `W001`–`W006` / `I001`–`I002` range. An undocumented code is a test failure.


---

## ADR-014: Schema Versioning and Migration Path

**Status:** Accepted

**Context:** Product's front-matter schema will evolve. Fields will be added, renamed, or have their semantics clarified. A repository created with Product v0.1 may contain front-matter that Product v0.2 reads differently — silently producing wrong results — or refuses to read at all, hard-erroring on every command. Both outcomes are unacceptable for a tool that manages long-lived project artifacts.

The schema version must be machine-readable, forward-compatible by default, and upgradeable without requiring the developer to manually edit every artifact file.

**Decision:** `product.toml` carries a `schema-version` field. Product validates this on startup against its own supported schema range. Front-matter fields unknown to the current schema version are ignored with a warning (forward compatibility). Fields present in the schema but absent in a file are filled with documented defaults (backward compatibility). `product migrate schema` performs in-place upgrades when a breaking change is introduced.

---

### Schema Version in `product.toml`

```toml
name = "picloud"
schema-version = "1"          # integer, incremented on breaking changes
```

Schema version is an integer, not semver. It increments only on breaking changes — field renames, removed fields, changed semantics. Adding an optional field with a default is not a breaking change and does not increment the version.

---

### Compatibility Rules

**Forward compatibility (Product older than schema):** If `product.toml` declares `schema-version = "2"` and the running binary only supports up to version `"1"`, Product exits with error E008:

```
error[E008]: schema version mismatch
  --> product.toml
   |
 2 | schema-version = "2"
   |                  ^^^ this repository requires schema version 2
   |                      this binary supports up to schema version 1
   |
   = hint: upgrade product with `cargo install product --force`
```

**Backward compatibility (Product newer than schema):** If `product.toml` declares `schema-version = "1"` and the binary supports version `"2"`, Product runs normally but emits W007 on startup:

```
warning[W007]: schema upgrade available
  schema version 1 is supported but version 2 is current
  run `product migrate schema` to upgrade (dry-run with --dry-run)
```

This warning is suppressible with `schema-version-warning = false` in `product.toml` for repositories that have made an explicit decision to stay on an older schema.

**Unknown front-matter fields:** Fields in artifact files not recognised by the current schema are silently ignored. They are preserved on write — Product never strips fields it does not understand. This ensures that tooling built on top of Product can add custom fields to front-matter without Product destroying them.

---

### `product migrate schema` Command

```
product migrate schema              # upgrade to current schema version
product migrate schema --dry-run    # show what would change without writing
product migrate schema --from 1     # explicit source version (defaults to product.toml value)
```

The migrate command:
1. Reads `product.toml` schema version
2. Applies each migration step in sequence (1→2, 2→3, etc.)
3. Writes updated artifact files atomically (temp file + rename, see ADR-015)
4. Updates `schema-version` in `product.toml` last
5. Reports a summary: N files updated, M files unchanged

If any file write fails mid-migration, the command reports the failure and leaves `schema-version` in `product.toml` unchanged. The partially migrated files remain — they are individually valid for the new schema — but the operator is told which files were updated and which were not. Re-running `product migrate schema` is idempotent.

---

### Breaking Change Protocol

When a schema change is introduced:

1. Increment `schema-version` in the Product source
2. Write a migration function `migrate_v1_to_v2()` that transforms affected front-matter fields
3. Document the change in `CHANGELOG.md` with before/after examples
4. Add a scenario test that runs a v1 repository through the migration and asserts the v2 output
5. Keep the migration function permanently — it must be possible to upgrade from any historical version to current in one command

---

**Rationale:**
- Integer schema version is simpler than semver for this use case. Schema compatibility is binary: a field either exists and has the expected semantics, or it doesn't. Patch and minor version distinctions don't apply.
- Forward incompatibility is a hard error, not a warning. Running a new schema repository through old Product would produce silently wrong graph output — missing edges, incorrect status values. Hard error is the only safe response.
- Backward incompatibility is a warning, not an error. The old schema is still readable; it's just missing new capabilities. The developer can choose when to migrate.
- Preserving unknown fields on write is critical for extensibility. If Product stripped unrecognised fields, adding a custom field would be permanently lost on the next `product feature status` invocation.

**Rejected alternatives:**
- **Semver for schema** — over-engineered. Schema evolution for a flat YAML structure does not benefit from the three-level distinction.
- **No versioning, always latest** — the path to silent data corruption. Rejected without further consideration.
- **Per-file schema version** — each artifact file declares its own schema version. Rejected because it makes migration a per-file operation with no single point of truth. `product.toml` is the correct single source of schema version truth.

**Test coverage:**

Scenario tests:
- `schema_version_forward_error.rs` — write `schema-version = "99"` to `product.toml`. Run any command. Assert exit code 1 and error E008 with the upgrade hint.
- `schema_version_backward_warning.rs` — write `schema-version = "0"` to `product.toml` (simulating an old repo). Run `product graph check`. Assert W007 appears on stderr and the command still completes successfully.
- `schema_migrate_dry_run.rs` — run `product migrate schema --dry-run` on a v1 repo. Assert no files are modified. Assert stdout describes what would change.
- `schema_migrate_idempotent.rs` — run `product migrate schema` twice. Assert the second run reports zero files changed.
- `schema_migrate_preserves_unknown_fields.rs` — add a custom field `custom-tag: foo` to a feature file. Run `product migrate schema`. Assert `custom-tag: foo` is still present in the file after migration.
- `schema_version_mismatch_format.rs` — assert error E008 includes the file path, the declared version, the supported version, and the upgrade hint.


---

## ADR-015: File Write Safety — Atomic Writes and Advisory Locking

**Status:** Accepted

**Context:** Product mutates files in two scenarios: authoring commands (`product feature status`, `product feature link`, `product adr new`) and generation commands (`product checklist generate`, `product graph rebuild`, `product migrate schema`). Two failure modes are possible:

1. **Torn writes:** a command writes partially to a file and is interrupted (process kill, power loss, disk full). The file is left in a corrupt state — truncated YAML front-matter, incomplete markdown.

2. **Concurrent writes:** two invocations of Product run simultaneously (common in CI with parallel jobs, or a developer running a command while a CI check runs). Both read the same file, both compute updates, and the last writer silently discards the first writer's changes.

Neither failure mode is acceptable for a tool that manages long-lived project artifacts. A corrupt front-matter file silently breaks the graph. Silent data loss from concurrent writes is worse than a conflict error.

**Decision:** All file writes use atomic temp-file-plus-rename. An advisory lock on `product.toml` serialises concurrent Product invocations on the same repository. Reads never acquire the lock.

---

### Atomic Writes

Every file write in Product follows this sequence:

1. Compute the full new file content in memory
2. Write to a temporary file in the same directory: `.<filename>.product-tmp.<pid>`
3. `fsync` the temporary file
4. Rename the temporary file to the target filename (atomic on POSIX systems)
5. On failure at any step: delete the temporary file, surface error E009

```rust
fn write_file_atomic(path: &Path, content: &str) -> Result<(), ProductError> {
    let tmp = path.with_file_name(format!(
        ".{}.product-tmp.{}",
        path.file_name().unwrap().to_str().unwrap(),
        std::process::id()
    ));
    std::fs::write(&tmp, content)?;
    // fsync before rename
    let file = std::fs::File::open(&tmp)?;
    file.sync_all()?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}
```

Rename is atomic on all POSIX filesystems. On Windows (where rename over an existing file requires an explicit move), `std::fs::rename` is used with the understanding that Windows atomic rename semantics differ; a Windows-specific implementation may be needed if Windows support is added.

---

### Advisory Lock

Product acquires an exclusive advisory lock on a `.product.lock` file in the same directory as `product.toml` before any write operation. The lock is released on process exit (including on signal).

Read-only commands (`product feature list`, `product context`, `product graph check`) do not acquire the lock.

Write commands acquire the lock with a **3-second timeout**. If the lock is not acquired within 3 seconds, Product exits with error E010:

```
error[E010]: repository locked
  another Product process is running on this repository
  lock held by PID 48291 (started 2026-04-11T09:14:22Z)
  wait for it to complete, or delete .product.lock if the process has died
```

The lock file contains the PID and start time of the holding process, enabling the error message to be informative. If the holding PID is not running (stale lock from a crashed process), Product detects this and acquires the lock without the timeout.

**Implementation:** `fd-lock` crate — cross-platform advisory file locking with no external dependencies.

---

### Temporary File Cleanup

On startup, Product scans for `*.product-tmp.*` files in the repository directories and deletes them. These are always safe to delete — they are either complete (and were renamed) or incomplete (and should be discarded). This handles the case where a previous invocation was killed after writing the temp file but before the rename.

---

**Rationale:**
- Atomic rename is the standard POSIX pattern for safe file writes. It is used by git, package managers, and text editors for exactly this reason. Implementing it in Product follows established practice.
- Advisory locking is advisory — a non-Product process can still modify files. This is intentional: Product should not prevent editors, git operations, or other tools from working. It only serialises concurrent Product invocations.
- The 3-second lock timeout is short enough to fail fast (a developer running two commands simultaneously gets an immediate error, not a hang) but long enough to survive brief system load spikes.
- Stale lock detection (PID not running) prevents the lock file from becoming a permanent block after a crash. The developer should not need to manually delete `.product.lock` under normal failure conditions.

**Rejected alternatives:**
- **No locking, accept last-write-wins** — silent data loss. Rejected.
- **Exclusive lock on every file written** — too granular. Would require acquiring N locks for a command that writes N files, with partial failure and rollback complexity.
- **SQLite as the write store** — SQLite handles locking internally. Rejected because it would make all artifact files non-human-editable binary blobs, contradicting the foundational design decision (ADR-002).
- **Process mutex via socket** — more reliable than file locking on some systems. Rejected because it requires a listening socket and introduces a cleanup problem on process death.

**Test coverage:**

Scenario tests:
- `atomic_write_content.rs` — write a feature file atomically. Assert the file contains the expected content. Assert no `.product-tmp.*` files remain.
- `atomic_write_interrupted.rs` — simulate a write failure after temp file creation (inject error before rename). Assert the target file is unchanged. Assert the temp file is deleted.
- `lock_concurrent_writes.rs` — spawn two Product processes simultaneously, both running `product feature status FT-001 complete`. Assert exactly one succeeds and the other exits with E010. Assert the file contains a valid result from exactly one process.
- `lock_stale_cleanup.rs` — create a `.product.lock` file with a non-existent PID. Run any write command. Assert the command succeeds (stale lock was detected and cleared).
- `tmp_cleanup_on_startup.rs` — create leftover `.product-tmp.*` files. Run `product feature list` (read-only). Assert the tmp files are deleted on startup.


---

## ADR-016: Formal Block Grammar

**Status:** Accepted

**Context:** ADR-011 defines the AISP-influenced formal block notation for test criteria files. It specifies the block types and symbol subset but defers the question of how blocks are parsed. Without a defined grammar, two implementations of the parser may accept different inputs, the error messages for malformed blocks will be inconsistent, and the Rust type model for parsed formal blocks will be ambiguous.

This ADR defines the grammar, the Rust type model, and the error behaviour for malformed blocks.

**Decision:** Formal blocks are parsed as structured text using a hand-written recursive descent parser over the minimal symbol subset defined in ADR-011. The parser produces a typed AST. Blocks that fail to parse are reported as E001 parse errors with line-level precision. Blocks that are syntactically valid but semantically meaningless (e.g., an empty `⟦Γ:Invariants⟧{}` block) produce W004 warnings.

---

### Grammar (informal BNF)

```
formal-section   ::= block*
block            ::= "⟦" block-type "⟧" "{" block-body "}"
                   | evidence-block
block-type       ::= "Σ:Types" | "Γ:Invariants" | "Λ:Scenario"
                   | "Λ:ExitCriteria" | "Λ:Benchmark"
block-body       ::= statement ( "\n" statement )*
statement        ::= type-def | invariant | scenario-field | exit-field | benchmark-field

benchmark-field  ::= "baseline" "≜" "condition" "(" ident ")"
                   | "target"   "≜" "condition" "(" ident ")"
                   | "scorer"   "≜" "rubric_llm" "(" scorer-params ")"
                   | "pass"     "≜" expr
scorer-params    ::= ident ":" literal ("," ident ":" literal)*

type-def         ::= ident "≜" type-expr
type-expr        ::= ident | union-type | tuple-type | list-type | func-type
union-type       ::= type-expr "|" type-expr
tuple-type       ::= "⟨" type-expr ("," type-expr)* "⟩"
list-type        ::= type-expr "+"       (* one or more *)
                   | type-expr "*"       (* zero or more *)
func-type        ::= type-expr "→" type-expr

invariant        ::= quantifier | comparison
quantifier       ::= ("∀" | "∃") binding ":" expr
binding          ::= ident | ident "∈" ident
expr             ::= ident | literal | func-call | infix | set-expr
infix            ::= expr ("=" | "≠" | "<" | ">" | "≤" | "≥" | "∧" | "∨") expr
set-expr         ::= "|" "{" expr "|" expr "}" "|"   (* set cardinality *)
func-call        ::= ident "(" expr ("," expr)* ")"
comparison       ::= expr ("=" | "≠" | "<" | ">") expr

scenario-field   ::= ("given" | "when" | "then") "≜" expr
exit-field       ::= ident comparison

evidence-block   ::= "⟦Ε⟧" "⟨" evidence-fields "⟩"
evidence-fields  ::= evidence-field (";" evidence-field)*
evidence-field   ::= "δ≜" float | "φ≜" integer | "τ≜" stability
stability        ::= "◊⁺" | "◊⁻" | "◊?"

ident            ::= [A-Za-z_][A-Za-z0-9_]*
literal          ::= integer | float | string | duration
integer          ::= [0-9]+
float            ::= [0-9]+ "." [0-9]+
string           ::= '"' [^"]* '"'
duration         ::= integer ("s" | "ms" | "min" | "h")
```

The grammar is intentionally permissive on `expr` — the goal is structural validation and AST construction, not full formal verification. An expression that parses but cannot be evaluated is not an error; it is simply stored as a string in the AST leaf.

---

### Rust Type Model

```rust
pub enum FormalBlock {
    Types(Vec<TypeDef>),
    Invariants(Vec<Invariant>),
    Scenario(ScenarioBlock),
    ExitCriteria(Vec<ExitField>),
    Benchmark(BenchmarkBlock),
    Evidence(EvidenceBlock),
}

pub struct TypeDef {
    pub name: String,
    pub expr: TypeExpr,
}

pub enum TypeExpr {
    Named(String),
    Union(Box<TypeExpr>, Box<TypeExpr>),
    Tuple(Vec<TypeExpr>),
    List(Box<TypeExpr>, Multiplicity),
    Func(Box<TypeExpr>, Box<TypeExpr>),
}

pub struct ScenarioBlock {
    pub given: Option<String>,   // stored as raw expression string
    pub when: Option<String>,
    pub then: Option<String>,
}

pub struct BenchmarkBlock {
    pub baseline:    String,          // condition name, e.g. "none"
    pub target:      String,          // condition name, e.g. "product"
    pub scorer:      ScorerConfig,
    pub pass:        String,          // raw pass expression, stored verbatim
}

pub struct ScorerConfig {
    pub kind:        String,          // e.g. "rubric_llm"
    pub params:      Vec<(String, String)>,
}

pub struct EvidenceBlock {
    pub delta: f64,              // δ — confidence [0.0, 1.0]
    pub phi: u8,                 // φ — coverage [0, 100]
    pub tau: Stability,
}

pub enum Stability { Stable, Unstable, Unknown }

pub struct Invariant {
    pub raw: String,             // stored verbatim for context bundle output
    pub quantifier: Option<Quantifier>,
}
```

`Invariant.raw` stores the original text verbatim. The AST is used for validation; the raw string is used for context bundle output. This ensures the bundle output matches exactly what the author wrote, without any round-trip formatting changes.

---

### Parse Error Behaviour

Block delimiter errors (unclosed `⟦`, unrecognised block type) are E001 errors — the file cannot be processed further.

Malformed content inside a block (invalid expression, missing `≜`) is E001 on the specific line. The rest of the block is skipped; subsequent blocks in the same file are still parsed.

An empty block body (`⟦Γ:Invariants⟧{}`) is W004 — syntactically valid but semantically meaningless.

An evidence block with `δ` outside [0.0, 1.0] or `φ` outside [0, 100] is E001.

---

### Opaque Storage for Context Bundles

Formal blocks are stored as both a parsed AST (for validation) and the original raw text (for bundle output). The raw text is extracted between the outer `{...}` delimiters and preserved byte-for-byte. This means whitespace, comments, and any content that parses but is not fully modelled in the AST is round-tripped faithfully.

**Rationale:**
- Hand-written recursive descent is the right tool for a small, well-defined grammar with good error recovery requirements. Parser combinator libraries (nom, pest) add compile-time complexity for a grammar this size without meaningful benefit.
- Storing `Invariant.raw` verbatim rather than pretty-printing from the AST ensures the context bundle output matches the author's intent exactly — Product is a context assembly tool, not a formatter.
- The grammar is intentionally permissive on expressions. Full semantic validation of formal expressions is not Product's job — that belongs to the agent or tool consuming the bundle.

**Rejected alternatives:**
- **Treat formal blocks as opaque strings (no parsing)** — simplest implementation. Rejected because it removes the ability to validate evidence block ranges, detect empty blocks (W004), or surface parse errors with line precision. The grammar provides validation without requiring full semantic analysis.
- **Pest PEG parser** — clean grammar definition, good error messages. Rejected because it adds a build-time dependency and a `.pest` file to maintain. For a grammar this small, the overhead is not justified.
- **Regex-based extraction** — extract block content with regex patterns. Rejected because nested `⟨⟩` and `{}` delimiters cannot be correctly parsed with regex. A recursive descent parser is required for correct delimiter matching.

**Test coverage:**

Scenario tests:
- `parse_types_block.rs` — parse `⟦Σ:Types⟧{ Node≜IRI; Role≜Leader|Follower }`. Assert two `TypeDef` entries with correct names and union type structure.
- `parse_invariants_block.rs` — parse a block with a universal quantifier. Assert `Invariant.raw` matches the input verbatim.
- `parse_scenario_block.rs` — parse a `⟦Λ:Scenario⟧` block with all three fields. Assert `given`, `when`, `then` are all populated.
- `parse_evidence_block.rs` — parse `⟦Ε⟧⟨δ≜0.95;φ≜100;τ≜◊⁺⟩`. Assert `delta=0.95`, `phi=100`, `tau=Stable`.
- `parse_evidence_delta_out_of_range.rs` — parse `⟦Ε⟧⟨δ≜1.5;φ≜100;τ≜◊⁺⟩`. Assert E001 with the file path, line number, and the out-of-range value.
- `parse_unclosed_delimiter.rs` — parse a file with `⟦Γ:Invariants⟧{ ... ` (no closing `}`). Assert E001 with line number. Assert subsequent blocks in the same file are still parsed.
- `parse_empty_block_warning.rs` — parse `⟦Γ:Invariants⟧{}`. Assert W004. Assert no error.
- `parse_raw_roundtrip.rs` — parse an invariant block and assert that `Invariant.raw` is byte-for-byte identical to the original input (including whitespace).
- `parse_unknown_block_type.rs` — parse `⟦X:Unknown⟧{ ... }`. Assert E001 with "unrecognised block type".


---

## ADR-017: Migration Command Specification

**Status:** Accepted

**Context:** `product migrate from-prd` and `product migrate from-adrs` were listed in the phase plan without specification. These are the highest-risk commands in Product: they read freeform markdown prose and write many new files based on heuristic parsing. Unlike all other Product commands, they produce artifacts that require human review — the parser cannot determine intent with certainty from unstructured input.

The migration command must be specified completely before implementation: what heuristics it uses, what output it produces, what rollback story exists, and how the developer confirms and corrects the output.

**Decision:** Migration is a two-phase process: **extraction** (parse the source document, propose a set of artifacts) then **confirmation** (developer reviews and commits). No files are written until the developer explicitly confirms. Extraction is deterministic given a document; there is no ambiguous state. All extracted artifacts are written atomically as a batch.

---

### Supported Source Documents

**`product migrate from-prd SOURCE.md`** — parses a monolithic PRD document. Detects features from heading structure.

**`product migrate from-adrs SOURCE.md`** — parses a monolithic ADR document. Detects individual ADRs and extracts test criteria from ADR subsections.

Both commands accept `--validate` for dry-run output without writing files.

---

### Extraction Heuristics: PRD → Features

The parser scans for H2 (`##`) headings that match feature-like patterns. A heading is treated as a feature boundary if it:
- Is at H2 level
- Does not match a set of known non-feature headings: `Vision`, `Goals`, `Non-Goals`, `Target Environment`, `Core Architecture`, `Open Questions`, `Resolved Decisions`, `Phase Plan`, `Overview`, `Introduction`, `Background`, `References`

For each candidate feature heading:
- `title` is the heading text, stripped of leading numbers and punctuation (`## 5. Products and IAM` → `Products and IAM`)
- `phase` is inferred from the nearest preceding `### Phase N` heading, or 1 if none found
- `status` is `planned` by default
- `depends-on` is empty — not inferred (requires human judgment)
- `adrs` and `tests` are empty — not linked (requires `product graph check` to identify gaps)

The body of the section (all content until the next H2) becomes the feature file body.

**Checklist inference:** If the source PRD contains a checklist section (lines matching `- [ ]` or `- [x]`), checked items set the corresponding feature `status` to `complete`, unchecked items remain `planned`. This handles migration from an existing `checklist.md`.

---

### Extraction Heuristics: ADRs → ADR Files + Test Criteria

The parser scans for H2 (`##`) headings matching the pattern `ADR-NNN:` or `## ADR-NNN`.

For each ADR:
- `id` is extracted from the heading prefix
- `title` is the heading text after the prefix
- `status` is extracted from a `**Status:**` line in the body (`Accepted`, `Proposed`, etc.)
- `supersedes` and `superseded-by` are extracted from `**Supersedes:**` / `**Superseded By:**` lines
- `features` is empty — not inferred

**Test criteria extraction:** Within each ADR body, the parser looks for subsections matching these heading patterns:
- `### Test coverage`, `### Tests`, `### Test Coverage`
- `### Exit criteria`, `### Exit Criteria`

Within these subsections, the parser identifies individual test items by:
- Bullet points beginning with a test name pattern
- Sub-headings (`####`) within the test section

For each extracted test item:
- `title` is the bullet or sub-heading text
- `type` is inferred: `exit-criteria` if from an exit criteria subsection; `scenario`, `invariant`, or `chaos` from keyword matching in the title (e.g., "chaos" → `chaos`, "invariant" → `invariant`, otherwise `scenario`)
- `status` is `unimplemented`
- `validates.adrs` contains the parent ADR ID
- `validates.features` is empty
- Formal blocks are not generated — the prose content becomes the Description section body

---

### Output Format (Dry-Run and Confirmation)

`product migrate from-adrs picloud-adrs.md --validate` produces:

```
Migration plan: picloud-adrs.md → 9 ADRs, 34 test criteria

ADR files to create:
  docs/adrs/ADR-001-rust-language.md                (status: accepted)
  docs/adrs/ADR-002-openraft-consensus.md            (status: accepted)
  ... (7 more)

Test criteria files to create:
  docs/tests/TC-001-binary-compiles.md               (type: exit-criteria, adr: ADR-001)
  docs/tests/TC-002-raft-leader-election.md           (type: scenario, adr: ADR-002)
  ... (32 more)

Warnings:
  [W008] ADR-003: status not found, defaulting to "proposed"
  [W009] ADR-007: no test subsection found — no test criteria extracted

Conflicts:
  docs/adrs/ADR-001-rust-language.md already exists — will skip (use --overwrite to replace)

Run without --validate to create these files.
Run with --interactive for per-artifact confirmation.
```

---

### Execution Modes

**`--validate`** (default safe mode) — prints the migration plan and exits. No files written.

**`--execute`** — writes all proposed files. Skips files that already exist. Reports skipped files.

**`--overwrite`** — writes all proposed files. Overwrites files that already exist. Requires explicit confirmation prompt unless `--yes` is also passed.

**`--interactive`** — for each proposed artifact, prints the proposed front-matter and first 200 characters of body, then prompts: `[a]ccept / [e]dit / [s]kip / [q]uit`. `edit` opens the proposed content in `$EDITOR`. This mode is recommended for first migration of a large document.

---

### Rollback

Migration writes files atomically (ADR-015). If any write fails mid-batch, the error is reported and the remaining files are not written. Already-written files are not rolled back — they are valid artifact files. The developer can delete them manually or run migration again with `--overwrite`.

`product migrate` never modifies the source document. The source PRD and ADR files are read-only inputs.

`product migrate` never modifies `product.toml` or `checklist.md`. These are updated by `product checklist generate` after migration.

---

### Post-Migration Workflow

After migration, the recommended workflow is:

```bash
product migrate from-adrs picloud-adrs.md --execute
product migrate from-prd picloud-prd.md --execute
product graph check          # surfaces all broken links (features with no ADRs, etc.)
# manually add feature→ADR links based on graph check output
product feature link FT-001 --adr ADR-001 --adr ADR-002  # repeat per feature
product graph check          # should now exit 0 or 2 (warnings only)
product migrate link-tests   # infer TC→Feature links transitively through ADR links
product graph check          # W002 warnings reduce significantly
product checklist generate
```

`product graph check` after migration will always produce warnings (W001 orphaned ADRs, W002 features with no tests, etc.) because feature→ADR link edges require manual review. The developer fills these in using `product feature link`. Once feature→ADR links are confirmed, `product migrate link-tests` infers the transitive TC→Feature links automatically — see ADR-027.

---

**Rationale:**
- Two-phase extraction → confirmation prevents the most dangerous failure mode: writing 40 files and discovering the heuristics got 10 of them wrong. With `--validate`, the developer sees the full plan before committing.
- `--interactive` mode is the recommended path for a first migration. It forces a review of each artifact, which is valuable because the developer catches heuristic errors and also re-familiarises themselves with the content as it is being structured.
- Not inferring `depends-on` edges or feature→ADR links is correct. These relationships require semantic understanding of the content, not pattern matching on structure. Guessing wrong would be worse than leaving them empty.
- Preserving the source document unchanged means migration can be re-run safely if the first attempt was wrong. The source is always the ground truth.

**Rejected alternatives:**
- **Infer feature→ADR links from ADR body mentions of feature names** — too fragile. ADR prose mentions feature concepts by name but not by ID. Mismatches would require more cleanup than just linking manually.
- **Write all files immediately, provide `product migrate undo`** — rollback is complex in a file system context. The `--validate` → `--execute` two-phase approach achieves the same safety without requiring an undo log.
- **LLM-assisted migration** — use an LLM to interpret the PRD and generate structured artifacts. Would produce higher-quality extraction for ambiguous documents. Rejected for v1: Product must work without network access or API keys. Can be added as `product migrate --ai` in a future version.

**Test coverage:**

Scenario tests:
- `migrate_prd_heading_detection.rs` — parse a PRD with 5 H2 sections, 2 of which are `Goals` and `Non-Goals` (excluded). Assert exactly 3 feature files are proposed.
- `migrate_prd_phase_inference.rs` — parse a PRD with `### Phase 1` and `### Phase 2` headings. Assert features under each phase heading get the correct `phase` value.
- `migrate_prd_checklist_status.rs` — parse a PRD with a checklist section where 3 items are checked. Assert the 3 corresponding feature files have `status: complete`.
- `migrate_adrs_id_extraction.rs` — parse an ADR file with `## ADR-001: Rust` and `## ADR-002: openraft`. Assert exactly 2 ADR files are proposed with IDs `ADR-001` and `ADR-002`.
- `migrate_adrs_test_extraction.rs` — parse an ADR with a `### Test coverage` subsection containing 4 bullet points. Assert 4 test criterion files are proposed with `validates.adrs: [ADR-XXX]`.
- `migrate_adrs_test_type_inference.rs` — assert bullets containing "chaos" produce `type: chaos`, bullets containing "invariant" produce `type: invariant`, and others produce `type: scenario`.
- `migrate_validate_no_write.rs` — run `product migrate from-prd --validate`. Assert zero files are created in the repository.
- `migrate_execute_skips_existing.rs` — create `ADR-001-rust-language.md` before running migration. Assert the file is skipped and the skip is reported. Assert the original file content is unchanged.
- `migrate_interactive_skip.rs` — run `product migrate --interactive`, respond `s` (skip) to all prompts. Assert zero files are created.
- `migrate_source_unchanged.rs` — run `product migrate from-prd PRD.md --execute`. Assert the source `PRD.md` is byte-for-byte identical before and after the command.
- `migrate_picloud_prd.rs` — integration test: run migration against the actual PiCloud PRD. Assert at least 10 feature files are created. Assert `product graph check` exits with 2 (warnings) not 1 (errors) after migration (no broken links, only coverage gaps).


---

## ADR-018: Testing Strategy — Property-Based, Integration, and LLM Benchmark

**Status:** Accepted

**Context:** Product has three distinct failure classes that require three distinct testing approaches:

1. **Algorithmic correctness** — graph algorithms (topological sort, betweenness centrality, BFS, reachability) and the front-matter parser must produce correct results for all valid inputs, not just the ones the test author thought to write. Unit tests on hand-crafted inputs cannot cover the boundary cases that distributed systems and parser edge cases produce.

2. **Command correctness** — the full CLI surface (argument parsing, file I/O, error formatting, exit codes, stdout/stderr separation) must behave correctly on real repository state. Algorithmic unit tests cannot catch bugs in how the CLI routes a subcommand, formats a diagnostic message, or handles a concurrent write.

3. **Value delivery** — the core claim of Product is that context bundles improve LLM implementation quality. This claim is currently unvalidated. If context bundles do not measurably improve agent outputs, the product's fundamental design assumption is wrong and must be revised.

No single testing approach covers all three. This ADR specifies all three, defines their scope boundaries, and assigns them to phases.

---

### Design 1: Property-Based Testing (proptest)

**Target failure class:** Algorithmic correctness — inputs the test author did not anticipate.

**Tool:** `proptest` crate. Generates thousands of random inputs satisfying user-defined strategies, shrinks failing inputs to minimal reproducible examples.

**Scope:** Pure functions only — graph construction, traversal algorithms, front-matter parser, file write logic. No filesystem, no CLI, no network.

**Repository location:** `tests/property/` — separate from unit tests to allow independent execution and longer run budgets.

#### Generators

```rust
/// Generates a valid DAG of Feature nodes.
/// Only adds edges from lower-index to higher-index nodes,
/// guaranteeing acyclicity by construction.
fn arb_dag(
    size: impl Strategy<Value = usize>,
    edge_density: f64,
) -> impl Strategy<Value = FeatureGraph>

/// Generates a connected graph — required for centrality to be meaningful.
fn arb_connected_graph(
    size: impl Strategy<Value = usize>,
    density: f64,
) -> impl Strategy<Value = FeatureGraph>

/// Generates syntactically valid Feature structs.
/// IDs are valid format, statuses are valid enum values,
/// phases are in 1..=10. Does NOT generate broken links.
fn arb_valid_feature() -> impl Strategy<Value = Feature>

/// Generates arbitrary byte strings including edge cases:
/// empty string, valid UTF-8, invalid UTF-8, lone delimiters,
/// extremely long strings, YAML injection attempts.
fn arb_arbitrary_input() -> impl Strategy<Value = String>

/// Generates a valid YAML key-value pair not in the Product schema.
fn arb_unknown_field() -> impl Strategy<Value = (String, String)>
```

#### Property Set

**Parser robustness (from ADR-013):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P001 | No input causes a panic | `∀s:String: parse_frontmatter(s) ≠ panic` |
| TC-P002 | Valid front-matter round-trips | `∀f:Feature: parse(serialise(f)) = f` |
| TC-P003 | Unknown fields preserved on write | `∀f:Feature, k:UnknownField: serialise(inject(f,k)) ⊇ k` |
| TC-P004 | Malformed input returns structured error | `∀s:InvalidYAML: parse(s) = Err(E001)` |

**Graph algorithm correctness (from ADR-012):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P005 | Topo order respects all dependency edges | `∀g:DAG, (u,v)∈g.edges: pos(topo(g),u) < pos(topo(g),v)` |
| TC-P006 | Topo sort detects all cycles | `∀g:CyclicGraph: topo_sort(g) = Err(E003)` |
| TC-P007 | Centrality always in range | `∀g:ConnectedGraph, n∈g.nodes: 0.0 ≤ centrality(g,n) ≤ 1.0` |
| TC-P008 | Reverse reachability inverts forward | `∀g:Graph, u,v∈g.nodes: reachable(g,u,v) ↔ reachable(rev(g),v,u)` |
| TC-P009 | BFS deduplication — node appears once | `∀g:Graph, seed:Node, d:Depth: |{n \| n∈bfs(g,seed,d)}| = |bfs(g,seed,d)|` |

**File write safety (from ADR-015):**

| TC | Property | Formal expression |
|---|---|---|
| TC-P010 | Atomic write — no torn state | `∀content:String, cutAt:Offset: file_after_interrupt(cutAt) ∈ {original, new}` |
| TC-P011 | Write + re-read is identity | `∀content:String: read(atomic_write(path, content)) = content` |

**Configuration:**

```toml
# .proptest-regressions are committed — shrunk failing cases are permanent regression tests
[proptest]
cases = 1000          # default per property
max_shrink_iters = 500
failure_persistence = "file"   # .proptest-failures/
```

---

### Design 2: Integration Test Harness

**Target failure class:** Command correctness — full CLI behaviour on real repository state.

**Scope:** Full binary execution. Every test runs the compiled `product` binary against a real temporary directory. No mocking.

**Repository location:** `tests/integration/`

#### Harness API

```rust
pub struct Harness {
    dir: TempDir,
    bin: PathBuf,    // path to compiled product binary
}

impl Harness {
    pub fn new() -> Self
    pub fn write(&self, path: &str, content: &str) -> &Self
    pub fn run(&self, args: &[&str]) -> Output
    pub fn read(&self, path: &str) -> String
    pub fn exists(&self, path: &str) -> bool
    pub fn file_mtime(&self, path: &str) -> SystemTime
}

pub struct Output {
    pub stdout:    String,
    pub stderr:    String,
    pub exit_code: i32,
}

impl Output {
    pub fn assert_exit(&self, code: i32) -> &Self
    pub fn assert_stderr_contains(&self, s: &str) -> &Self
    pub fn assert_stderr_matches_error(&self, code: &str) -> &Self
    pub fn assert_stdout_clean(&self) -> &Self   // no YAML, no front-matter
    pub fn assert_json_stderr(&self) -> Value    // parse and return
    pub fn assert_no_tmp_files(&self) -> &Self
}
```

#### Fixture Library

Standard repository configurations defined once, composed freely:

```rust
pub fn fixture_minimal() -> Harness           // 1 feature, 1 ADR, linked
pub fn fixture_broken_link() -> Harness       // feature references non-existent ADR
pub fn fixture_dep_cycle() -> Harness         // FT-001 ↔ FT-002 cycle
pub fn fixture_supersession_cycle() -> Harness // ADR-001 ↔ ADR-002 cycle
pub fn fixture_orphaned_adr() -> Harness      // ADR with no feature links
pub fn fixture_phase_1_complete() -> Harness  // all phase-1 features complete
pub fn fixture_full_picloud() -> Harness      // migrated PiCloud repo (generated once, committed)
```

#### Scenario Test Set

**Error model (ADR-013):**

| TC | Fixture | Command | Asserts |
|---|---|---|---|
| IT-001 | broken_link | `graph check` | exit 1, E002 on stderr, file+line, hint |
| IT-002 | broken_link | `graph check --format json` | exit 1, valid JSON, `errors[0].code="E002"` |
| IT-003 | minimal | `graph check` | exit 0, stdout empty |
| IT-004 | orphaned_adr | `graph check` | exit 2, W001 on stderr |
| IT-005 | minimal | `context FT-001` | exit 0, stdout contains `⟦Ω:Bundle⟧`, no `---` delimiters |
| IT-006 | minimal | `context FT-001 > file` | file created, stderr empty |
| IT-007 | dep_cycle | `graph check` | exit 1, E003 names both features |
| IT-008 | any | bad YAML in feature file | exit 1, E001, no panic |

**Concurrent writes (ADR-015):**

| TC | Setup | Asserts |
|---|---|---|
| IT-009 | Two threads call `feature status FT-001` simultaneously | Exactly one exits 0, one exits 1 (E010). File valid. |
| IT-10 | Stale `.product.lock` (dead PID) | Next write command succeeds, lock cleared |
| IT-11 | Write interrupted at byte N (simulated) | File is either original or new content — never partial |

**Schema versioning (ADR-014):**

| TC | Setup | Asserts |
|---|---|---|
| IT-12 | `schema-version = "99"` | exit 1, E008, upgrade hint |
| IT-13 | `schema-version = "0"` (old) | exit 0, W007 on stderr, command completes |
| IT-14 | `migrate schema --dry-run` on old repo | exit 0, no files changed, stdout describes plan |
| IT-15 | `migrate schema` twice | Second run: exit 0, "0 files changed" |

**Migration (ADR-017):**

| TC | Source | Asserts |
|---|---|---|
| IT-16 | picloud-prd.md `--validate` | exit 0, zero files created, stdout shows plan |
| IT-17 | picloud-adrs.md `--execute` | exit 0, ≥9 ADR files, ≥30 TC files created |
| IT-18 | picloud-prd.md source unchanged | source file byte-identical before/after |
| IT-19 | picloud-prd.md then `graph check` | exit 2 (warnings only, no broken links) |

#### Golden File Tests

Migration output is verified against committed golden files. Intentional heuristic changes require `UPDATE_GOLDEN=1 cargo test` and a reviewed diff.

```
tests/
  fixtures/
    picloud-prd.md
    picloud-adrs.md
  golden/
    features/
      FT-001-cluster-foundation.md
      ...
    adrs/
      ADR-001-rust-language.md
      ...
    tests/
      TC-001-binary-compiles.md
      ...
```

---

### Design 3: LLM Context Quality Benchmark

**Target failure class:** Value delivery — does Product actually improve LLM implementation quality?

**Scope:** End-to-end quality measurement. Runs the compiled binary to generate context bundles, sends them to an LLM, scores the output against a rubric using a separate LLM call.

**Repository location:** `benchmarks/`

**Run cadence:** Not in CI. Triggered manually on release candidates, after context bundle format changes (ADR-006, ADR-011, ADR-012), and monthly for trend tracking.

#### Repository Layout

```
benchmarks/
  runner/
    src/main.rs          ← benchmark runner binary
  tasks/
    task-001-raft-leader-election/
      prompt.md
      rubric.md
    task-002-frontmatter-parser/
      prompt.md
      rubric.md
    task-003-context-bundle-assembly/
      prompt.md
      rubric.md
  results/
    2026-04-11/
      results.json
      results.md         ← human-readable summary
    latest -> 2026-04-11/
```

#### Task Structure

Each task defines a realistic implementation request grounded in the PiCloud project:

**`prompt.md`** — the implementation request, stated without embedded context:
```markdown
Implement the Raft leader election logic for PiCloud's cluster foundation.
The implementation must satisfy the platform's architectural constraints
and pass the defined test criteria for this feature.
```

**`rubric.md`** — binary-scored criteria only. No holistic judgments.
```markdown
# Rubric: Raft Leader Election

## Correctness (weight: 3)
- Uses openraft crate, not a custom Raft implementation
- Implements RaftStorage trait
- Leader election completes within 10s timeout
- Exactly one node holds Leader role at any time
- RDF graph reflects leader identity via picloud:hasRole

## Architecture (weight: 2)
- Implementation is in Rust
- No unwrap() calls on production paths
- No unsafe blocks outside marked modules

## Test coverage (weight: 2)
- Includes a scenario test for leader election
- Includes a chaos test for leader failover
- Invariant checked at test boundaries
```

#### Three Conditions

Every task is run under three context conditions:

| Condition | Context provided |
|---|---|
| `none` | No context beyond the prompt |
| `naive` | Full `picloud-prd.md` + `picloud-adrs.md` concatenated |
| `product` | Output of `product context FT-001 --depth 2` |

#### Scoring Protocol

Each rubric criterion is scored by a separate LLM call with a narrow binary question to minimise scorer variance:

```
"Does the following implementation satisfy this criterion:
'Uses openraft crate, not a custom Raft implementation'?
Answer only YES or NO.

Implementation:
[implementation text]"
```

Final score = Σ(satisfied_criteria × weight) / Σ(all_criteria × weight)

Each condition is run N=5 times at temperature=0. The reported score is the mean across runs.

#### Pass Thresholds

A benchmark TC passes when:
- `score(product) ≥ 0.80` — absolute quality threshold
- `score(product) - score(naive) ≥ 0.15` — Product must add measurable value over naive context

Both conditions must hold. A high product score on an easy task where naive also scores high does not constitute a pass.

#### Result Format

```json
{
  "run_date": "2026-04-11T09:00:00Z",
  "product_version": "0.1.0",
  "model": "claude-sonnet-4-6",
  "runs_per_condition": 5,
  "tasks": [
    {
      "id": "task-001-raft-leader-election",
      "tc": "TC-030",
      "conditions": {
        "none":    { "mean": 0.41, "scores": [0.39, 0.41, 0.44, 0.40, 0.41] },
        "naive":   { "mean": 0.63, "scores": [0.61, 0.65, 0.63, 0.62, 0.64] },
        "product": { "mean": 0.87, "scores": [0.85, 0.89, 0.86, 0.88, 0.87] }
      },
      "delta_product_vs_naive": 0.24,
      "pass": true
    }
  ],
  "aggregate": {
    "mean_product_score": 0.87,
    "mean_delta_vs_naive": 0.21,
    "tasks_passing": 3,
    "tasks_total": 3
  }
}
```

Results are committed to the repository. The trend across runs is the primary signal — a declining `delta_product_vs_naive` over releases indicates context bundle quality is regressing.

#### Initial Task Set (Phase 3)

Three tasks covering the three most important features:

| TC | Task | Feature | Key rubric dimension |
|---|---|---|---|
| TC-030 | Raft leader election | FT-001 | Architecture compliance (openraft, RDF) |
| TC-031 | Front-matter parser | FT-Product-001 | Robustness (no panics, error codes) |
| TC-032 | Context bundle assembly | FT-Product-002 | Correctness (depth, dedup, ordering) |

---

### Testing Phase Assignment

| Design | Phase | Prerequisite |
|---|---|---|
| Integration harness infrastructure | Phase 1 | Binary compiles |
| Integration: error model tests (IT-001–IT-008) | Phase 1 | `graph check` implemented |
| Integration: concurrency tests (IT-009–IT-11) | Phase 1 | Write commands implemented |
| Property: parser robustness (TC-P001–TC-P004) | Phase 1 | Parser implemented |
| Integration: schema tests (IT-12–IT-15) | Phase 2 | Schema versioning implemented |
| Integration: migration tests (IT-16–IT-19) | Phase 2 | Migration implemented |
| Property: graph algorithms (TC-P005–TC-P009) | Phase 2 | Algorithms implemented |
| Property: file safety (TC-P010–TC-P011) | Phase 2 | Atomic writes implemented |
| LLM benchmark infrastructure | Phase 3 | Context bundles complete |
| LLM benchmark tasks (TC-030–TC-032) | Phase 3 | Full feature set complete |

---

**Rationale:**
- Three separate designs are necessary because each catches a disjoint failure class. Collapsing them into one approach (e.g., "just write more unit tests") would leave two failure classes untested. The cost of three approaches is justified by the risk distribution.
- Property-based tests are assigned to pure functions only. Attempting to property-test the full CLI (generating random repository structures and asserting on binary output) produces tests that are slow, brittle, and produce unhelpful failure messages. The integration harness handles that scope.
- The LLM benchmark uses binary rubric criteria specifically to reduce scorer variance. Holistic judgments ("is this good Rust?") have high variance between LLM calls. Binary questions ("does it use openraft?") produce consistent scores across runs.
- Running N=5 per condition at temperature=0 is a deliberate variance-reduction choice. Temperature=0 is not fully deterministic on all models, but variance at temperature=0 is small enough that mean-of-5 produces a stable score.
- The `delta_product_vs_naive ≥ 0.15` threshold is the most important pass criterion. A product score of 0.90 is meaningless if naive also scores 0.87 — Product would be providing no incremental value. The delta threshold enforces that Product earns its place in the workflow.

**Rejected alternatives:**
- **Only property tests** — cannot test CLI surface, error formatting, exit codes, or concurrent behaviour.
- **Only integration tests** — hand-crafted inputs miss parser edge cases and graph algorithm boundary conditions that proptest finds routinely.
- **Only the LLM benchmark** — high cost, slow feedback loop. Unsuitable as a development-time safety net. The property and integration tests must run on every commit.
- **Manual LLM evaluation** — subjective, unrepeatable, non-comparable across releases. The rubric-based approach is mechanical and produces a number that can be tracked over time.


---

## ADR-019: Continuous Gap Analysis — LLM-Driven Specification Review in CI

**Status:** Accepted

**Context:** The existing testing strategy (ADR-018) validates that Product works correctly and that context bundles improve LLM implementation quality. It does not validate that the *specifications themselves* are complete and internally consistent. A repository can have zero structural errors (`product graph check` exits 0), passing property tests, and passing integration tests — and still contain ADRs that make untested claims, reference undocumented constraints, or contradict each other. These specification gaps are invisible to structural tooling because they require semantic understanding of the content.

An LLM is precisely the right tool for semantic review of structured documents. The gap analysis capability runs an LLM against each ADR's full context bundle and asks it to identify specific, enumerated gap types. The result is a structured set of findings that can be tracked over time, baselined, suppressed, and resolved — giving gap analysis the same CI lifecycle as static analysis.

The key design constraint is CI reliability. LLM output is non-deterministic. A gap analysis that produces different results on two identical repository states would make CI unstable and unusable. This ADR specifies the three mechanisms that make gap analysis deterministic enough for CI: structured output schema, temperature=0, and run-twice intersection for high-severity findings.

**Decision:** Implement `product gap check` as a continuous LLM-driven specification review command. It analyses ADRs using depth-2 context bundles, checks for eight defined gap types (G001–G008), produces deterministic structured findings, and integrates with a `gaps.json` baseline for suppression and resolution tracking. The `--changed` flag scopes CI analysis to the affected ADR subgraph.

---

### Gap Type Specification

Seven gap types are checked. The set is fixed — the prompt is instructed to check only these types and ignore general quality issues. This constraint is critical for determinism: an open-ended "find any problems" prompt produces unbounded, incomparable findings across runs.

| Code | Severity | Trigger condition |
|---|---|---|
| G001 | high | ADR body contains a testable claim (performance threshold, behavioural invariant, correctness property) with no linked TC that exercises it |
| G002 | high | A `⟦Γ:Invariants⟧` formal block is present but no linked TC of type `scenario` or `chaos` references the ADR and addresses the invariant |
| G003 | medium | ADR has no **Rejected alternatives** section, or the section is empty |
| G004 | medium | ADR rationale references an external constraint, library behaviour, or assumption not captured in any linked artifact (feature, ADR, or TC) |
| G005 | high | This ADR makes a claim that is logically inconsistent with a claim in a linked ADR (shared feature or within depth-2 context) |
| G006 | medium | A feature linked to this ADR has aspects — stated in the feature body — not addressed by any of the feature's linked ADRs |
| G007 | low | ADR rationale references decisions, constraints, or rationale that have been superseded by a more recent ADR in the context bundle |

---

### Context Bundle for Gap Analysis

Gap analysis uses `product context ADR-XXX --depth 2`. This produces:

- The ADR under analysis
- All features linked to it
- All test criteria linked to those features
- All other ADRs linked to those features (the 1-hop ADR neighbourhood)
- Their test criteria

This is the same bundle an implementation agent would receive. Gap analysis validates specification completeness from the agent's perspective — if the agent cannot find the information it needs in this bundle, that is a gap.

---

### Prompt Specification

The gap analysis prompt is versioned and stored at `benchmarks/prompts/gap-analysis-v{N}.md`. The version is referenced in `product.toml` under `[gap-analysis]`. Changing the prompt increments the version — findings from different prompt versions are not comparable and must not be merged in `gaps.json`.

**Prompt version upgrade protocol:** When `prompt-version` is incremented in `product.toml`, `gaps.json` suppressions from the previous version are retained but tagged with the version they were created under. On the first run with the new prompt version, all existing suppressions are treated as provisional — they are not cleared, but `product gap check` emits W-class warnings for each one: "suppression GAP-ADR002-G001-a3f9 was created under prompt-version 1, re-verify with prompt-version 2." The developer reviews and either re-confirms (`product gap suppress --re-confirm`) or removes the suppression. This prevents stale suppressions from masking gaps that the new prompt detects differently.

The prompt structure:

```markdown
You are reviewing an architectural specification for completeness and consistency.
You will be given a context bundle containing an ADR and related artifacts.

Check ONLY for the following gap types. Do not report any other issues.

[Gap type table with codes, descriptions, and examples]

Respond ONLY with a JSON array of findings matching this schema exactly.
Do not include any prose before or after the JSON.
If you find no gaps, respond with an empty array: []

Schema:
{
  "id": "GAP-{ADR_ID}-{CODE}-{HASH}",   // deterministic ID per finding
  "code": "G001",
  "severity": "high",
  "description": "...",                  // one sentence, specific and actionable
  "affected_artifacts": ["ADR-002"],
  "suggested_action": "...",             // one sentence
  "evidence": "..."                      // quote from the context bundle that triggered this
}

Context bundle:
{BUNDLE}
```

The `evidence` field is required for G005 (contradiction) — it must quote the specific conflicting claim from each ADR. This forces the model to ground its finding in actual text rather than hallucinating a contradiction.

---

### Gap ID Derivation

Gap IDs are deterministic. The short hash is derived from: `sha256(adr_id + gap_code + sorted(affected_artifact_ids) + finding_description)[0:4]`. Same logical finding → same ID across runs. This is what makes suppression stable.

```rust
fn gap_id(adr_id: &str, code: &str, artifacts: &[&str], description: &str) -> String {
    let mut sorted = artifacts.to_vec();
    sorted.sort();
    let input = format!("{}{}{}{}", adr_id, code, sorted.join(","), description);
    let hash = &sha256(input.as_bytes())[..4];
    format!("GAP-{}-{}-{}", adr_id, code, hex(hash))
}
```

In practice, the description field introduces some variance (the model may phrase the same gap differently between runs). The run-twice intersection (for high severity) handles this — two runs that describe the same gap in slightly different words will produce different IDs and thus not intersect, suppressing the finding. This is conservative: it means some real gaps may not be reported. The alternative (fuzzy matching on descriptions) introduces its own instability. Conservative is correct for CI.

---

### `--changed` Scoping

In CI, gap analysis must not run against every ADR on every commit. The scoping algorithm:

1. `git diff --name-only HEAD~1` to identify changed files
2. Filter to files under the `adrs/` directory matching the ADR prefix
3. For each changed ADR, traverse the reverse graph to find all features it is linked to
4. For each of those features, traverse forward to find all other ADRs linked to that feature
5. The analysis set = changed ADRs ∪ their 1-hop ADR neighbours

This scoping ensures that a change to ADR-002 also analyses ADR-005 if they share a feature — because the change to ADR-002 may now contradict ADR-005 (G005). Without this expansion, G005 would never be caught by `--changed` mode.

The analysis set is bounded: `|changed_adrs| × |avg_adr_neighbours|`. For a well-structured repository with average ADR fan-out of 3, a PR changing 2 ADRs analyses at most ~8 ADRs. At ~10 seconds per ADR analysis, CI time is proportional and predictable.

---

### `gaps.json` Lifecycle

```
Initial state:   gaps.json does not exist → all findings are new
First run:       findings reported, developer suppresses known/expected ones
Subsequent runs: new findings (not in suppressions) → exit code 1
                 suppressed findings → exit code 0 (logged as informational)
                 resolved findings (were suppressed, now not detected) → logged, moved to resolved
```

`gaps.json` is committed to the repository. It is the shared team baseline. A suppression added by one developer is respected by all CI runs and all teammates.

`product gap suppress` mutates `gaps.json` atomically (ADR-015). The suppression records the gap ID, the reason, the suppressing commit, and the timestamp. This creates an audit trail of deliberate decisions to accept known gaps.

---

### Handling Model Errors

If the model call fails (network error, timeout, invalid JSON response), `product gap check` reports the error on stderr and exits 2 (analysis warning). It does not exit 1 (new gaps found). A transient model error never fails CI — it only produces a warning that the analysis was incomplete.

If the model returns JSON that does not match the finding schema, the malformed findings are discarded individually. Valid findings from the same response are retained. A log line on stderr records each discarded finding.

This asymmetry (model errors are warnings, not failures) is intentional. CI is not the right place to retry flaky API calls. The operator can re-run the analysis manually, or the next commit will trigger another run.

---

**Rationale:**
- Gap analysis belongs in CI, not as an ad-hoc command, because specification gaps compound over time. An unchecked gap in ADR-002 today becomes a missing test, then a misunderstood invariant, then a production bug six months later. Continuous analysis catches gaps when they are introduced, not when they manifest.
- The fixed set of seven gap types is essential for CI reliability. An open-ended prompt produces incomparable results across runs and across prompt versions. Enumerating the gap types converts an unbounded quality review into a bounded, checkable specification.
- Run-twice intersection for high-severity findings is a conservative but correct approach to the non-determinism problem. The alternative — accepting any single-run finding — produces false positives that pollute `gaps.json` with hallucinated contradictions. The cost is that some real gaps require two consistent runs to surface; the benefit is that CI never fails on a hallucination.
- `--changed` scoping with 1-hop expansion is the only CI-viable approach. Full-repository analysis on every commit is prohibitively expensive. Analysing only changed ADRs without expansion misses cross-ADR contradictions introduced by the change. The 1-hop expansion is the minimum scope that catches G005.
- `gaps.json` suppression follows the `cargo audit` model because it is already well-understood by the Rust community Product targets. Operators know how to work with it: audit, suppress known issues, fail on new ones.

**Rejected alternatives:**
- **PR comments only, no CI gate** — analysis results are informational only, no build failure. Rejected because informational findings are routinely ignored. A CI gate is the only mechanism that ensures gaps are addressed.
- **Run on every ADR every commit** — complete coverage, no scoping complexity. Rejected on cost and latency grounds. A repository with 30 ADRs at 10s per analysis adds 5 minutes to every CI run. At $0.01/1K tokens per ADR analysis, it adds non-trivial cost per commit.
- **Semantic similarity for gap ID matching** — use embedding similarity to match a suppressed gap to a re-detected gap even if the description changed. Rejected because embedding similarity requires a model call, adds complexity, and the threshold is tunable in a way that creates fragility. Exact hash matching is brittle but predictable.
- **Store findings in artifact front-matter** — gap findings annotate the ADR file directly rather than a separate `gaps.json`. Rejected because it creates noise in git history (every CI run would potentially produce a commit), contaminates the ADR content with tooling metadata, and makes it impossible to suppress a finding without modifying the artifact under analysis.

**Test coverage:**

Scenario tests:
- `gap_check_single_adr.rs` — run `product gap check ADR-001` against a fixture where ADR-001 has a testable claim with no linked TC. Assert exit code 1 and a G001 finding in stdout JSON.
- `gap_check_no_gaps.rs` — run `product gap check ADR-001` against a fixture with full TC coverage. Assert exit code 0 and an empty findings array.
- `gap_check_suppressed.rs` — add a suppression for a known gap to `gaps.json`. Run analysis. Assert exit code 0. Assert the finding appears in output with `"suppressed": true`.
- `gap_check_resolved.rs` — suppress a gap, then fix it (add the missing TC). Run analysis. Assert the gap no longer appears in findings. Assert `gaps.json` resolved list is updated.
- `gap_check_changed_scoping.rs` — modify ADR-002 in git. Run `product gap check --changed`. Assert only ADR-002 and its 1-hop neighbours are analysed (not ADR-007 which shares no features).
- `gap_check_model_error_exits_2.rs` — inject a network failure for the model call. Assert exit code 2 (warning), not 1 (new gaps). Assert error appears on stderr.
- `gap_check_invalid_json_discarded.rs` — inject a model response with one valid finding and one malformed finding. Assert the valid finding is in output. Assert the malformed finding is logged to stderr and discarded.
- `gap_id_deterministic.rs` — run gap analysis twice against identical repository state. Assert all high-severity findings have identical IDs between runs.
- `gap_suppress_mutates_baseline.rs` — run `product gap suppress GAP-ADR002-G001-a3f9 --reason "deferred"`. Assert `gaps.json` contains the suppression with the reason, timestamp, and current commit hash.
- `gap_changed_expansion.rs` — fixture: ADR-002 and ADR-005 share feature FT-001. Modify ADR-002. Run `--changed`. Assert ADR-005 is included in the analysis set.

Invariants:
- `gap_id_format.rs` — all gap IDs must match `GAP-[A-Z]+-[A-Z0-9]+-[A-Z0-9]{4,8}` pattern.
- `gap_stdout_stderr_separation.rs` — gap findings are always on stdout. Analysis errors are always on stderr. Verified by piping stdout only and asserting it is valid JSON.
- `gap_json_schema.rs` — every finding in output must have all required fields: id, code, severity, description, affected_artifacts, suggested_action. Missing fields are a test failure.


---

## ADR-020: MCP Server — Dual Transport (stdio and HTTP)

**Status:** Accepted

**Context:** Product must be usable from two distinct environments with fundamentally different connectivity models:

1. **Local desktop** — Claude Code runs as a subprocess in the same OS session as the developer. The natural MCP transport here is stdio: Claude Code spawns `product mcp` as a child process and communicates over stdin/stdout. No network, no authentication, no configuration beyond `.mcp.json`.

2. **Remote client (phone, browser, remote agent)** — claude.ai on a phone cannot spawn subprocesses. It connects to MCP servers over HTTP via the MCP Streamable HTTP transport. Product must bind to a network port, accept HTTP requests, and authenticate them.

Both use cases share the same tool surface. The transport is not a product boundary — it is a wire protocol. Implementing two separate binaries, or two separate tool registrations, would create maintenance burden and inevitable divergence. A single `product mcp` command with a transport flag is the correct design.

**Decision:** `product mcp` defaults to stdio transport. `product mcp --http` switches to HTTP Streamable transport. The tool registry, graph loading, and all tool handlers are shared between transports. Authentication is a transport-layer concern: stdio has none (trust the parent process), HTTP requires a bearer token.

---

### stdio Transport

```bash
product mcp           # stdio, reads repo from cwd
product mcp --repo /path/to/repo   # explicit repo path
```

Wire protocol: newline-delimited JSON over stdin/stdout per the MCP spec. Claude Code spawns this as a subprocess. The `.mcp.json` at repo root is the configuration contract.

```json
{
  "mcpServers": {
    "product": {
      "command": "product",
      "args": ["mcp"],
      "cwd": "${workspaceFolder}"
    }
  }
}
```

`${workspaceFolder}` is resolved by Claude Code to the open repository root. Product reads `product.toml` from this directory.

---

### HTTP Transport (Streamable HTTP)

```bash
product mcp --http
product mcp --http --port 8080
product mcp --http --bind 127.0.0.1    # localhost only
product mcp --http --bind 0.0.0.0      # all interfaces (remote access)
product mcp --http --token $SECRET
```

**Protocol:** MCP Streamable HTTP. Client sends HTTP POST to `/mcp`. Server responds either inline (for non-streaming tools) or as a server-sent event stream (for long-running tools like `product_gap_check`). A single endpoint handles both.

**Authentication:** Bearer token in the `Authorization` header. If `--token` is set (or `PRODUCT_MCP_TOKEN` env var), all requests without a valid token receive `401 Unauthorized`. If no token is configured, the server starts but logs a warning — unauthenticated HTTP is acceptable for localhost-only (`--bind 127.0.0.1`) but not for remote access.

**TLS:** Not handled by Product. The operator terminates TLS upstream. Recommended setups:
- **Local network:** HTTP is acceptable — traffic stays on the LAN
- **Remote access:** Cloudflare Tunnel, ngrok, or a reverse proxy (Caddy, nginx) provides TLS termination. Product binds HTTP; the tunnel provides HTTPS to the client.

**CORS:** Configurable in `product.toml`. For claude.ai access: `cors-origins = ["https://claude.ai"]`.

**Phone setup (complete):**
```bash
# On desktop/server:
export PRODUCT_MCP_TOKEN=$(openssl rand -hex 32)
product mcp --http --bind 0.0.0.0 --port 7777

# Or with Cloudflare Tunnel for HTTPS:
cloudflared tunnel --url http://localhost:7777

# In claude.ai → Settings → Connectors → Add MCP Server:
# URL:    https://your-tunnel.cfargotunnel.com/mcp
# Header: Authorization: Bearer $PRODUCT_MCP_TOKEN
```

---

### Tool Registry

Tools are registered once. The transport layer calls them identically:

```rust
pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn Tool>>,
    write_enabled: bool,
}

impl ToolRegistry {
    pub async fn call(&self, name: &str, args: Value) -> ToolResult {
        let tool = self.tools.get(name)
            .ok_or_else(|| ToolError::NotFound(name.to_string()))?;
        if tool.requires_write() && !self.write_enabled {
            return Err(ToolError::WriteDisabled);
        }
        tool.call(args).await
    }
}
```

The stdio handler and the HTTP handler both call `ToolRegistry::call`. There is no code path that is transport-specific in tool implementation.

---

### Write Safety in HTTP Mode

HTTP transport is stateless — multiple clients could theoretically send concurrent write requests. The same advisory lock (ADR-015) that serialises concurrent CLI invocations also serialises concurrent MCP write calls. A write tool call that cannot acquire the lock within 3 seconds returns a tool error (not an HTTP error) with the lock-holder's PID.

---

### Graceful Shutdown

HTTP mode responds to SIGTERM and SIGINT. On signal:
1. Stop accepting new connections
2. Complete in-flight requests (up to 10 second drain timeout)
3. Release file lock if held
4. Exit 0

This ensures that a `product mcp --http` process running as a systemd service restarts cleanly.

---

**Rationale:**
- Single binary, dual transport is the correct design. Two binaries would diverge on tool surface, error handling, and graph loading. The transport is genuinely a thin layer — the tool logic has no transport awareness.
- MCP Streamable HTTP is the current MCP specification for remote servers. SSE-based (the older spec) is also supported by claude.ai but is being superseded. Implementing Streamable HTTP positions Product correctly for the current and future spec.
- Bearer token auth is sufficient for this use case. OAuth would be more appropriate for a multi-user SaaS tool. Product is a personal developer tool — a static bearer token stored in a password manager or environment variable is the right complexity level.
- TLS delegation to a reverse proxy is standard practice for application servers written in Rust. Implementing TLS in Product would add a dependency (rustls or openssl), a certificate management problem, and certificate renewal complexity. Cloudflare Tunnel eliminates all of this and provides a publicly accessible HTTPS endpoint in one command.
- CORS is required for claude.ai access from a browser — the browser enforces CORS policy before any MCP request reaches the server. Configuring `cors-origins = ["https://claude.ai"]` in `product.toml` is the minimal configuration for phone access.

**Rejected alternatives:**
- **Two separate binaries: `product-mcp-stdio` and `product-mcp-http`** — maintenance burden, inevitable divergence. Rejected.
- **WebSocket transport** — supported by some MCP clients but not the primary transport for claude.ai. Streamable HTTP has broader client support and simpler server implementation.
- **gRPC** — excellent for high-throughput service-to-service communication. Overkill for a developer tool handling tens of requests per session.
- **Product-as-daemon with IPC** — one `product` daemon, CLI and MCP both talk to it via a Unix socket. Eliminates the cold-start cost of graph loading per invocation. Rejected for v1: the daemon lifecycle (start, stop, version skew between daemon and CLI) adds operational complexity that is not justified at the current scale.

**Test coverage:**

Scenario tests:
- `mcp_stdio_tool_call.rs` — spawn `product mcp` as a subprocess. Send a valid JSON-RPC tool call over stdin. Assert the response on stdout matches the expected MCP tool result format.
- `mcp_http_tool_call.rs` — start `product mcp --http --port 17777 --token test`. Send an HTTP POST to `http://localhost:17777/mcp`. Assert 200 response with correct tool result.
- `mcp_http_no_token_401.rs` — start server with `--token test`. Send request without Authorization header. Assert 401.
- `mcp_http_wrong_token_401.rs` — send request with wrong bearer token. Assert 401.
- `mcp_http_write_disabled.rs` — start server with `mcp.write = false`. Call a write tool. Assert tool error (not HTTP error), message "write tools disabled".
- `mcp_http_concurrent_writes.rs` — send two concurrent write tool calls. Assert one succeeds, one returns the lock-held error with PID.
- `mcp_http_graceful_shutdown.rs` — start server, send SIGTERM during an in-flight tool call. Assert the in-flight call completes before the process exits.
- `mcp_tool_registry_shared.rs` — assert that calling `product_context` via stdio and via HTTP on the same repository produces identical output.
- `mcp_cors_header.rs` — configure `cors-origins = ["https://claude.ai"]`. Assert CORS response headers are correct for a preflight request from that origin.


---

## ADR-021: Implementation Pipeline — `product verify` and the Knowledge Boundary

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

**Test coverage:**

Scenario tests:
- `verify_all_pass_completes_feature.rs` — all TCs configured with passing runners. Run `product verify FT-001`. Assert all TCs `passing`, feature `complete`.
- `verify_one_fail_in_progress.rs` — one TC fails. Assert feature stays `in-progress`.
- `verify_unrunnable_no_runner.rs` — all TCs have no `runner`. Assert feature status unchanged, W-class warning.
- `verify_updates_tc_frontmatter.rs` — run verify. Assert `last-run`, `last-run-duration` written to TC files.
- `verify_failure_message_written.rs` — failing TC. Assert `failure-message` written with test output.
- `verify_regenerates_checklist.rs` — run verify. Assert `checklist.md` updated.
- `verify_platform_runs_cross_cutting.rs` — run `product verify --platform`. Assert TCs linked to cross-cutting ADRs run. Assert feature-specific TCs not run.
- `verify_requires_satisfied.rs` — TC with `requires: [binary-compiled]`. Prerequisite command exits 0. Assert TC runs normally.
- `verify_requires_not_satisfied.rs` — TC with `requires: [two-node-cluster]`. Prerequisite command exits 1. Assert TC status becomes `unrunnable`, `failure-message` contains prerequisite name. Assert feature status unchanged.
- `verify_requires_missing_prereq_def.rs` — TC requires a prerequisite not defined in `product.toml`. Assert E-class error with the prerequisite name and a hint to add it to `[verify.prerequisites]`.
- `verify_wrapper_script.rs` — TC configured with `runner: bash`, `runner-args: ["scripts/test-harness/raft.sh"]`. Script exits 0. Assert TC status `passing`. Script exits 1. Assert TC status `failing`. Product has no knowledge of what the script does internally.
- `harness_scripts_present.rs` — assert `scripts/harness/implement.sh` and `scripts/harness/author.sh` exist and are executable.




---

## ADR-022: Authoring Resources — System Prompts and Pre-Commit Review

**Status:** Accepted

**Context:** Earlier versions of this ADR described `product author [feature|adr|review]` as CLI commands that started agent sessions. This was identified as a violation of the knowledge boundary established in ADR-021. Product does not invoke agents — it provides the knowledge and resources agents need.

For authoring sessions specifically, three things are needed: a versioned system prompt that tells the agent how to author graph-aware specifications, access to Product's MCP tool surface so the agent can read the graph as it writes, and a fast feedback loop for catching structural issues in draft artifacts before they are committed.

Product owns the system prompts as versioned files in the repository and the pre-commit review command. It does not start agent sessions.

**Decision:** System prompts for authoring sessions are versioned files stored in `benchmarks/prompts/`. A developer or harness loads the appropriate prompt and connects their agent to Product MCP — via stdio (Claude Code) or HTTP (remote clients including claude.ai on mobile). `product adr review --staged` provides fast structural feedback on draft ADRs at pre-commit time. `product install-hooks` installs the pre-commit hook. Both commands are Product's; agent invocation is the harness's.

---

### System Prompt Files

Stored at paths configured in `product.toml` under `[author]`:

```
benchmarks/prompts/
  author-feature-v1.md     # graph-aware feature authoring
  author-adr-v1.md         # graph-aware ADR authoring
  author-review-v1.md      # spec gardening / coverage improvement
  implement-v1.md          # implementation context template
```

Each file is self-contained — it can be pasted into any LLM interface, loaded as a Claude Code system prompt, configured as a claude.ai Project instruction, or fed to any other agent. Product does not parse or interpret these files; it only exposes their paths via the MCP tool `product_prompts_list` and `product_prompts_get`.

**`author-feature-v1.md` preamble (excerpt):**
```markdown
You are authoring a new feature specification for a repository managed by Product.
You have access to Product MCP tools. Before writing any content:

1. Call product_feature_list — understand what features exist
2. Call product_graph_central — identify the top-5 foundational ADRs  
3. Call product_context on the most related existing feature
4. Ask clarifying questions based on what you found

Only scaffold files after completing these steps.
When done: call product_graph_check and product_gap_check on new artifacts.
```

**`author-adr-v1.md` preamble (excerpt):**
```markdown
Before writing any content:
1. Call product_graph_central — read the top-5 ADRs by centrality first
2. Call product_adr_list — see what decisions already exist
3. Call product_impact on the area you're deciding — understand blast radius

Every ADR must have: Context, Decision, Rationale, Rejected alternatives,
Test coverage. Do not end without all five sections present and a linked TC.
```

**`author-review-v1.md` preamble (excerpt):**
```markdown
Your goal is to improve specification coverage without adding new features.
1. Call product_graph_check — fix structural issues first
2. Call product_metrics_stats — identify weak metrics
3. Walk features by lowest phi score — propose formal blocks
4. Find features with W003 warnings — propose exit-criteria TCs
```

---

### How Agents Access Prompts

**Claude Code (stdio MCP, local):**
```bash
# .mcp.json is already in the repo — Claude Code connects automatically
# Developer opens Claude Code and pastes the prompt or uses a custom slash command:
/author-feature   # configured to send author-feature-v1.md as context
```

**claude.ai Project (HTTP MCP, phone or browser):**

The `author-feature-v1.md` content is pasted into the Project's instruction field once. Every conversation in that Project is automatically a graph-aware authoring session. No CLI command needed — the phone is always in authoring mode when that Project is open.

```
Project: PiCloud Development
Instructions: [contents of author-feature-v1.md]
Connected MCP servers: http://your-desktop:7778/mcp
```

**`product prompts init`** — scaffolds `benchmarks/prompts/` with the default prompt files if they don't exist:

```
product prompts init                  # create default prompt files
product prompts list                  # show available prompts and versions
product prompts get author-feature    # print prompt to stdout (for piping)
product prompts update author-feature # bump to latest version
```

These are file management commands, not agent invocation.

---

### Pre-Commit Hook

`product install-hooks` writes `.git/hooks/pre-commit`:

```bash
#!/bin/sh
# Installed by: product install-hooks
# Product is a knowledge tool. This hook runs knowledge checks, not agents.
STAGED_ADRS=$(git diff --cached --name-only | grep "^docs/adrs/")
if [ -n "$STAGED_ADRS" ]; then
    echo "Running product adr review on staged ADRs..."
    product adr review --staged
    # Advisory only — exit 0 regardless of findings
fi
exit 0
```

`product adr review --staged` performs:

**Structural checks (local, instant, no LLM):**
- All five required sections present (Context, Decision, Rationale, Rejected alternatives, Test coverage)
- `status` field set and valid
- At least one entry in `features` front-matter
- At least one TC linked
- Evidence blocks present on any `⟦Γ:Invariants⟧` blocks

**LLM review (single call, ~3 seconds):**
- Internal consistency: does rationale support the decision?
- Contradiction scan: compare against linked ADRs' decisions
- Missing test suggestion: given the claims, what TCs are obviously absent?

Output uses ADR-013 rustc-style diagnostics. Advisory — the commit proceeds regardless. Fast feedback before CI.

---

**Rationale:**
- System prompts as versioned files in the repository means they are version-controlled, reviewable in PRs, and shareable across any agent platform. They are not locked inside Product's binary. A team can fork them, iterate on them, and maintain their own prompt library alongside their specifications.
- `product prompts get author-feature` piping to stdin of any agent is the cleanest composition: `product prompts get author-feature | my-agent`. Product provides the prompt, the harness provides the agent.
- Pre-commit review is advisory and LLM-assisted because the goal is fast authoring-time feedback, not a CI gate. The structural checks (no LLM) complete in milliseconds. The LLM review adds 3 seconds. The developer sees both before the commit lands. The CI gap analysis gate is the hard enforcement point.
- `product prompts init` solves the bootstrap problem: a new repository has no prompt files. The command creates sensible defaults that the team can then evolve.

**Rejected alternatives:**
- **`product author feature` as a CLI command that starts Claude Code** — agent invocation is not Product's responsibility. Rejected (see ADR-021).
- **Prompts embedded in the binary** — not user-modifiable. Teams evolve their authoring approaches; baking prompts into the binary forces a Product upgrade to change a prompt. Rejected.
- **Pre-commit hook that starts an agent session** — a blocking agent session in a pre-commit hook makes commits slow and non-deterministic. The hook runs `product adr review --staged`, which is fast and deterministic. Rejected.

**Test coverage:**

Scenario tests:
- `prompts_init_creates_files.rs` — run `product prompts init` on a repo with no `benchmarks/prompts/`. Assert all default prompt files are created.
- `prompts_list_output.rs` — run `product prompts list`. Assert output lists all prompt files with version numbers.
- `prompts_get_stdout.rs` — run `product prompts get author-feature`. Assert stdout contains the prompt content. Assert stderr is empty.
- `pre_commit_hook_installed.rs` — run `product install-hooks`. Assert `.git/hooks/pre-commit` exists and is executable.
- `pre_commit_hook_runs_on_staged_adr.rs` — stage ADR with missing Rejected alternatives. Run hook. Assert structural finding on stdout. Assert exit code 0.
- `pre_commit_hook_skips_non_adr.rs` — stage a feature file. Assert hook does not run `adr review`.
- `adr_review_missing_section.rs` — review ADR missing Rejected alternatives. Assert finding with file path and section name.
- `adr_review_no_features.rs` — review ADR with `features: []`. Assert W001-class finding.
- `mcp_prompts_list_tool.rs` — call `product_prompts_list` via MCP. Assert JSON response lists available prompts.
- `mcp_prompts_get_tool.rs` — call `product_prompts_get` with `name: "author-feature"`. Assert response contains prompt content.




---

## ADR-023: Drift Detection — Spec vs. Implementation Verification

**Status:** Accepted

**Context:** Gap analysis (ADR-019) checks specification completeness. It validates that ADRs are internally consistent and well-covered by test criteria. It does not check whether the codebase matches the ADRs. An ADR can be complete, well-tested, and fully gap-free — and the implementation can still contradict it. This divergence is invisible to all current Product checks because they operate on the documentation graph, not the code.

Drift detection closes this gap by giving an LLM both the ADR context bundle and the relevant source files and asking it to identify where the code diverges from the decisions.

**Decision:** `product drift check` provides LLM-driven spec-vs-implementation verification. The LLM receives the ADR's depth-2 context bundle and the source files associated with it (resolved via configurable path patterns). It checks for four drift types (D001–D004). Findings follow the same baseline/suppression model as gap findings (`drift.json`). `product drift scan` reverses the direction: given a source path, identify which ADRs govern it.

---

### Source File Association

Product resolves source files for an ADR via two mechanisms:

**Pattern-based (configured):**
```toml
# product.toml
[drift]
source-roots = ["src/", "lib/"]
ignore = ["tests/", "benches/", "target/"]
max-files-per-adr = 20        # cap to keep context bundle size bounded
```

For each ADR, Product searches `source-roots` for files whose path or content contains the ADR's ID or any of its linked feature IDs. This is a heuristic — it will miss files with no explicit reference. The `--files` flag overrides for precision:

```bash
product drift check ADR-002 --files src/consensus/raft.rs src/consensus/leader.rs
```

**`source-files` in ADR front-matter (explicit):**
```yaml
---
id: ADR-002
source-files:
  - src/consensus/raft.rs
  - src/consensus/leader.rs
---
```

Explicit `source-files` in front-matter always override pattern-based discovery. This is the recommended approach for ADRs governing specific, known files.

---

### Drift Types

| Code | Severity | Description |
|---|---|---|
| D001 | high | Decision not implemented — ADR mandates X, no code implements X |
| D002 | high | Decision overridden — code does Y where ADR says do X |
| D003 | medium | Partial implementation — some aspects implemented, some not |
| D004 | low | Undocumented implementation — code does X with no ADR governing why |

D004 is the "code ahead of spec" case. It is a low-severity finding that suggests an ADR should be written, not that something is wrong.

---

### `product drift scan`

Reverse direction: given a source path, find the ADRs that govern it.

```
product drift scan src/consensus/raft.rs
  → ADR-002: openraft for cluster consensus
  → ADR-006: Oxigraph for RDF projection (via raft log → projection pipeline)
  → ADR-001: Rust as implementation language
```

The scan loads the file, asks the LLM to identify which ADRs from the full graph are relevant to this code, and returns them ranked by relevance. This is "ADR archaeology" — understanding the decisions behind an unfamiliar file without reading the entire spec.

---

### `drift.json` Baseline

Same structure as `gaps.json`. Suppressions reference `DRIFT-{ADR_ID}-{CODE}-{HASH}`. The same suppression lifecycle applies: new findings fail CI, suppressed findings pass, resolved findings are recorded.

```json
{
  "schema-version": "1",
  "suppressions": [
    {
      "id": "DRIFT-ADR002-D003-f4a1",
      "reason": "Partial implementation is intentional — full openraft storage layer in phase 2",
      "suppressed_by": "git:abc123",
      "suppressed_at": "2026-04-11T09:00:00Z"
    }
  ]
}
```

---

**Rationale:**
- Drift detection requires source code access, which makes it qualitatively different from gap analysis. Gap analysis operates entirely within the docs graph. Drift analysis crosses the docs/code boundary. This distinction justifies a separate command, separate finding codes, and a separate baseline file.
- `source-files` in ADR front-matter is the high-precision path. For ADRs governing specific subsystems (consensus, storage, IAM), the author knows exactly which files implement the decision. For cross-cutting ADRs (ADR-001 Rust), pattern-based discovery is appropriate.
- D004 (undocumented implementation) is valuable during active development phases when code is written faster than specs. It prompts the developer to write the ADR that should govern the code they just wrote. It is low severity — not a failure, a reminder.

**Test coverage:**

Scenario tests:
- `drift_check_d002_detected.rs` — fixture with ADR saying "use openraft", source file using a custom Raft struct. Assert D002 finding.
- `drift_check_d001_detected.rs` — ADR mandates a specific interface, source file has no such interface. Assert D001 finding.
- `drift_scan_returns_adrs.rs` — call `product drift scan src/consensus/raft.rs` on a fixture where ADR-002 has `source-files: [src/consensus/raft.rs]`. Assert ADR-002 is in the result.
- `drift_suppressed_passes.rs` — suppress a D002 finding. Run drift check. Assert exit 0.
- `drift_source_files_frontmatter.rs` — ADR with `source-files` in front-matter. Assert those files are used for analysis regardless of pattern config.

---

## ADR-024: Architectural Fitness Functions — Continuous Metric Tracking

**Status:** Accepted

**Context:** `product graph check` and `product gap check` provide point-in-time binary assessments: the graph is valid or it isn't, there are gaps or there aren't. They do not show trends. A repository where `phi` (formal block coverage) has been declining for six weeks is not distinguishable from one where it has been stable at 0.70 — both pass today's CI check. The decline is invisible until `phi` drops below the configured threshold.

Architectural fitness functions (from "Building Evolutionary Architectures") address this: define metrics that measure architectural properties, record them over time, and gate on both current values and trends.

**Decision:** `product metrics record` appends a JSON snapshot to `metrics.jsonl` on every merge to main. `product metrics threshold` checks current values against configured thresholds in CI. `product metrics trend` renders the time series. `metrics.jsonl` is committed to the repository — the history is version-controlled alongside the code it describes.

---

### Tracked Metrics

| Metric | Computation | Good direction |
|---|---|---|
| `spec_coverage` | features with ≥1 linked ADR / total features | ↑ |
| `test_coverage` | features with ≥1 linked TC / total features | ↑ |
| `exit_criteria_coverage` | features with exit-criteria TC / total features | ↑ |
| `phi` | mean formal block coverage across all invariant+chaos TCs | ↑ |
| `gap_density` | new gaps opened in last 7d / total ADRs | ↓ |
| `gap_resolution_rate` | gaps resolved / gaps opened, rolling 30d | ↑ |
| `drift_density` | unresolved drift findings / total ADRs | ↓ |
| `centrality_stability` | variance in top-5 ADR centrality ranks, week-over-week | ↓ |
| `implementation_velocity` | features moved to `complete` in last 7d | tracked |
| `bundle_depth1_adr_p95` | 95th percentile of `depth-1-adrs` across all features | ↓ |
| `bundle_tokens_p95` | 95th percentile of `tokens-approx` across all features | ↓ |
| `bundle_domains_p95` | 95th percentile of `domains` count across all features | ↓ |
| `features_over_adr_threshold` | count of features where `depth-1-adrs` exceeds threshold | ↓ |

All metrics except `implementation_velocity`, `centrality_stability`, and the `features_over_*` count metrics are in [0.0, 1.0] or are raw counts/values. Bundle size metrics use percentile aggregation — per-feature bundle sizes are recorded in `metrics.jsonl` on each `product context --measure` call; the p95 values are recomputed by `product metrics record` from all available feature measurements.

---

### `metrics.jsonl`

Two entry types are appended to `metrics.jsonl`:

**Repository-wide snapshot** (written by `product metrics record`):
```json
{
  "type": "snapshot",
  "date": "2026-04-11T09:00:00Z",
  "commit": "abc123",
  "spec_coverage": 0.87,
  "test_coverage": 0.72,
  "exit_criteria_coverage": 0.61,
  "phi": 0.68,
  "gap_density": 0.03,
  "gap_resolution_rate": 0.75,
  "drift_density": 0.10,
  "centrality_stability": 0.02,
  "implementation_velocity": 2,
  "bundle_depth1_adr_p95": 6.0,
  "bundle_tokens_p95": 7800,
  "bundle_domains_p95": 4.0,
  "features_over_adr_threshold": 2
}
```

**Per-feature bundle measurement** (written by `product context FT-XXX --measure`):
```json
{
  "type": "bundle_measure",
  "date": "2026-04-11T09:14:22Z",
  "feature": "FT-003",
  "depth-1-adrs": 9,
  "depth-2-adrs": 14,
  "tcs": 12,
  "domains": 5,
  "tokens-approx": 11200
}
```

`metrics.jsonl` is committed to the repo. Merge conflicts are resolved by keeping both lines.

---

### Threshold Configuration

```toml
[metrics.thresholds]
spec_coverage           = { min = 0.90, severity = "error" }
test_coverage           = { min = 0.80, severity = "error" }
exit_criteria_coverage  = { min = 0.60, severity = "warning" }
phi                     = { min = 0.70, severity = "warning" }
gap_resolution_rate     = { min = 0.50, severity = "warning" }
drift_density           = { max = 0.20, severity = "warning" }

# Bundle size thresholds — signals features that may need splitting
bundle_depth1_adr_max   = { max = 8,    severity = "warning" }  # per-feature
bundle_tokens_max       = { max = 12000, severity = "warning" } # per-feature
bundle_domains_max      = { max = 6,    severity = "warning" }  # per-feature
features_over_adr_threshold = { max = 3, severity = "warning" } # repository-wide
```

Bundle size thresholds apply per-feature. When `product metrics threshold` runs, it checks every feature's last-measured `bundle` block against the per-feature thresholds and reports features that breach them. The `features_over_adr_threshold` metric is the repository-wide count of breaching features — this is what goes into CI as a gate.

---

### `product metrics trend` Output

ASCII sparkline for quick terminal inspection:

```
product metrics trend --metric phi --last 30d

phi (formal block coverage) — last 30 days
0.80 ┤                                    ╭──
0.75 ┤                               ╭───╯
0.70 ┤ ──────────────────────────────╯      ← threshold: 0.70
0.65 ┤
     └────────────────────────────────────
     2026-03-12              2026-04-11

current: 0.78  Δ7d: +0.03  Δ30d: +0.12  trend: ↑
```

`product metrics trend` with no flags shows all metrics as a summary table with current value, 7-day delta, and trend arrow.

---

**Rationale:**
- Committing `metrics.jsonl` to the repository is the correct storage decision. It co-locates the metric history with the artifacts it measures, it is version-controlled, it requires no external service, and it is inspectable with standard git tooling. The alternative (a metrics database or external dashboard) adds operational dependencies that contradict Product's repository-native design principle.
- ASCII sparklines in terminal are sufficient for a developer tool. An external dashboard would provide more visual richness but would require a server, a URL, and a login. The terminal is always available, especially during the authoring sessions where metrics are most relevant.
- `implementation_velocity` is tracked but has no threshold. It is an informational metric — fast velocity is not always good (quality may be declining), slow velocity is not always bad (hard problems take time). It should be observed, not gated on.
- Appending to `metrics.jsonl` rather than updating a single record means the full history is always available without a database. Trend computation reads all records at query time — acceptable for a file that grows by one line per merge to main.

**Test coverage:**

Scenario tests:
- `metrics_record_appends.rs` — run `product metrics record` twice. Assert `metrics.jsonl` has two lines and both are valid JSON with all required fields.
- `metrics_threshold_error_exits_1.rs` — set `spec_coverage` threshold, configure a repo below it. Run `product metrics threshold`. Assert exit code 1.
- `metrics_threshold_warning_exits_2.rs` — breach a warning-severity threshold only. Assert exit code 2.
- `metrics_threshold_clean_exits_0.rs` — all thresholds met. Assert exit code 0.
- `metrics_trend_renders.rs` — `metrics.jsonl` with 10 records. Run `product metrics trend`. Assert stdout contains sparkline output (non-empty, no errors).
- `metrics_jsonl_merge_conflict_safe.rs` — create `metrics.jsonl` with two records on the same line (simulating a bad merge). Assert `product metrics trend` handles it gracefully with a W-class warning.


---

## ADR-025: Concern Domains — ADR Classification and Cross-Cutting Scope

**Status:** Accepted

**Context:** At scale (100+ ADRs), the graph has a discovery problem. ADRs are nodes with edges, but they carry no information about what kind of concern they govern. An ADR about security and an ADR about storage are structurally identical — the only way to find all security ADRs is to already know which ones they are. A new feature author in a large repository has no systematic way to ask "have I considered all security implications?" because "security" is not a first-class concept in the graph.

Two categories of ADR emerge at scale that are currently invisible:

**Cross-cutting ADRs** apply to every feature regardless of graph links. ADR-013 (error model) governs how every component surfaces errors. ADR-015 (file write safety) governs every mutation. These ADRs are never "done being relevant" — they apply to every new feature, always. Currently they only appear in a feature's context bundle if the author remembers to link them.

**Domain ADRs** govern a concern area (security, storage, IAM) but apply only to features that touch that area. A feature that introduces a new storage mechanism should consider all storage ADRs. Currently there is no way to identify that set without reading every ADR manually.

**Decision:** Add a `domains` field and a `scope` field to ADR front-matter. Domains are a controlled vocabulary declared in `product.toml`. `scope: cross-cutting` marks ADRs that must be acknowledged by every feature. `scope: domain` marks ADRs that must be acknowledged by any feature touching a declared domain. Feature front-matter gains a `domains-acknowledged` block for explicit reasoning when a domain applies but no ADR link is added.

---

### Domain Vocabulary

Domains are declared in `product.toml`. Each domain has a name and a one-sentence description:

```toml
[domains]
security        = "Authentication, authorisation, secrets, trust boundaries"
storage         = "Persistence, durability, volume, block devices, backup"
consensus       = "Raft, leader election, log replication, cluster membership"
networking      = "mDNS, mTLS, DNS, service discovery, port allocation"
error-handling  = "Error model, diagnostics, exit codes, panics, recovery"
observability   = "OTel, metrics, tracing, logging, telemetry"
iam             = "Identity, OIDC, tokens, RBAC, workload identity"
scheduling      = "Workload placement, resource limits, eviction, CPU/memory"
api             = "CLI surface, MCP tools, event schema, resource language"
data-model      = "RDF, SPARQL, ontology, event sourcing, projections"
```

The vocabulary is project-specific and evolves as the project grows. Domains are not a universal taxonomy — they reflect the concern areas that matter for this specific system.

---

### ADR Front-Matter Extension

```yaml
---
id: ADR-013
title: Error Model and User-Facing Error Format
status: accepted
features: [FT-001, FT-002]
domains: [error-handling, developer-experience]
scope: cross-cutting    # cross-cutting | domain | feature-specific
---
```

**Scope values:**

| Value | Meaning | Pre-flight behaviour |
|---|---|---|
| `cross-cutting` | Applies to every feature without exception | Must be linked or acknowledged by every new feature |
| `domain` | Applies to any feature touching the declared domains | Must be linked or acknowledged if the feature declares any matching domain |
| `feature-specific` | Governs a narrow, specific area | No automatic pre-flight requirement |

`feature-specific` is the default when `scope` is absent — preserving backward compatibility with all existing ADRs.

---

### Feature Front-Matter Extension

```yaml
---
id: FT-009
title: Rate Limiting
phase: 2
status: planned
depends-on: [FT-004]
domains: [networking, api]          # domains this feature touches
adrs: [ADR-004, ADR-009, ADR-012]
tests: [TC-041, TC-042]
domains-acknowledged:
  security: >
    Rate limiting operates at the Resource API layer. IAM enforces
    access upstream. No new trust boundaries introduced.
  iam: >
    No new identity primitives. Rate limit state is per-resource,
    not per-identity. Existing RBAC roles are unchanged.
  storage: >
    Token bucket state is in-memory only. No persistence required.
    Intentional — limits reset on restart.
---
```

`domains-acknowledged` entries close domain gaps without requiring a linked ADR. The reasoning is mandatory — an acknowledgement without a reason is a validation error (E011). The reasoning is included in the feature's context bundle so the implementation agent understands the deliberate scope exclusions.

---

### Validation Rules

`product graph check` gains two new checks:

**E011 — Acknowledgement without reasoning:** a `domains-acknowledged` entry exists but the value is empty or whitespace-only.

**W010 — Unacknowledged cross-cutting ADR:** a cross-cutting ADR exists and is neither linked to nor acknowledged by a feature. Reported as a warning per-feature: "FT-009 has not acknowledged ADR-013 (cross-cutting, error-handling)."

**W011 — Domain gap without acknowledgement:** a feature declares a domain (via `domains`) that has domain-scoped ADRs, but the feature neither links those ADRs nor acknowledges the domain.

W010 and W011 are warnings, not errors. During active development phases, a feature author may not have completed domain review. The warnings surface the gaps without blocking CI.

---

### Cross-Cutting ADR Resolution in Context Bundles

When assembling a context bundle for a feature, cross-cutting ADRs are always included regardless of explicit graph links. They are included at a fixed position: after the feature content, before the domain ADRs, before the feature-specific ADRs.

Bundle order:
1. Feature content
2. Cross-cutting ADRs (all, ordered by betweenness centrality)
3. Domain ADRs for the feature's declared domains (top-2 by centrality per domain)
4. Feature-linked ADRs (direct links, by centrality)
5. Test criteria

This ensures the implementation agent sees the governance layer (cross-cutting) before the architectural context (domain and feature-specific).

---

**Rationale:**
- The domain taxonomy is the index that makes large graphs navigable. Without it, finding "all security ADRs" requires reading every ADR. With it, `signal graph check --domain security` returns them instantly.
- `scope: cross-cutting` is the mechanism for ADRs that must never be forgotten. Instead of relying on every feature author to remember to link ADR-013, the system enforces it automatically. The author is free to say "I've considered this and it doesn't apply" — but they cannot silently skip it.
- Mandatory reasoning in `domains-acknowledged` is the critical design. An acknowledgement without reasoning is indistinguishable from a checkbox that was ticked to silence the warning. The reasoning proves intent. It also becomes valuable documentation — future authors reading the feature can see why security was explicitly scoped out.
- Limiting domain ADRs in bundles to top-2 by centrality (not all domain ADRs) is the key to avoiding context explosion. In a domain with 15 ADRs, the top-2 by centrality are the most foundational — the ones that govern the others. Reading them first is sufficient for the agent to understand the domain's constraints.

**Rejected alternatives:**
- **Tag-based classification** — tags with no vocabulary control. No scope distinction (cross-cutting vs domain). No acknowledgement mechanism. Rejected.
- **Mandatory ADR linking for all domains** — requires a linked ADR for every domain the feature touches, even if no existing ADR is relevant. Creates pressure to create unnecessary ADRs. Rejected.
- **All domain ADRs in context bundle** — a feature touching storage would receive all 15 storage ADRs. Context explosion at exactly the scale this ADR is designed to avoid. Rejected.
- **Centrality as the only filter** — use centrality ranking without domain taxonomy. Cannot answer "which ADRs are about security." Centrality tells you what's important, not what topic it's about. Both are needed. Rejected.

**Test coverage:**

Scenario tests:
- `cross_cutting_always_in_bundle.rs` — ADR-013 marked `scope: cross-cutting`. Feature FT-009 has no explicit link to ADR-013. Assert `product context FT-009` includes ADR-013 in the bundle.
- `cross_cutting_bundle_position.rs` — assert cross-cutting ADRs appear before domain ADRs in the bundle, which appear before feature-linked ADRs.
- `domain_top2_centrality.rs` — domain `security` has 6 ADRs with known centrality scores. Feature FT-009 declares `domains: [security]` with no acknowledged ADRs. Assert the context bundle includes exactly the 2 highest-centrality security ADRs.
- `acknowledgement_requires_reason.rs` — feature front-matter has `domains-acknowledged: { security: "" }`. Assert E011 with file path and field name.
- `w010_unacknowledged_cross_cutting.rs` — ADR-013 is cross-cutting. FT-009 neither links nor acknowledges it. Run `product graph check`. Assert W010 naming FT-009 and ADR-013.
- `w011_domain_gap.rs` — FT-009 declares `domains: [security]`. Security domain has ADRs. FT-009 neither links nor acknowledges security. Assert W011.
- `acknowledgement_closes_gap.rs` — FT-009 has `domains-acknowledged: { security: "no trust boundaries" }`. Assert W011 does not fire for FT-009's security domain.
- `domains_vocab_unknown.rs` — feature declares `domains: [unknown-domain]`. Assert E012 (unknown domain, not in `product.toml` vocabulary).

Invariants:
- Every `scope: cross-cutting` ADR must appear in every context bundle for every feature. Verified by a property test generating arbitrary feature graphs and asserting all cross-cutting ADRs are present in every bundle.


---

## ADR-026: Pre-flight Analysis — Systematic Coverage Before Authoring

**Status:** Accepted

**Context:** ADR-025 introduces domain classification and cross-cutting scope. This creates the data needed for systematic coverage checking. The question is when and how that check runs.

Two options: passive (surface gaps after the feature is authored, via `graph check` warnings) or active (surface gaps before authoring begins, via a dedicated pre-flight command that blocks until acknowledged).

Passive is insufficient. By the time a feature is authored, the author has invested effort in a spec that may need significant revision to address domain gaps. The cost of late discovery is high.

Active pre-flight is the right model. It runs before the authoring session starts, presents the specific gaps, and requires either linking or acknowledging each one before proceeding. The authoring agent cannot begin until the coverage check is clean.

**Decision:** `product preflight FT-XXX` is a mandatory first step in the `product author feature` session and the `product implement FT-XXX` pipeline. It analyses the feature against the domain taxonomy and cross-cutting ADR set, presents a structured coverage report, and requires each gap to be resolved (linked or acknowledged) before the session or pipeline continues. Preflight results are cached for the duration of the session — subsequent runs within the same session skip the LLM calls.

---

### Preflight Report Format

```
product preflight FT-009

Pre-flight analysis: FT-009 — Rate Limiting
Feature domains: networking, api

━━━ Cross-Cutting ADRs (must acknowledge all) ━━━━━━━━━━━━━━

  ✓  ADR-001  Rust as implementation language          [linked]
  ✓  ADR-013  Error model and diagnostics              [linked]
  ✓  ADR-015  File write safety                        [linked]
  ✗  ADR-038  Observability requirements               [not linked, not acknowledged]
  ✗  ADR-040  CLI output conventions                   [not linked, not acknowledged]

━━━ Domain Coverage ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  networking  ✓  ADR-004 (linked), ADR-006 (linked)
  api         ✓  ADR-009 (linked), ADR-012 (linked)
  security    ✗  no coverage — 4 ADRs in domain, none linked or acknowledged
  iam         ✗  no coverage — 3 ADRs in domain, none linked or acknowledged
  storage     ~  ADR-007 (linked) — review: does rate limiting touch storage?

━━━ Summary ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  2 cross-cutting ADRs unacknowledged
  2 domains with no coverage
  1 domain flagged for review

To resolve:
  product feature link FT-009 --adr ADR-038
  product feature acknowledge FT-009 --domain security --reason "..."
  product feature acknowledge FT-009 --domain iam --reason "..."
  product feature acknowledge FT-009 --adr ADR-040 --reason "..."
```

---

### Resolution Commands

```bash
# Link an ADR (adds to feature's adrs list)
product feature link FT-009 --adr ADR-038

# Acknowledge a domain gap with reasoning
product feature acknowledge FT-009 --domain security \
  --reason "rate limiting operates at resource API layer, no trust boundaries introduced"

# Acknowledge a cross-cutting ADR with reasoning
product feature acknowledge FT-009 --adr ADR-040 \
  --reason "rate limiting has no special output requirements beyond standard error model"

# Acknowledge all gaps at once with a shared reason (use carefully)
product feature acknowledge FT-009 --all-domains \
  --reason "reviewed all domains, see individual notes in ADR-021"
```

All acknowledgement commands mutate feature front-matter atomically (ADR-015) and re-run preflight validation to confirm the gap is closed.

---

### Preflight in the `product author` Session

When `product author feature` is invoked, the first action before any user message is processed:

1. Run `product preflight FT-XXX`
2. If preflight is clean — proceed to authoring
3. If preflight has gaps — present the report to Claude via the MCP tool surface
4. Claude reads the unacknowledged cross-cutting ADRs and domain ADRs
5. For each gap, Claude either:
   - Calls `product_feature_link` to add the ADR (if it's clearly relevant)
   - Calls `product_feature_acknowledge` with a specific reason (if it's clearly not applicable)
   - Asks the user to make the call (if intent is ambiguous)
6. Re-runs preflight — repeats until clean
7. Only then begins writing feature content

This makes the domain coverage check part of the authoring session's natural flow rather than a separate gate the user must remember to run.

---

### Preflight in `product implement`

`product implement FT-XXX` already has a gap gate (ADR-021, Step 1). Preflight is inserted as Step 0, before the gap gate:

```
product implement FT-009

  Step 0 — Pre-flight check
  ✗ 2 cross-cutting ADRs unacknowledged (ADR-038, ADR-040)
  ✗ 2 domains with no coverage (security, iam)

  Implementation blocked. Run: product preflight FT-009
  Then re-run: product implement FT-009
```

Preflight failures always block `product implement`. Unlike the gap gate (which can be suppressed), preflight coverage gaps cannot be bypassed — they must be resolved or acknowledged.

---

### Coverage Matrix: `product graph coverage`

The portfolio-level view of feature × domain coverage:

```
product graph coverage

                    sec  stor  cons  net  obs  err  iam  sched  api
FT-001 Cluster       ✓    ✓     ✓    ✓    ✓    ✓    ✓    ✓     ✓
FT-002 Products      ✓    ✓     ·    ✓    ✓    ✓    ✓    ·     ✓
FT-003 RDF Store     ~    ✓     ·    ·    ✓    ✓    ~    ·     ✓
FT-004 IAM           ✓    ·     ·    ✓    ✓    ✓    ✓    ·     ✓
FT-009 Rate Limit    ✗    ✗     ·    ✓    ✗    ✗    ✗    ·     ✓

Legend:  ✓ covered (linked)   ~ acknowledged   · not applicable   ✗ gap
```

`·` (not applicable) appears when: the feature does not declare the domain in its `domains` field AND no cross-cutting ADRs exist for that domain. The domain genuinely doesn't apply.

`~` (acknowledged) appears when the domain is acknowledged with a reason but not linked to an ADR.

`product graph coverage --domain security` filters to show only the security column with full ADR details for each feature.

`product graph coverage --format json` produces machine-readable output for CI reporting.

---

### Preflight Caching

Preflight analysis involves graph traversal but no LLM calls. It runs in < 100ms on any repository up to 200 features. No caching is needed — each `product preflight` invocation reads the graph fresh (consistent with ADR-003: no persistent graph store).

---

**Rationale:**
- Pre-flight as Step 0 in `product implement` is the right enforcement point. The implementation agent should never receive a context bundle that has known domain gaps — it would implement a feature without knowing it's missing security consideration. The gate prevents this.
- Non-bypassability distinguishes pre-flight from gap analysis. Gap findings can be suppressed with a reason. Domain coverage gaps can be acknowledged with a reason. But acknowledgement is not suppression — it is an explicit statement of intent. The distinction matters: suppressing a gap silences the finding; acknowledging a domain gap documents a conscious decision.
- The coverage matrix is the most valuable output for engineering leadership and code review. It makes architectural blind spots visible at a glance. A feature with six domain gaps in the `✗` column is not ready for implementation regardless of whether its test criteria are written.
- Limiting domain ADR inclusion in preflight to top-2 by centrality is inherited from ADR-025. The same reasoning applies: centrality filters to the foundational decisions without requiring the author to read all 15 storage ADRs when adding a feature that touches storage.

**Rejected alternatives:**
- **Passive gap surfacing only (W010/W011 in `graph check`)** — discovered after authoring. Cost of late discovery is high. Rejected.
- **Mandatory ADR links for all domains** — cannot acknowledge a domain as not-applicable. Every feature touching `networking` would need to link all networking ADRs or get a hard error. Too rigid — not every feature has security implications, and forcing links creates meaningless associations. Rejected.
- **LLM-assisted domain assignment** — let the LLM suggest which domains a feature touches based on its content. Reduces manual tagging burden. Rejected for v1: LLM-suggested domains introduce non-determinism into what should be an explicit author decision. Can be added as `product preflight --suggest-domains` in a future version.
- **Single acknowledgement for all gaps** — one `--acknowledge-all` flag that silences every gap without requiring per-domain reasoning. Rejected: this is equivalent to suppression with no audit trail. The per-domain reasoning is the point.

**Test coverage:**

Scenario tests:
- `preflight_clean_exits_0.rs` — feature with all cross-cutting ADRs linked and all declared domains covered. Assert `product preflight FT-XXX` exits 0 and prints "Pre-flight clean."
- `preflight_cross_cutting_gap.rs` — ADR-038 is cross-cutting, not linked or acknowledged by FT-009. Assert preflight report names ADR-038. Assert exit code 1.
- `preflight_domain_gap.rs` — FT-009 declares `domains: [security]`, no security ADRs linked or acknowledged. Assert preflight reports security gap with the top-2 security ADRs by centrality named.
- `preflight_acknowledgement_closes_gap.rs` — run `product feature acknowledge FT-009 --domain security --reason "no trust boundaries"`. Re-run preflight. Assert security gap closed. Assert exit 0.
- `preflight_acknowledgement_without_reason_fails.rs` — run `product feature acknowledge FT-009 --domain security --reason ""`. Assert E011. Assert front-matter not mutated.
- `implement_blocked_by_preflight.rs` — FT-009 has preflight gaps. Run `product implement FT-009`. Assert exit 1, preflight error message, no agent invoked.
- `coverage_matrix_renders.rs` — run `product graph coverage` on a fixture with known coverage state. Assert output contains all features and all domains. Assert correct ✓/~/·/✗ symbols.
- `coverage_matrix_json.rs` — run `product graph coverage --format json`. Assert valid JSON with `features` array, each containing `domains` map with coverage status.
- `coverage_matrix_domain_filter.rs` — run `product graph coverage --domain security`. Assert output contains only the security column.
- `author_session_preflight_first.rs` — start `product author feature` for FT-009 with preflight gaps. Assert the first MCP tool call from the session is `product_preflight`, not a content scaffold call.

Invariants:
- Preflight must complete in < 100ms on a repository with 200 features and 100 ADRs. No LLM calls are made during preflight.
- Every `scope: cross-cutting` ADR must appear in the preflight report for every feature. No cross-cutting ADR may be silently omitted.

Exit criteria:
- `product preflight FT-001` on the migrated PiCloud repository identifies at least 2 cross-cutting ADRs and produces a clean report after they are acknowledged.
- `product graph coverage` on the PiCloud repository renders a complete matrix with correct ✓/✗ symbols for all features.


---

## ADR-027: Transitive TC Link Inference — `product migrate link-tests` and `product graph infer`

**Status:** Accepted

**Context:** After migrating a monolithic PRD and ADR document, TC files have `validates.adrs` populated (the parent ADR they were extracted from) but `validates.features` is empty. Feature files have `adrs` populated (after manual review by the developer) but their linked test criteria are unknown. The graph has the data needed to infer the missing TC→Feature edges: if `FT-001 → ADR-002` and `TC-002 → ADR-002`, then `TC-002 → FT-001` follows mechanically.

This inference is not performed during initial migration (ADR-017) because feature→ADR links require human review — they cannot be reliably heuristic-inferred from prose. Once the developer has confirmed the feature→ADR links, the transitive TC links are fully mechanical and safe to infer automatically.

The same inference also applies after any manual `product feature link FT-XXX --adr ADR-XXX` command: new TC→Feature links may follow from the new edge that were not present before.

**Decision:** Implement two inference commands. `product migrate link-tests` is the primary post-migration step. `product graph infer` is the general-purpose command that runs the same inference at any time. Both use the same algorithm, the same dry-run output format, and the same atomic write safety. Both skip cross-cutting ADRs. Both are idempotent and additive — they never remove existing links.

---

### Algorithm

```
For each ADR A where A.scope ≠ cross-cutting:
    F_set = { F | A ∈ F.adrs }                  // features governed by A
    T_set = { T | A ∈ T.validates.adrs }         // TCs that validate A
    for each T ∈ T_set, F ∈ F_set:
        if F ∉ T.validates.features:
            T.validates.features += [F]           // new transitive link
            emit: "TC-%s → FT-%s via ADR-%s"
```

The cross-cutting exclusion is the critical design decision. ADR-001 (Rust) and ADR-013 (error model) are linked to every feature. If their TCs were auto-linked to every feature, the resulting links would be semantically meaningless — a test that validates every feature validates none of them specifically. Cross-cutting ADRs exist to govern platform-wide concerns; their tests are similarly platform-wide. They do not belong in individual feature validation lists.

The correct graph state after inference: every TC that validates a domain-scoped or feature-specific ADR gains links to every feature that uses that ADR. Cross-cutting TC links remain empty — they are validated by `product graph check` separately as a platform-wide concern.

---

### `product migrate link-tests`

Post-migration entry point. Intended to run once, after `product migrate from-prd` and `product migrate from-adrs`, after the developer has manually confirmed feature→ADR links.

```
product migrate link-tests              # infer and apply all transitive TC links
product migrate link-tests --dry-run    # show what would change, write nothing
product migrate link-tests --adr ADR-002  # scope to one ADR's TCs only
```

Dry-run output:

```
Transitive TC link inference (dry run)
────────────────────────────────────────────────────────────
ADR-002 — openraft for Cluster Consensus  [scope: domain]
  TC-002 Raft Leader Election    → +FT-001, +FT-005   (2 new)
  TC-003 Raft Leader Failover    → +FT-001, +FT-005   (2 new)
  TC-004 Raft Learner Join       → +FT-001             (1 new)

ADR-006 — Oxigraph for RDF Projection  [scope: domain]
  TC-008 SPARQL Basic Query      → +FT-001, +FT-003   (2 new)
  TC-009 Graph Projection        → +FT-003             (1 new)

ADR-001 — Rust as Implementation Language  [scope: cross-cutting]
  → skipped (cross-cutting, would link to all features)

ADR-013 — Error Model  [scope: cross-cutting]
  → skipped (cross-cutting, would link to all features)
────────────────────────────────────────────────────────────
8 new TC→Feature links across 5 TCs and 2 ADRs
2 ADRs skipped (cross-cutting)
0 links already existed (idempotent)

Run without --dry-run to apply.
```

---

### `product graph infer`

General-purpose inference command. Runs the same algorithm as `link-tests` but is not migration-specific. Use after any manual feature→ADR link addition to pick up newly implied TC links.

```
product graph infer                      # infer all missing transitive TC links
product graph infer --dry-run
product graph infer --feature FT-009    # scope to one feature's new links
product graph infer --adr ADR-021       # scope to one ADR's TCs
```

`product graph infer` is idempotent — safe to run on any repository at any time with no risk of incorrect mutations. Existing links are never removed.

**Integration with `product feature link`:** when `product feature link FT-009 --adr ADR-021` is run, Product immediately computes the set of TCs that would be inferred and asks:

```
product feature link FT-009 --adr ADR-021

  Linked: FT-009 → ADR-021

  Transitive TC links inferred:
    TC-041 Rate Limit Under Load    → FT-009  (via ADR-021)
    TC-042 Token Bucket Refill      → FT-009  (via ADR-021)

  Add these TC links automatically? [Y/n]
```

If confirmed, the TC links are applied in the same atomic write batch as the ADR link. If declined, the developer can run `product graph infer --feature FT-009` later.

---

### Reverse Inference: ADR→Feature back-link

When `product migrate link-tests` or `product graph infer` adds `FT-001` to `TC-002.validates.features`, it also adds `TC-002` to `FT-001.tests` if not already present. This keeps the bidirectional front-matter consistent — the feature knows about its tests without requiring a separate step.

```
Before inference:
  FT-001.tests = [TC-001]        # only explicitly linked TC
  TC-002.validates.features = [] # empty after migration

After inference:
  FT-001.tests = [TC-001, TC-002]  # TC-002 added by reverse inference
  TC-002.validates.features = [FT-001, FT-005]
```

Both mutations are written atomically in the same operation. If the write fails mid-batch, neither is applied (ADR-015).

---

### Interaction with Cross-Cutting ADRs

Cross-cutting TCs are not linked to individual features. But they must still be validated. The validation model is: cross-cutting TCs appear in the context bundle for every feature (via ADR-025's cross-cutting bundle inclusion rule) and are run as part of the platform-level test suite, not as part of any individual feature's verify step.

`product verify FT-001` runs `FT-001.tests`. Cross-cutting TCs are not in that list. They are run by `product verify --platform` — a separate command that runs all TCs linked to cross-cutting ADRs regardless of feature association. This keeps feature-level verification fast and focused while ensuring platform-wide invariants are still exercised.

---

**Rationale:**
- Transitive inference is sound for domain-scoped ADRs. The relationship is: feature uses ADR → TC validates that ADR's constraints → TC validates the feature's use of those constraints. No human judgment is required; the relationship is mechanical.
- The cross-cutting exclusion is not optional. Without it, `product migrate link-tests` on a repository where every feature links to ADR-001 (Rust) would add every feature to every TC that validates ADR-001 — tens or hundreds of spurious links. These links would inflate W002 resolution numbers without adding analytical value.
- The interactive confirmation in `product feature link` is the right UX for ongoing development. During migration, the developer runs `link-tests` once as a batch. During active development, they want to know immediately what TC links follow from a new ADR link — not discover it on the next `graph check` run.
- Reverse inference (updating `FT.tests` when `TC.validates.features` is updated) maintains the invariant that the graph is consistent from both directions. Without it, `FT-001.tests` would be out of date until the developer manually ran `product feature link FT-001 --test TC-002`, which they would rarely remember to do.

**Rejected alternatives:**
- **Infer links during `migrate from-adrs` without confirmed feature→ADR links** — feature→ADR links are not reliably inferred by the migration heuristic. Inferring TC→Feature links on top of unconfirmed feature→ADR links propagates the heuristic error into TC front-matter. Two layers of approximation produce unreliable results. Rejected: inference must wait for confirmed links.
- **Include cross-cutting ADR TCs in feature test lists** — meaningful for platform integrity but makes `product verify FT-001` run the entire platform test suite for every feature. Feature verification becomes slow and its output noisy. `product verify --platform` is the right separation.
- **Manual TC linking only** — without `link-tests`, a repository with 30 features and 60 TCs requires 180 manual link operations after migration (each of 60 TCs linked to its 3 average features). This friction is prohibitive and ensures the links are never completed. Automation with dry-run review is the correct balance.
- **Bidirectional sync as a continuous background process** — a daemon watches for file changes and updates links automatically. Rejected: daemon lifecycle complexity (ADR-003 reasoning applies). The interactive confirmation in `product feature link` provides the same real-time benefit without a daemon.

**Test coverage:**

Scenario tests:
- `link_tests_basic.rs` — FT-001 links ADR-002. TC-002 validates ADR-002. Run `product migrate link-tests`. Assert TC-002 gains `validates.features: [FT-001]`. Assert FT-001 gains `tests: [TC-002]`.
- `link_tests_multi_feature.rs` — FT-001 and FT-005 both link ADR-002. TC-002 validates ADR-002. Assert TC-002 gains both FT-001 and FT-005.
- `link_tests_cross_cutting_excluded.rs` — ADR-001 is cross-cutting. TC-001 validates ADR-001. All features link ADR-001. Run `link-tests`. Assert TC-001.validates.features remains empty.
- `link_tests_idempotent.rs` — run `product migrate link-tests` twice. Assert file content identical after both runs. Assert second run reports "0 new links."
- `link_tests_dry_run_no_write.rs` — run `product migrate link-tests --dry-run`. Assert zero files modified. Assert stdout contains the inference plan.
- `link_tests_adr_scope.rs` — run `product migrate link-tests --adr ADR-002`. Assert only TCs linked to ADR-002 are updated. TCs for ADR-006 unchanged.
- `graph_infer_general.rs` — add FT-009 → ADR-021 link. Run `product graph infer --feature FT-009`. Assert TC-041 and TC-042 (which validate ADR-021) gain FT-009 in their features list.
- `feature_link_interactive_confirm.rs` — run `product feature link FT-009 --adr ADR-021`. Assert interactive prompt shows inferred TC links. On confirmation, assert TC links applied atomically with the ADR link.
- `feature_link_interactive_decline.rs` — decline the interactive TC link prompt. Assert only the ADR link is applied. Assert TC files unchanged.
- `reverse_inference_updates_feature.rs` — after inference adds FT-001 to TC-002.validates.features, assert FT-001.tests now includes TC-002.
- `atomic_batch_write.rs` — inject a write failure midway through a multi-file inference batch. Assert all-or-nothing: either all files updated or none. Assert no partial state.
- `platform_verify_cross_cutting.rs` — run `product verify --platform`. Assert TCs linked to cross-cutting ADRs are run. Assert their status is updated. Assert feature-specific TCs are not run.

Invariants:
- After any `link-tests` or `graph infer` run: for every non-cross-cutting ADR A, for every feature F in F.adrs, for every TC T in T.validates.adrs where A is in T.validates.adrs — F must be in T.validates.features. Verified by a property test on arbitrary graph states.
- Cross-cutting ADR TCs must never appear in any feature's `tests` list. Verified by checking all features after any inference run.

Exit criteria:
- `product migrate link-tests` on the migrated PiCloud repository produces ≥ 20 new TC→Feature links after feature→ADR links are confirmed. `product graph check` W002 warning count drops by ≥ 50% after running `link-tests`.

---

## ADR-029: Code Structure and Quality Standards

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

**Test coverage:**

Scenario tests (these test the check scripts themselves, not the Product codebase):
- `file_length_passes.rs` — create a temp Rust project where all files are under 300 lines. Run `file-length.sh`. Assert exit 0.
- `file_length_warn.rs` — add a 350-line file. Run `file-length.sh`. Assert exit 2. Assert the file name appears in output.
- `file_length_fail.rs` — add a 450-line file. Run `file-length.sh`. Assert exit 1. Assert the file name and line count appear in output.
- `function_length_passes.rs` — all functions under 30 statement lines. Assert exit 0.
- `function_length_warn.rs` — one function with 35 statement lines. Assert exit 2.
- `function_length_fail.rs` — one function with 45 statement lines. Assert exit 1 with file path and line number.
- `module_structure_passes.rs` — all required modules present, `main.rs` under 80 lines. Assert exit 0.
- `module_structure_missing.rs` — delete `src/graph/`. Assert exit 1 naming `src/graph/`.
- `module_structure_main_too_long.rs` — `main.rs` with 100 lines. Assert exit 1 with line count.
- `single_responsibility_passes.rs` — all files begin with single-sentence `//!` doc comment without "and". Assert exit 0.
- `single_responsibility_missing.rs` — file with no `//!` first line. Assert exit 1 naming the file.
- `single_responsibility_and.rs` — file with `//! Graph construction and traversal.` Assert exit 1 with the violating comment.

Invariants:
- All five check scripts must themselves pass their own checks — `scripts/checks/*.sh` must be under 100 lines each (shell, not Rust, but same principle applies).

Exit criteria:
- All five TC-CQ scripts pass on the Product codebase at end of Phase 1. No source file exceeds 400 lines. No function exceeds 40 statement lines. All required modules exist. All files have single-responsibility doc comments.

---

## ADR-030: Dependency Artifact Type — First-Class External System Declarations

**Status:** Accepted

**Context:** ADRs capture architectural decisions — why a dependency was chosen, what was rejected, and the rationale. They do not capture the runtime facts about a dependency: what version is required, what interface it exposes, whether it is currently available, and which features depend on it. These facts are different in kind from decision rationale and serve different consumers.

Four problems exist without a dependency artifact type:

1. **Preflight gaps** — `product preflight FT-007` checks domain coverage and spec gaps, but cannot check whether PostgreSQL 14 is running on port 5432. There is no structured place to declare that requirement.

2. **Missing graph edges** — `product impact DEP-001` cannot exist because "openraft" is not a graph node. When openraft releases a breaking change, there is no query that returns every affected feature. That impact is discoverable only by reading every ADR.

3. **Context bundle blind spots** — an agent implementing a feature that calls an external API needs the interface contract: auth mechanism, rate limits, error model. This information is not in decision rationale and is not currently in context bundles.

4. **No dependency bill-of-materials** — there is no command that returns every external dependency the product has, across all features, with versions and availability checks. This is valuable for security audits, upgrade planning, and onboarding.

ADRs remain the right home for the *decision* to use a dependency. `DEP-XXX` artifacts are the right home for the *runtime facts* about that dependency.

**Decision:** Add `Dependency` (`DEP-XXX`) as a first-class artifact type. Dependencies declare their type, version constraint, interface description, and an optional availability check command. They are linked to features via a `uses` edge and to ADRs via a `governs` edge. Preflight, TC prerequisites, context bundles, impact analysis, and gap analysis all integrate with the new type.

---

### Dependency Types

| Type | Meaning | Availability check pattern |
|---|---|---|
| `library` | Build-time code dependency (crate, npm, NuGet, Maven) | Usually none — version managed by package manifest |
| `service` | Runtime service that must be running (database, queue, cache, message broker) | TCP check, health endpoint, CLI ping |
| `api` | External HTTP or gRPC API | HTTP health check, auth validation |
| `tool` | CLI tool required at runtime or in CI | `which tool && tool --version` |
| `hardware` | Physical hardware requirement | `uname`, device node presence check |
| `runtime` | Execution environment (OS version, SDK, JVM) | Version check command |

---

### Front-Matter Schema

**Library dependency:**

```yaml
---
id: DEP-001
title: openraft
type: library
source: crates.io
version: ">=0.9,<1.0"
status: active           # active | deprecated | evaluating | migrating
features: [FT-001, FT-002, FT-005]
adrs: [ADR-002]          # decision that governs use of this dependency
availability-check: ~    # null — library, no runtime check
breaking-change-risk: medium   # low | medium | high
---

This crate provides Raft consensus with pluggable storage and network layers.
It is used for leader election, log replication, and cluster membership.
```

**Service dependency:**

```yaml
---
id: DEP-005
title: PostgreSQL Event Store
type: service
version: ">=14"
status: active
features: [FT-007, FT-012]
adrs: [ADR-015]
interface:
  protocol: tcp
  port: 5432
  auth: md5
  connection-string-env: DATABASE_URL
  health-endpoint: ~
availability-check: "pg_isready -h ${PG_HOST:-localhost} -p ${PG_PORT:-5432}"
breaking-change-risk: low
---

PostgreSQL is used as the backing store for the event log in development and
test environments. Production uses the embedded storage layer (DEP-002).
```

**API dependency:**

```yaml
---
id: DEP-007
title: GitHub Container Registry
type: api
version: "v2"
status: active
features: [FT-018]
adrs: [ADR-022]
interface:
  base-url: https://ghcr.io
  auth: bearer-token
  auth-env: GHCR_TOKEN
  rate-limit: 5000/hour
  error-model: OCI Distribution Spec v1.1
availability-check: >
  curl -sf -H "Authorization: Bearer ${GHCR_TOKEN}"
  https://ghcr.io/v2/ > /dev/null
breaking-change-risk: low
---
```

**Hardware dependency:**

```yaml
---
id: DEP-010
title: Raspberry Pi 5 — NVMe Storage
type: hardware
version: ~
status: active
features: [FT-001, FT-004]
adrs: [ADR-001]
interface:
  arch: aarch64
  storage-min-gb: 500
  storage-device-pattern: /dev/nvme*
availability-check: >
  uname -m | grep -q aarch64
  && ls /dev/nvme* 2>/dev/null | head -1 | grep -q nvme
breaking-change-risk: low
---
```

---

### Dependency Statuses

| Status | Meaning |
|---|---|
| `active` | In use, maintained, current version in use |
| `evaluating` | Under consideration — not yet committed |
| `deprecated` | Scheduled for removal, features migrating away |
| `migrating` | Active migration in progress to a successor dependency |

When `status: deprecated` or `migrating`, `product graph check` emits W013 ("feature uses a deprecated dependency") for every feature still linked to it.

---

### New Graph Edges

| Edge | From | To | Description |
|---|---|---|---|
| `uses` | Feature | Dependency | Feature requires this dependency at runtime |
| `governs` | ADR | Dependency | Decision that chose this dependency |
| `supersedes` | Dependency | Dependency | This dependency replaces another (migration) |

The reverse of every edge is traversable. `product impact DEP-001` uses reverse-graph BFS to find every feature and ADR that would be affected by a breaking change in openraft.

---

### Integration: Preflight

`product preflight FT-XXX` is extended to check dependency availability. For each DEP linked to the feature where `availability-check` is non-null, Product executes the check command. The same semantics as `[verify.prerequisites]` — exit 0 = satisfied, non-zero = not satisfied. Product never installs or starts dependencies; it only checks them.

```
product preflight FT-007

━━━ Dependency Availability ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  DEP-001  openraft        [library — no check]    ✓
  DEP-005  PostgreSQL 14+  [pg_isready ...]         ✗ not running
  DEP-002  embedded store  [library — no check]    ✓

━━━ Cross-Cutting ADRs ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ...
```

Dependency availability failures are warnings (exit 2), not errors (exit 1) — the agent can still implement the feature without the dependency running; it just cannot run tests that require it. The TC `requires` mechanism (ADR-021) handles the runtime gate separately.

---

### Integration: TC `requires` Field

TCs can reference DEP IDs directly in `requires`:

```yaml
---
id: TC-042
title: Event Store Persistence
type: scenario
requires: [DEP-005]          # resolves to DEP-005.availability-check automatically
runner: bash
runner-args: ["scripts/test-harness/event-store.sh"]
---
```

Product resolves `DEP-005` to its `availability-check` command. No need to duplicate the check command in `[verify.prerequisites]` — the dependency declaration is the single source of truth. Named prerequisite strings in `[verify.prerequisites]` still work for non-dependency checks.

---

### Integration: Context Bundles

A "Dependencies" section is inserted into context bundles after ADRs and before test criteria:

```markdown
## Dependencies

### DEP-001 — openraft [library, >=0.9,<1.0]

[dependency body text, front-matter stripped]

Interface: no runtime interface (build-time library)
Availability: no check required

### DEP-005 — PostgreSQL Event Store [service, >=14]

[dependency body text, front-matter stripped]

Interface:
  protocol: tcp / port: 5432
  auth: md5 / env: DATABASE_URL
  availability-check: pg_isready -h ${PG_HOST:-localhost} -p 5432
```

An agent implementing a feature receives the complete interface contract for every external dependency. It knows what environment variables to read, what ports to connect to, what auth mechanism to use, and what check to run to verify the dependency is present.

---

### Integration: Impact Analysis

`product impact DEP-001` performs reverse-graph BFS from the dependency node:

```
product impact DEP-001

Impact analysis: DEP-001 — openraft

Direct dependents:
  Features:  FT-001 (in-progress), FT-002 (complete), FT-005 (planned)
  ADRs:      ADR-002 (governs)

Transitive dependents (via feature dependencies):
  Features:  FT-007 (planned) — depends-on FT-001

Breaking change risk: medium
Summary: 4 features, 1 ADR. 1 feature already complete may need revisiting.
```

---

### Integration: Gap Analysis — G008

New gap code for gap analysis (ADR-019):

| Code | Severity | Description |
|---|---|---|
| G008 | medium | Feature uses a dependency (`uses` edge to DEP) with no ADR governing its use (`governs` edge from any ADR to that DEP) |

This enforces the principle that every external dependency choice is documented as an architectural decision. A feature that adds a new `uses` edge to a DEP without a corresponding ADR is a specification gap.

---

### New Commands

```
product dep list                      # all dependencies with status
product dep list --type service       # filter by type
product dep list --status deprecated  # find deprecated deps
product dep show DEP-001              # full dependency detail
product dep features DEP-001          # which features use this dependency
product dep check DEP-005             # run availability check manually
product dep check --all               # run all availability checks
product dep bom                       # full dependency bill of materials
product dep bom --format json         # machine-readable for security audits
```

`product dep bom` produces a structured bill of materials across all features:

```
Dependency Bill of Materials — product v0.1

Libraries (build-time):
  DEP-001  openraft          >=0.9,<1.0    crates.io   active
  DEP-003  oxigraph          >=0.4         crates.io   active
  DEP-004  clap              >=4.0         crates.io   active

Services (runtime):
  DEP-005  PostgreSQL        >=14          —           active (dev/test only)

Hardware:
  DEP-010  Raspberry Pi 5    —             —           active

Total: 5 dependencies across 3 types
Breaking change risk: 1 medium (DEP-001), 4 low
```

---

### New Validation Codes

| Code | Tier | Description |
|---|---|---|
| E013 | Dependency | Dependency has no linked ADR — every dependency requires a governing decision |
| W013 | Validation | Feature uses a deprecated or migrating dependency |
| W015 | Validation | Dependency `availability-check` failed during preflight |

E013 is a hard error (exit code 1). Every external dependency is an architectural choice — why this library over alternatives, what version constraint, what the tradeoffs are. That choice belongs in an ADR. A dependency without an ADR is an undocumented decision, which is the same class of problem as a broken link: the graph is structurally incomplete.

`product dep new "openraft" --type library` scaffolds both the `DEP-XXX` file and an `ADR-XXX` stub linked to it. The author is prompted to complete the ADR. Creating a DEP without creating or linking an ADR is a deliberate friction point — the tool makes the easy path the correct path.

G008 (LLM-detected undocumented dependency decisions) remains in gap analysis but is now a backstop for the rare case where a DEP has an ADR link that doesn't actually document the decision clearly. E013 handles the structural absence; G008 handles the semantic absence.

---

**Rationale:**
- Separating dependency facts from decision rationale keeps each artifact focused. An ADR that also contains version constraints, interface specs, and availability check commands becomes a kitchen-sink document. `DEP-XXX` is the right unit for runtime facts.
- The `uses` edge from Feature to Dependency is the missing link in the graph. Without it, `product impact DEP-001` cannot exist. With it, a single command returns the full blast radius of any dependency change.
- Availability checks co-located with the dependency declaration are the single source of truth. The same check is used by preflight, by TC `requires` resolution, and by `product dep check`. No duplication, no drift.
- The six dependency types cover the actual range of external dependencies in real projects without over-engineering. `library` and `service` handle 80% of cases. `api`, `tool`, `hardware`, and `runtime` handle the rest.
- `breaking-change-risk` is a human-declared field, not computed. It communicates intent: "this dependency is stable and unlikely to break" vs. "this is a pre-1.0 library and we expect breaking changes." It informs prioritisation of upgrade work.

**Rejected alternatives:**
- **Dependencies modelled only as ADRs** — the current state. ADRs capture decisions, not runtime facts. A developer reading ADR-002 knows why openraft was chosen; they do not know what environment variable the connection string lives in, what port the service binds to, or what command to run to check it. The information requirements are different.
- **Dependencies as front-matter fields on features** — `external-deps: [openraft>=0.9, postgres>=14]`. Duplicates across features sharing the same dependency. No graph node to query. No `product impact` possible. Rejected.
- **Using `[verify.prerequisites]` for all dependency checks** — `[verify.prerequisites]` is a project-level dictionary of named shell commands. It is not typed, versioned, or linked to features. It cannot be queried. It doesn't produce a bill of materials. Rejected as insufficient at scale.
- **Separate dependency management tool** — a `product-deps` companion binary. Rejected: the dependency graph and the artifact graph are the same graph. Separating them into different tools creates the synchronisation problem that the unified graph is designed to avoid.

**Test coverage:**

Scenario tests:
- `dep_parse_library.rs` — parse a `library` type dependency. Assert all fields deserialise correctly. Assert `availability-check: ~` parses to `None`.
- `dep_parse_service.rs` — parse a `service` type dependency with `interface` block. Assert interface fields (protocol, port, auth, env) are present.
- `dep_uses_edge.rs` — feature links `uses: [DEP-001]`. Assert graph contains `FT-001 →uses→ DEP-001` and reverse `DEP-001 →usedBy→ FT-001`.
- `dep_governs_edge.rs` — ADR links `governs: [DEP-001]`. Assert graph contains both directions.
- `dep_impact_direct.rs` — DEP-001 linked to FT-001 and FT-002. Assert `product impact DEP-001` names both features as direct dependents.
- `dep_impact_transitive.rs` — FT-003 depends-on FT-001; FT-001 uses DEP-001. Assert `product impact DEP-001` includes FT-003 in transitive dependents.
- `dep_preflight_check_passes.rs` — DEP-005 has `availability-check` that exits 0. Run `product preflight FT-007`. Assert DEP-005 shows as available.
- `dep_preflight_check_fails.rs` — DEP-005 availability check exits 1. Assert preflight report names DEP-005 as unavailable. Assert exit code 2 (warning, not error).
- `dep_tc_requires_dep_id.rs` — TC declares `requires: [DEP-005]`. Product resolves to DEP-005's availability check. Assert the resolved check command matches DEP-005 `availability-check`.
- `dep_context_bundle_section.rs` — feature uses DEP-001 and DEP-005. Assert context bundle contains a "Dependencies" section with both entries, interface block included for DEP-005.
- `dep_bom_output.rs` — run `product dep bom`. Assert output groups by type, lists all active dependencies. Assert `--format json` produces valid JSON.
- `dep_bom_json_schema.rs` — assert JSON BOM output contains for each dep: id, title, type, version, status, features (list), breaking-change-risk.
- `dep_w013_deprecated.rs` — DEP-005 status `deprecated`. Feature FT-007 uses DEP-005. Run `product graph check`. Assert W013 naming FT-007 and DEP-005.
- `dep_e013_no_adr.rs` — DEP-005 has no `adrs` links. Run `product graph check`. Assert exit code 1 and E013 naming DEP-005 with the message "every dependency requires a governing decision."
- `dep_gap_g008.rs` — feature uses DEP-005. No ADR has `governs: [DEP-005]`. Run `product gap check FT-007`. Assert G008 finding.
- `dep_list_filter.rs` — run `product dep list --type service`. Assert only service-type dependencies returned.
- `dep_check_manual.rs` — run `product dep check DEP-005` with availability check that exits 0. Assert output shows check passed. With exit 1: assert shows failed.
- `dep_supersedes_edge.rs` — DEP-011 supersedes DEP-005. Assert graph contains `DEP-011 →supersedes→ DEP-005`. Assert `product impact DEP-005` includes DEP-011 in dependents.

Invariants:
- A dependency with `availability-check: ~` (null) never causes preflight to report a check failure — null checks are always considered satisfied.
- `product dep bom` must include every DEP artifact in the repository regardless of feature links. Orphaned DEPs appear in the BOM with `features: []`.

Exit criteria:
- `product dep bom` on the migrated PiCloud repository produces a complete BOM with correct type groupings.
- `product impact DEP-001` returns at least FT-001 and FT-002 after migration and feature→DEP link setup.
- TC `requires: [DEP-005]` resolves to DEP-005's availability check without requiring a matching entry in `[verify.prerequisites]`.
