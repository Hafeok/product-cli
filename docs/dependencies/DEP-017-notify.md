---
id: DEP-017
title: notify
type: library
source: "https://crates.io/crates/notify"
version: "6"
status: active
features:
  - FT-033
adrs: []
availability-check: "cargo check"
breaking-change-risk: low
---

# notify

Cross-platform file system event watcher. Used in agent context generation for monitoring file system changes. Configured with `default-features = false` and `macos_kqueue` feature.
