---
id: TC-500
title: request draft lists drafts directory entries
type: scenario
status: unimplemented
validates:
  features:
  - FT-041
  adrs:
  - ADR-038
phase: 1
runner: cargo-test
runner-args: tc_500_request_draft_lists_drafts_directory_entries
---

Validates FT-041 / ADR-038 decision 12.

**Setup:** fixture with a `.product/requests/` directory containing two draft YAMLs and one non-YAML file (README.md).

**Act 1:** run `product request draft`.

**Assert 1:**
- Exit code 0
- Output lists the two YAML files with their filenames and (if parseable) their `type:` and first line of `reason:`
- The non-YAML file is either skipped silently or listed with a clear "unparseable" marker
- Drafts are listed in mtime-descending order (most recently edited first)

**Act 2:** run `product request create` without a pre-existing drafts directory.

**Assert 2:**
- `.product/requests/` is created
- A new draft file is written with a timestamp-prefixed name (e.g. `2026-04-17T08-30-42-create.yaml`)
- The file contains a template (type scaffold, `reason:`, empty `artifacts:` or `changes:` section)
- `$EDITOR` is invoked on the new file (in interactive mode); in non-interactive / CI mode, just the file path is printed

**Act 3:** apply a request YAML from an arbitrary path outside `.product/requests/`.

**Assert 3:**
- `product request apply /tmp/some-request.yaml` works identically to applying from the drafts dir — drafts dir is convention, not a store
