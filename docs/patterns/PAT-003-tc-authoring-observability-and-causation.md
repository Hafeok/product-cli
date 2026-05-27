---
id: PAT-003
title: 'TC authoring: observability and causation'
status: live
domains:
- testing
adrs:
- ADR-051
requires: []
examples:
- FT-066
- FT-072
---

## When to use

Every TC of type `scenario`, `session`, `smoke`, or `contract`
from phase 5 onward. These are the TC types that ADR-051 marks
required-for the `observes:` field. The discipline is also good
practice for invariant / property / chaos TCs even though they
are optional under the gate — naming the observed surface keeps
the assertion target unambiguous for both the author and the
reader.

## Prerequisites

None. The field is declared in front-matter; the body follows.

## The pattern

Declare `observes:` explicitly in front-matter and assert on the
named surface in the body. The surface is what is observable
*after* the action — a file on disk, a graph node, an exit code,
a captured stdout — not the return-type shape of the action
itself.

```yaml
---
id: TC-778
title: mcp_feature_status_writes_to_disk
type: scenario
observes: [file, mcp-response]
runner: cargo-test
runner-args: tc_778_mcp_feature_status_writes_to_disk
---
```

```rust
#[test]
fn tc_778_mcp_feature_status_writes_to_disk() {
    let s = Session::new();
    s.apply(/* request creating FT-X with status: planned */).assert_applied();

    // Action under test — MCP call advertising a write.
    let envelope = mcp_call("product_feature_status",
        json!({ "id": "FT-X", "status": "complete" }));

    // Observe the causation: read the file the response claims to have written.
    let path = s.dir.path().join("docs/features/FT-X-*.md");
    let body = std::fs::read_to_string(path).expect("read FT-X");
    assert!(body.contains("status: complete"),
        "MCP returned success but feature file is unchanged: {}", body);

    // The envelope is necessary evidence, not sufficient.
    assert_eq!(envelope["status"], "complete");
}
```

The file assertion is load-bearing. The envelope assertion is a
secondary check. Reversing the priority recreates the FT-046
failure mode.

## Anti-patterns

- **TC asserts on `Ok(_)` shape only.** A test that observes only
  that the function returned without error tells you nothing
  about whether the side effect happened. Replace with an
  assertion on the named surface.
- **TC observes only the MCP response envelope without inspecting
  the file the response claims to have written** (the FT-046
  shape). Every FT-046-class bug ships green tests of this shape.
  The fix is to read the file the action mutates and assert on
  its contents.
- **TC declares `observes: [file]` but its body never reads the
  file.** The structural gate (`product graph check`) catches
  missing-field cases; the body-reference check catches this
  case. A TC that names a surface and never inspects it is
  decorative.

## Worked example

FT-066's TC-778, TC-779, TC-787 are the post-fix TC family for
the MCP status-write and link-reciprocation paths. Each composes
a temp repo, invokes the MCP tool, and asserts on the on-disk
file. TC-778 is the cleanest reference for the
`observes: [file, mcp-response]` shape — it reads the feature
file's front-matter after the MCP call and asserts that
`status: complete` is present.

FT-072 is the feature that authored the contract structurally
— it added the `observes:` field to the TC parser, the graph
check error code, and the body-reference warning. Together with
FT-066 it forms the two-feature backbone of this pattern: FT-066
discovered the failure mode in oral tradition; FT-072 made the
fix structural.
