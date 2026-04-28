---
id: TC-698
title: implement_pipeline_honors_per_repo_implement_prompt
type: scenario
status: passing
validates:
  features:
  - FT-056
  adrs:
  - ADR-022
phase: 5
runner: cargo-test
runner-args: "tc_698_implement_pipeline_honors_per_repo_implement_prompt"
last-run: 2026-04-28T09:50:28.631429697+00:00
last-run-duration: 0.2s
---

**Test Type:** scenario

**Setup:**

1. Create a tempdir repo with `product init`.
2. Apply a `product request` that creates a feature `FT-X` with one
   linked TC (so `product implement FT-X` has a target). Marking
   the TC as `runner: bash` with a no-op runner-args is enough —
   the test only exercises the `--dry-run` path which exits before
   agent invocation, so verify is never called.
3. Write a sentinel string into
   `<repo>/benchmarks/prompts/implement-v1.md`, e.g.
   `# CUSTOM IMPLEMENT PROMPT — sentinel-9f3b2a`.

**Execution (override path):**

1. Run `product implement FT-X --dry-run` and capture stdout.
2. Parse the line `Context file: <path>` from stdout.
3. Read the file at `<path>`.

**Expected (override path):**

- The captured file content begins with the sentinel string from
  `benchmarks/prompts/implement-v1.md`.
- The dynamic suffix (TC status table, the
  `When done: product verify FT-X` instruction, and the context
  bundle) appears below the sentinel.
- Process exit code is 0 (because `--dry-run` stops before agent
  invocation).

**Execution (fallback path):**

1. Delete `<repo>/benchmarks/prompts/implement-v1.md`.
2. Re-run `product implement FT-X --dry-run`.
3. Read the new context file.

**Expected (fallback path):**

- The file content includes the embedded
  `src/author/prompts/implement.txt` body (the fallback used by
  `author::prompts::get` when the override is absent).
- The dynamic suffix is still appended below.

**Negative case:**

- Run the same flow on a repo where
  `benchmarks/prompts/implement-v1.md` exists but is empty. The
  prompt file is still produced (no panic, no error) and contains
  only the dynamic suffix. Process exits 0.