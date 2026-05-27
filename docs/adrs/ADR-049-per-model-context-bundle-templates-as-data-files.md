---
id: ADR-049
title: Per-Model Context Bundle Templates as Data Files
status: accepted
features:
- FT-070
- FT-071
- FT-072
- FT-073
- FT-074
- FT-075
supersedes: []
superseded-by: []
domains:
- api
- observability
scope: cross-cutting
content-hash: sha256:d301365ce79df7402f8a5469223b76196bafb1325b5c162044f5f1c265639af3
---

**Status:** Proposed

**Context:** ADR-006 established the context bundle as the primary LLM interface and explicitly rejected adding a `--max-tokens` flag — token-budget management is the agent's responsibility. ADR-006 was silent on how the bundle is structurally rendered.

The earlier `--for-llm` flag (from `product-context-llm-format-spec.md`) introduced an XML-tagged variant of the bundle aimed at Claude. In practice we now ship to a heterogeneous fleet of models with three rendering-relevant differences:

- **Format preference.** Claude is trained to recognise XML tags as a prompt-organising mechanism. GPT-4 prefers Markdown. GPT-3.5 / GPT-4o-mini prefer JSON for programmatic parsing. Gemini does well with YAML.
- **Context window size.** Claude Opus 4.7 has ~1M tokens; Claude Haiku 4.5 has a smaller working budget; GPT-4o-mini has yet another. The bundle that fits comfortably in one model exceeds the working budget of another.
- **Attention patterns.** Some models attend more strongly to the start of the prompt; some to the end. Where critical content sits matters, and matters differently per model.

A single hard-coded XML form (or a single Markdown form) is the wrong fit for at least three of the four model families we run routines through.

**Decision:** Per-model rendering choices for `product context` are expressed as **data templates** loaded at invocation time. A template is a TOML file declaring structural format, section ordering, and informational token-budget hints for a target model. Templates are selected via a `--target NAME` flag (or the equivalent MCP `target` parameter); the previous `--for-llm` flag becomes a deprecated alias for `--target claude-opus`.

Three sub-decisions are locked together by this ADR:

1. **Templates are data, not code.** A template is a TOML file. There is no template language, no logic, no loops, no conditionals. A malformed template can produce a suboptimal rendering but cannot produce incorrect bundle content.
2. **Built-in templates ship as files, not embedded in the binary.** They live in `$PRODUCT_INSTALL/templates/` and can be copied into `~/.product/templates/` or `.product/templates/` for user / repo overrides. Resolution order is repo-local → user → built-in; first match wins.
3. **Summarisation is out of scope for v1.** Templates choose structural format and ordering. They cannot summarise, filter, or otherwise modify artifact body content. Bundle assembly remains the source of truth; templates only choose how to display it. Token-budget settings are informational warnings — bundles are never auto-truncated.

**Rationale:**

- **Data templates keep the rendering layer safe.** A template malfunction produces ugly output, never wrong output. The bundle's logical content is determined by the assembly pipeline (depth, selection, supersession collapse) which sits behind the template layer. ADR-006's "complete and accurate bundle" invariant is preserved.
- **Files-not-embedded means teams can iterate without a Product release.** A team that wants a custom layout copies a built-in template, edits it, drops it in `.product/templates/`, and uses `--target my-template`. No fork, no rebuild, no upgrade gate. This mirrors ADR-022's decision to ship system prompts as versioned files rather than baking them into the binary.
- **Summarisation is a separate, much harder problem.** Body compression interacts with model-specific tokenisation, with prompt-injection risk, and with the user's expectation that "what `product context` shows me is what the agent sees". Conflating summarisation with rendering would push Product across the knowledge boundary established by ADR-021. We keep them separate.
- **A closed allowlist of section names keeps the contract auditable.** Templates can choose ordering and inclusion among a known set; they cannot invent new sections. This means the assembly pipeline knows exactly which structural slots exist, and the validator can flag typos before runtime.
- **Resolution order favours the closest authority.** Repo-local templates win over user templates, which win over built-ins. Read-only built-ins ensure `product context templates --reset NAME` always has a coherent fallback.

**Rejected alternatives:**

- **A template language (Liquid, Handlebars, Tera).** Considered for the expressiveness — conditional sections, per-artifact loops, computed metadata. Rejected: the failure mode is "the template ran logic and produced wrong content", which violates the ADR-006 invariant. The closed-allowlist approach gives 95% of the value with none of the safety cost.
- **Embed built-in templates in the binary via `include_str!`.** Considered for atomicity — the binary always ships with consistent templates, no install-step coordination. Rejected: users cannot inspect, copy, or evolve them without a code change. `product context templates --show NAME` becomes useful precisely because templates are real files.
- **Per-flag rendering knobs (`--xml`, `--markdown`, `--with-deliverables`, `--no-bundle-metrics`).** Considered as a lighter mechanism. Rejected at six-flag count — flags compose poorly, are not shareable across teams, and cannot capture model-specific token-budget hints. A template captures all knobs in one named bundle.
- **Summarising artifact bodies inside the rendering layer.** Considered for the small-context-window case (Claude Haiku, GPT-4o-mini). Rejected for v1: summarisation requires model awareness Product does not have, and would silently change what the agent sees relative to what `product feature show` shows. Out of scope until proven necessary.
- **`--max-tokens` truncation.** Already rejected by ADR-006. This ADR does not relitigate that decision; templates' `token_budget` keys are warnings only.
- **Auto-detect target from environment / model name.** Considered for ergonomics. Rejected: too many false positives, and the routine prompts (ADR-022 system prompts) are the right place to specify target. Each routine ships with the explicit target it expects.

**Implications:**

- **CLI surface.** `product context FT-XXX --target NAME` is the new flag. `product context templates [--show NAME | --where | --reset NAME]` manages templates. The `--for-llm` flag is retained as a deprecated alias for `--target claude-opus` and emits a stderr deprecation note.
- **Config.** `product.toml` gains `[context].default-target`. Unset falls back to `human` to preserve backward compatibility (`product context FT-XXX` without flags still produces terminal-readable Markdown).
- **MCP.** The `product_context` tool gains a `target` parameter; the response gains `format`, `target`, `token_count_approx`, `exceeded_target_max`, `exceeded_hard_max` fields.
- **Validation.** `product graph check` validates resolved templates at startup. Invalid templates are excluded from the available targets list with a startup warning; they do not block the binary from running on other targets. Templates with a newer `schema_version` than the binary recognises are rejected.
- **Routines.** Step 3 of `implement-next-feature.md` (and analogous steps in other routines) explicitly sets `--target claude-opus` rather than relying on the repo's `default-target`. This isolates routines from local config drift.

**Test coverage:** TC-742 through TC-766 (25 session tests covering parse, validation, resolution, format-per-target, ordering, defaults, list/show/reset, MCP parity, and the `--for-llm` deprecation alias). TC-767 is the FT-063 exit-criteria.

**Schema-version compatibility:** The template format itself carries `schema_version = 1`. Future bumps follow ADR-014 conventions: additive within a major version; deprecations precede removals. Templates with an unrecognised newer version are rejected with an upgrade message, never silently downgraded.
