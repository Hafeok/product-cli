---
id: TC-830
title: tc_observes_field_parses_as_flat_list
type: scenario
status: passing
validates:
  features:
  - FT-072
  adrs: []
phase: 1
runner: cargo-test
runner-args: tc_830_tc_observes_field_parses_as_flat_list
observes:
- file
- graph
last-run: 2026-05-27T14:11:07.133454142+00:00
last-run-duration: 0.4s
---

## Description

Compose a temp repo. Write a TC YAML file with `observes: [file,
graph]` in the front-matter. Load it via the parser.

Assert:

1. The parsed `TcFrontMatter` exposes `observes:
   ["file", "graph"]` as a `Vec<String>`.
2. Round-tripping the parsed value back through the writer
   produces a file whose `observes:` field is preserved byte-for-
   byte after a `serialise → write → read → parse` cycle.
3. An empty list (`observes: []`) parses as an empty vector
   without error.
4. A missing `observes:` key parses as a default empty vector
   (no error).

## Formal specification

⟦Λ:Scenario⟧
Given a TC file with `observes: [file, graph]` in front-matter,
When the parser loads the file,
Then the `TcFrontMatter.observes` field equals `["file",
  "graph"]`,
And serialising and reparsing yields the same value,
And missing or empty observes parses to an empty vector
  without error.

⟦Ε⟧⟨δ≜1.0;φ≜1;τ≜◊⁺⟩