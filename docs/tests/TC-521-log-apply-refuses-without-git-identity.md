---
id: TC-521
title: log apply refuses without git identity
type: scenario
status: unimplemented
validates:
  features: []
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_521_log_apply_refuses_without_git_identity
---

## Description

`product request apply` refuses with a clear error when `git config user.name` or `git config user.email` is not set.

## Setup

1. Fixture repository where git is initialised but `user.name` and `user.email` are both unset (`git config --local` removes any repo-level values; `HOME=/tmp/empty-home` ensures no global config).
2. A valid request YAML.

## Steps

1. Run `product request apply request.yaml`.
2. Assert exit code ≥ 1.
3. Assert stderr mentions "git identity" or "git config" and either "user.name" or "user.email".
4. Assert `requests.jsonl` is unchanged (no entry appended).
5. Assert no artifact files were written.

## Invariant

Apply refuses to fabricate an identity. No entry can ever lack a meaningful `applied-by`.
