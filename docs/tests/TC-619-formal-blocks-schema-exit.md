---
id: TC-619
title: formal_blocks_schema_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-049
  adrs:
  - ADR-031
phase: 5
runner: cargo-test
runner-args: "tc_619_formal_blocks_schema_exit"
---

## Exit Criteria — FT-049 Formal Blocks in LLM Schema Output

FT-049 is complete when all of the following hold:

1. `product schema` includes a `## Formal Blocks` section (TC-617) listing
   all five AISP blocks with examples and "required by" annotations.
2. `product schema --type formal` renders only the Formal Blocks section,
   without the other four artifact schemas (TC-618).
3. The Test Criterion schema contains a cross-reference line pointing at
   `Formal Blocks` so an LLM reading `type: invariant` can discover the
   block grammar without external context (TC-617).
4. `product agent-init` regenerates AGENT.md with the new section.
5. An LLM-authored `type: invariant` TC guided solely by the schema output
   passes `product graph check` with no W004 warnings.
6. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`, and
   `cargo build` all pass.
7. Every TC under FT-049 has `runner: cargo-test` and `runner-args` set to
   the integration test function name.
