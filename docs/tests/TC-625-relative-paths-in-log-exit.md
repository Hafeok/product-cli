---
id: TC-625
title: relative_paths_in_log_exit
type: exit-criteria
status: unimplemented
validates:
  features:
  - FT-051
  adrs:
  - ADR-039
phase: 5
runner: cargo-test
runner-args: "tc_625_relative_paths_in_log_exit"
---

## Exit Criteria — FT-051 Relative Paths in the Request Log

FT-051 is complete when all of the following hold:

1. Every path field emitted by `append_apply_entry`,
   `append_undo_entry`, and `append_migrate_entry` is relativised against
   `repo_root` and uses POSIX separators (TC-623).
2. Two clones of the same repo at different absolute paths produce
   byte-identical `file:` values in new log entries (TC-623 invariant).
3. `product request-log migrate-paths` appends a `migrate` entry with
   sentinel `path-relativize`, rewrites offending lines in place, and
   leaves `product request-log verify` exiting 0 (TC-624).
4. `product request-log verify` exits 0 on a fresh post-FT-051 log with
   no warnings, and emits `W-path-absolute` on an unmigrated absolute
   path — loud when it shouldn't happen (TC-624).
5. Writes outside `repo_root` (escape paths) keep the absolute path and
   trigger `W-path-absolute` — deliberately not silently relativised.
6. The hash chain's `entry-hash` field is not recomputed for historical
   entries; the migrate entry is the authority for the rewrite.
7. `cargo test`, `cargo clippy -- -D warnings -D clippy::unwrap_used`,
   and `cargo build` all pass.
8. Every TC under FT-051 has `runner: cargo-test` and `runner-args` set
   to the integration test function name.
