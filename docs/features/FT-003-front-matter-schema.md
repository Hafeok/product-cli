---
id: FT-003
title: Front-Matter Schema
phase: 1
status: in-progress
depends-on: []
adrs:
- ADR-002
- ADR-014
- ADR-016
tests:
- TC-005
- TC-006
- TC-007
- TC-008
- TC-060
- TC-061
- TC-062
- TC-063
- TC-064
- TC-065
- TC-071
- TC-072
- TC-073
- TC-074
- TC-075
- TC-076
- TC-077
- TC-078
- TC-079
domains: []
domains-acknowledged: {}
---

### Feature

```yaml
---
id: FT-001
title: Cluster Foundation
phase: 1
status: in-progress          # planned | in-progress | complete | abandoned
depends-on: []               # feature IDs that must be complete before this one
domains: [consensus, networking, storage, iam, observability]
                             # concern domains this feature touches
adrs: [ADR-001, ADR-002, ADR-003, ADR-006]
tests: [TC-001, TC-002, TC-003, TC-004]
domains-acknowledged:        # explicit reasoning for domains with no linked ADR
  scheduling: >
    No workload scheduling in phase 1. Cluster foundation does not
    place containers — that is phase 2. Intentionally out of scope.
---
```

The `depends-on` field declares implementation dependencies between features. Product validates that these edges form a DAG — cycles are a hard error. `product feature next` uses topological sort over this DAG to determine the correct implementation order, replacing the previous phase-label ordering.

### ADR

```yaml
---
id: ADR-002
title: openraft for Cluster Consensus
status: accepted             # proposed | accepted | superseded | abandoned
features: [FT-001]
supersedes: []
superseded-by: []
domains: [consensus, networking]   # concern domains this ADR governs
scope: domain               # cross-cutting | domain | feature-specific (default)
source-files:                # optional: source files that implement this decision
  - src/consensus/raft.rs    # used by `product drift check` for precise analysis
  - src/consensus/leader.rs  # if absent, Product uses pattern-based discovery
---
```

### Test Criterion

Test criterion files use a hybrid format. The YAML front-matter carries graph metadata. The file body contains a prose description followed by optional AISP-influenced formal blocks (see ADR-011).

**Types and formal block requirements:**

| Type | Description | Formal blocks |
|---|---|---|
| `scenario` | Given/when/then integration test | Optional (`⟦Λ:Scenario⟧`) |
| `invariant` | Property that must hold for all valid inputs | Mandatory (`⟦Γ:Invariants⟧`) |
| `chaos` | System behaviour under fault injection | Mandatory (`⟦Γ:Invariants⟧`) |
| `exit-criteria` | Measurable threshold for phase completion | Optional (`⟦Λ:ExitCriteria⟧`) |
| `benchmark` | Quality measurement producing a score over time | Mandatory (`⟦Λ:Benchmark⟧`) |

The `benchmark` type is distinct from the others: it does not produce a binary pass/fail result. It produces a score in [0.0, 1.0] tracked over releases. A benchmark test criterion references an external task directory and rubric file rather than expressing an inline assertion.

**Scenario example:**
```markdown
---
id: TC-002
title: Raft Leader Election
type: scenario
status: unimplemented        # unimplemented | implemented | passing | failing
validates:
  features: [FT-001]
  adrs: [ADR-002]
phase: 1
runner: cargo-test           # cargo-test | bash | pytest | custom
                             # omit if test infrastructure not yet available
runner-args: ["--test", "raft_leader_election", "--", "--nocapture"]
runner-timeout: 60s          # optional, default 30s
---