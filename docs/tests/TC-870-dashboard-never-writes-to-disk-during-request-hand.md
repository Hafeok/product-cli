---
id: TC-870
title: dashboard never writes to disk during request handling
type: invariant
status: unimplemented
validates:
  features:
  - FT-105
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_870_dashboard_never_writes_to_disk_during_request_handling
observes:
- disk-state
---

**observes:** [disk-state]

Take a SHA-256 of every file under the temp repo before boot. Boot the
server. Issue a representative request burst against every route
(GET on each path, plus a few `POST`s that return `405`). Take the
SHA-256 again after the burst. The two snapshots must be identical —
no file created, modified, deleted, or touched.

This is the load-bearing invariant for ADR-052: the dashboard is
read-only. Any drift indicates a handler reached into `fileops::*`
or acquired the write lock.

Surface:
- **disk-state:** repo file tree hashes are byte-equal pre/post.
