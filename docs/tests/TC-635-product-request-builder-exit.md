---
id: TC-635
title: product_request_builder_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-052
  adrs:
  - ADR-044
phase: 5
---

## Exit Criteria — FT-052 Product Request Builder

FT-052 is complete when all of the following hold:

1. `product request new create|change` creates
   `.product/requests/draft.yaml`; `product request continue`
   resumes it; `product request discard` deletes it; one active
   draft per working directory is enforced.
2. Every `product request add …` subcommand (feature, adr, tc,
   dep, doc, target, acknowledgement) appends to the draft and
   runs incremental structural validation against the draft plus
   the existing graph, completing in under 100ms.
3. `product request add dep --adr new` creates both the dep and
   its governing ADR in the same step and reports E013 as
   satisfied within the draft.
4. `product request status` renders all artifacts with ✓ / ⚠ / ✗
   indicators and a warning / error count; `product request
   show` prints the raw draft YAML.
5. `product request submit` validates the full draft, refuses on
   any E-class finding (leaving the draft byte-identical and no
   files written), and on success applies atomically, archives
   the draft, and emits one entry to the request log.
6. A draft produced by the builder and a hand-written YAML of
   the same intent validate and apply identically — there is no
   builder-only capability.
7. `product.toml` supports `[request-builder]` with
   `warn-on-warnings` in `{always, warn, block}` and the
   behaviour matches the spec.
8. `cargo test`, `cargo clippy -- -D warnings -D
   clippy::unwrap_used`, and `cargo build` all pass.
9. Every TC under FT-052 has `runner: cargo-test` and
   `runner-args` set to the integration test function name.
