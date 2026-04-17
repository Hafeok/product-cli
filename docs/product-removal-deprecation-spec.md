# Product Removal and Deprecation Specification

> Standalone reference for how Product tracks and verifies ADR-mandated removals
> and deprecations — in Product itself and in any subject codebase.
>
> New TC type: `absence`
> New ADR front-matter fields: `removes`, `deprecates`
> New codes: G009, W022, W023

---

## The Problem

ADRs frequently mandate removal or deprecation of something:

- "Replace AutoMapper with manual mapping" — AutoMapper NuGet dependency must go
- "Product request is the only write interface" — `product feature new` must fail
- "`source-files` in ADR front-matter is deprecated" — field must warn if present
- "Migrate from EF Core 6 to EF Core 8" — EF Core 6 packages must be absent

The current TC model only expresses positive assertions — things that should be true.
Removals and deprecations are negative assertions — things that should be absent,
unreachable, or inert. The graph has no way to express them. An ADR that says "X is
removed" makes an untested claim.

---

## The Design

### New TC type: `absence`

An absence TC asserts that something which should no longer exist is in fact gone,
or that something deprecated emits the correct warning when encountered.

Same front-matter structure as all other TCs. Same runner model. The difference is
semantic: the assertion is a negative one, and the runner must invert its expectation
— a command that should be gone should exit non-zero; a file that should be absent
should not exist; a package that should be removed should not appear in the dependency
list.

```yaml
---
id: TC-045
title: AutoMapper removed — no references in codebase
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-no-automapper.sh"]
runner-timeout: 30s
validates:
  adrs: [ADR-019]
  features: []    # cross-cutting — run via product verify --platform
---

ADR-019 mandates migration from AutoMapper to manual mapping.
This TC asserts that AutoMapper is no longer referenced anywhere
in the codebase — no .csproj references, no using statements,
no IMapper or CreateMap calls.
```

Absence TCs run via `product verify --platform` — they are cross-cutting assertions
that apply to the whole codebase, not to a specific feature. They use `validates.adrs`
to link to the governing decision and leave `validates.features` empty.

### New ADR fields: `removes` and `deprecates`

An ADR that mandates removal or deprecation declares it explicitly in front-matter:

```yaml
---
id: ADR-019
title: Replace AutoMapper with manual mapping
removes:
  - AutoMapper NuGet dependency
  - IMapper interface usage
  - CreateMap configuration calls
  - AutoMapperProfile classes
deprecates: []
features: [FT-005]
validates: [TC-045, TC-046]
---
```

```yaml
---
id: ADR-032
title: Product Request — The Single Write Interface
removes:
  - product feature new
  - product adr new
  - product test new
  - product dep new
  - product feature link
  - product feature acknowledge
  - product adr link
  - product adr amend
  - product test status
  - product feature status
deprecates:
  - source-files      # ADR front-matter field — deprecated in favour of git tags
features: [...]
validates: [TC-CQ-006, TC-CQ-007, TC-CQ-008, ...]
---
```

The `removes` and `deprecates` fields are freeform strings — they describe what is
being removed in human-readable form. Product does not parse or interpret them. Their
purpose is to:

1. Make the ADR self-documenting about what it eliminates
2. Enable G009 (structural gap check: removal declared but no absence TC linked)
3. Enable W022 (graph check: same condition, W-class)

### G009 — Removal declared with no absence TC

`product gap check` (structural) gains a new check:

**G009** — ADR has `removes` or `deprecates` entries but no linked TC of type
`absence`. Every declared removal must have an enforcing TC.

| Code | Tier | Severity | Condition |
|---|---|---|---|
| G009 | Gap | high | ADR has `removes` or `deprecates` entries with no linked `absence` TC |

G009 is structural — no LLM required. Product checks: does this ADR have non-empty
`removes` or `deprecates` AND at least one linked TC with `type: absence`?

### W022 — Same check as a graph warning

`product graph check` emits W022 for the same condition. G009 is the gap analysis
code (runs on demand or on `--changed` ADRs); W022 is the structural check (runs
on every `product graph check`).

| Code | Tier | Condition |
|---|---|---|
| W022 | Validation | ADR declares `removes` or `deprecates` with no linked absence TC |

### W023 — Deprecated field encountered

When Product reads a front-matter field that is declared deprecated (via an accepted
ADR's `deprecates` list), it emits W023:

```
warning[W023]: deprecated field 'source-files' in ADR-002
  This field was deprecated by ADR-023 (git tag-based drift detection).
  Run: product request change to remove it.
  See: product adr show ADR-023
```

W023 fires during `product graph check`. The deprecated field is still read and
processed (for backward compatibility) — W023 is a reminder, not a blocker.

---

## Runner Patterns

The runner model is language-agnostic. The absence TC runner is always a shell
command that exits 0 if the assertion passes (thing is absent) and non-zero if it
fails (thing is still present).

### Pattern 1: Removed CLI command

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-command-removed.sh COMMAND [ARGS...]
# Asserts that 'product COMMAND' exits non-zero (is removed).

COMMAND="$1"
shift

OUTPUT=$(product "$COMMAND" "$@" 2>&1)
EXIT=$?

if [ "$EXIT" -eq 0 ]; then
  echo "FAIL: 'product $COMMAND' succeeded — should be removed"
  echo "Output: $OUTPUT"
  exit 1
fi

# Optional: assert the error message mentions the replacement
if [ -n "$ASSERT_MESSAGE" ]; then
  if ! echo "$OUTPUT" | grep -q "$ASSERT_MESSAGE"; then
    echo "FAIL: removed command did not print expected message: $ASSERT_MESSAGE"
    echo "Got: $OUTPUT"
    exit 1
  fi
fi

echo "PASS: 'product $COMMAND' correctly rejected (exit $EXIT)"
exit 0
```

Usage in TC front-matter:
```yaml
runner: bash
runner-args: ["scripts/test-harness/assert-command-removed.sh", "feature", "new"]
```

With message assertion:
```yaml
runner: bash
runner-args: ["scripts/test-harness/assert-command-removed.sh", "feature", "new"]
runner-env:
  ASSERT_MESSAGE: "product request create"
```

### Pattern 2: NuGet package removed (C# / .NET)

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-no-nuget.sh PACKAGE_NAME
# Asserts that a NuGet package is not referenced in any project.

PACKAGE="$1"
VIOLATIONS=0

# Check .csproj files
while IFS= read -r file; do
  if grep -qi "$PACKAGE" "$file"; then
    echo "FAIL: $file still references $PACKAGE"
    grep -i "$PACKAGE" "$file"
    VIOLATIONS=$((VIOLATIONS + 1))
  fi
done < <(find . -name "*.csproj" -not -path "*/bin/*" -not -path "*/obj/*")

# Check packages.props or Directory.Packages.props
for props in Directory.Packages.props Directory.Build.props; do
  if [ -f "$props" ] && grep -qi "$PACKAGE" "$props"; then
    echo "FAIL: $props still references $PACKAGE"
    grep -i "$PACKAGE" "$props"
    VIOLATIONS=$((VIOLATIONS + 1))
  fi
done

if [ "$VIOLATIONS" -gt 0 ]; then
  echo "FAIL: $PACKAGE found in $VIOLATIONS location(s)"
  exit 1
fi

echo "PASS: $PACKAGE not referenced in any project"
exit 0
```

Usage:
```yaml
runner: bash
runner-args: ["scripts/test-harness/assert-no-nuget.sh", "AutoMapper"]
```

### Pattern 3: C# API usage removed

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-no-cs-pattern.sh PATTERN DESCRIPTION
# Asserts that a C# code pattern does not appear in the source.

PATTERN="$1"
DESCRIPTION="${2:-$PATTERN}"
VIOLATIONS=0

while IFS= read -r file; do
  if grep -qP "$PATTERN" "$file" 2>/dev/null || grep -q "$PATTERN" "$file" 2>/dev/null; then
    echo "FAIL: $file contains $DESCRIPTION"
    grep -n "$PATTERN" "$file" | head -5
    VIOLATIONS=$((VIOLATIONS + 1))
  fi
done < <(find src -name "*.cs" -not -path "*/bin/*" -not -path "*/obj/*")

if [ "$VIOLATIONS" -gt 0 ]; then
  echo "FAIL: $DESCRIPTION found in $VIOLATIONS file(s)"
  exit 1
fi

echo "PASS: $DESCRIPTION not found in source"
exit 0
```

Usage (multiple patterns for AutoMapper):
```bash
# scripts/test-harness/assert-no-automapper.sh
set -euo pipefail

bash scripts/test-harness/assert-no-nuget.sh "AutoMapper" || exit 1
bash scripts/test-harness/assert-no-cs-pattern.sh "using AutoMapper" "AutoMapper using statement" || exit 1
bash scripts/test-harness/assert-no-cs-pattern.sh "IMapper" "IMapper interface" || exit 1
bash scripts/test-harness/assert-no-cs-pattern.sh "\.CreateMap<" "CreateMap call" || exit 1
bash scripts/test-harness/assert-no-cs-pattern.sh "MapperConfiguration" "MapperConfiguration" || exit 1

echo "PASS: AutoMapper completely removed"
exit 0
```

### Pattern 4: npm/yarn package removed (JavaScript/TypeScript)

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-no-npm.sh PACKAGE_NAME
PACKAGE="$1"

if node -e "require('./package.json').dependencies['$PACKAGE']" 2>/dev/null; then
  echo "FAIL: $PACKAGE in dependencies"
  exit 1
fi

if node -e "require('./package.json').devDependencies['$PACKAGE']" 2>/dev/null; then
  echo "FAIL: $PACKAGE in devDependencies"
  exit 1
fi

if grep -r "from '$PACKAGE'" src/ 2>/dev/null | grep -q .; then
  echo "FAIL: $PACKAGE import found in source"
  grep -r "from '$PACKAGE'" src/ | head -5
  exit 1
fi

echo "PASS: $PACKAGE not referenced"
exit 0
```

### Pattern 5: Cargo crate removed (Rust)

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-no-crate.sh CRATE_NAME
CRATE="$1"

if grep -q "^$CRATE\b\|\"$CRATE\"\|'$CRATE'" Cargo.toml 2>/dev/null; then
  echo "FAIL: $CRATE in Cargo.toml"
  grep "$CRATE" Cargo.toml
  exit 1
fi

if grep -rq "extern crate $CRATE\|use $CRATE::" src/ 2>/dev/null; then
  echo "FAIL: $CRATE usage found in source"
  grep -rn "extern crate $CRATE\|use $CRATE::" src/ | head -5
  exit 1
fi

echo "PASS: $CRATE not referenced"
exit 0
```

### Pattern 6: File or directory absent

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-file-absent.sh PATH [PATH...]
FAILED=0

for path in "$@"; do
  if [ -e "$path" ]; then
    echo "FAIL: $path exists but should be absent"
    FAILED=$((FAILED + 1))
  fi
done

if [ "$FAILED" -gt 0 ]; then
  exit 1
fi

echo "PASS: all specified paths absent"
exit 0
```

### Pattern 7: Deprecation warning emitted (in-migration state)

For the case where something is deprecated but still present during migration,
the TC asserts the deprecation signal is working — not that the thing is gone:

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-deprecation-warning.sh
# Asserts that AutoMapper usage triggers a compiler warning.
# Use during migration period; replace with assert-no-automapper.sh when done.

BUILD_OUTPUT=$(dotnet build src/ 2>&1)
BUILD_EXIT=$?

# If it doesn't build at all, something is wrong
if [ "$BUILD_EXIT" -ne 0 ] && ! echo "$BUILD_OUTPUT" | grep -q "warning"; then
  echo "FAIL: Build failed without deprecation warning"
  echo "$BUILD_OUTPUT" | tail -20
  exit 1
fi

# AutoMapper references must produce [Obsolete] warnings
if echo "$BUILD_OUTPUT" | grep -qi "AutoMapper"; then
  if echo "$BUILD_OUTPUT" | grep -qi "obsolete\|deprecated\|CS0618"; then
    echo "PASS: AutoMapper usage produces expected obsolete warning"
    exit 0
  else
    echo "FAIL: AutoMapper referenced without obsolete warning"
    echo "Add [Obsolete] to AutoMapper wrapper or update to tagged obsolete"
    exit 1
  fi
fi

# AutoMapper not referenced at all — also a pass for this TC
echo "PASS: AutoMapper not present (migration complete)"
exit 0
```

---

## Complete Example: AutoMapper Migration in a C# Project

### ADR-019 — Replace AutoMapper with manual mapping

```yaml
---
id: ADR-019
title: Replace AutoMapper with manual mapping
status: accepted
domains: [data-model, api]
scope: domain
features: [FT-005, FT-006, FT-007]
removes:
  - AutoMapper NuGet package (AutoMapper, AutoMapper.Extensions.Microsoft.DependencyInjection)
  - IMapper interface usage
  - CreateMap and MapperConfiguration calls
  - AutoMapperProfile classes
  - MapFrom and Ignore configuration
deprecates:
  - MappingExtensions.cs helper class (replaced by explicit projection methods)
validates: [TC-045, TC-046, TC-047]
---

## Context

AutoMapper is used throughout the codebase for object-to-object mapping between
domain models and DTOs. This creates hidden coupling, makes mapping logic hard to
debug, and produces runtime errors for unmapped properties that should be compile-time
errors.

## Decision

Replace all AutoMapper usage with explicit manual mapping methods. Each mapping is
a static method or constructor that is fully visible, refactorable, and testable.

## Rationale
...

## Rejected alternatives
...

## Test coverage

TC-045 verifies AutoMapper is fully removed.
TC-046 verifies MappingExtensions.cs deprecation warning is present during migration.
TC-047 verifies that MappingExtensions.cs is eventually removed.
```

### TC-045 — AutoMapper removed

```yaml
---
id: TC-045
title: AutoMapper completely removed from codebase
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-no-automapper.sh"]
runner-timeout: 60s
validates:
  adrs: [ADR-019]
  features: []
---

Asserts that AutoMapper is fully removed:
- No AutoMapper NuGet reference in any .csproj
- No `using AutoMapper` statements
- No IMapper interface usage
- No CreateMap or MapperConfiguration calls

This TC replaces TC-046 once the migration is complete.
```

### TC-046 — MappingExtensions.cs emits deprecation warning (migration state)

```yaml
---
id: TC-046
title: MappingExtensions.cs usage produces CS0618 obsolete warning
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-deprecation-warning.sh"]
runner-timeout: 120s
validates:
  adrs: [ADR-019]
  features: []
---

During the migration period, MappingExtensions.cs is still present but
decorated with [Obsolete]. This TC asserts that any usage produces the
expected compiler warning (CS0618).

Mark this TC as `unrunnable` once TC-047 is passing.
```

### TC-047 — MappingExtensions.cs removed

```yaml
---
id: TC-047
title: MappingExtensions.cs removed
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-file-absent.sh",
              "src/Common/MappingExtensions.cs",
              "src/Infrastructure/Mapping/MappingExtensions.cs"]
runner-timeout: 10s
validates:
  adrs: [ADR-019]
  features: []
---

Asserts that MappingExtensions.cs no longer exists anywhere in src/.
This TC is the completion signal for the AutoMapper migration.
```

---

## Complete Example: Product Own Commands Removed

### From ADR-032 — Product Request as Single Write Interface

The ADR's `removes:` list drives the absence TCs. One TC per removed command group:

```yaml
---
id: TC-CQ-006
title: Individual create commands removed — product feature new, adr new, test new, dep new
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-commands-removed.sh",
              "feature new", "adr new", "test new", "dep new"]
runner-timeout: 10s
validates:
  adrs: [ADR-032]
  features: []
---
```

```yaml
---
id: TC-CQ-007
title: Individual link commands removed — product feature link, adr link, feature acknowledge
type: absence
status: unimplemented
runner: bash
runner-args: ["scripts/test-harness/assert-commands-removed.sh",
              "feature link", "adr link", "feature acknowledge"]
runner-timeout: 10s
validates:
  adrs: [ADR-032]
  features: []
---
```

```bash
#!/usr/bin/env bash
# scripts/test-harness/assert-commands-removed.sh CMD1 CMD2 ...
# Asserts multiple product subcommands are removed.
FAILED=0

while [ $# -gt 0 ]; do
  CMD="$1 $2"
  shift 2

  OUTPUT=$(product $CMD 2>&1)
  EXIT=$?

  if [ "$EXIT" -eq 0 ]; then
    echo "FAIL: 'product $CMD' succeeded — should be removed"
    FAILED=$((FAILED + 1))
  elif echo "$OUTPUT" | grep -qi "product request"; then
    echo "PASS: 'product $CMD' rejected with correct hint"
  else
    echo "WARN: 'product $CMD' rejected but missing 'product request' hint"
    echo "  Output: $OUTPUT"
  fi
done

[ "$FAILED" -eq 0 ]
```

---

## Migration Lifecycle for an Absence TC

An absence TC typically passes through this lifecycle:

```
unimplemented   → the removal is mandated but not yet implemented
                  (G009/W022 fire — this is expected during development)

unrunnable      → runner infrastructure not available for CI yet
                  (explicitly acknowledged with a reason)

failing         → the removal was attempted but something still exists
                  (TC runs, finds the thing still present, exits non-zero)

passing         → the removal is complete and enforced
                  (TC runs, confirms absence, exits 0)
```

For deprecation TCs with two phases (deprecated-but-present → removed):

```
Phase 1 (migration in progress):
  TC-046  passing     ← deprecation warning present ✓
  TC-047  failing     ← file still exists ✗

Phase 2 (migration complete):
  TC-046  unrunnable  ← acknowledged: "superseded by TC-047"
  TC-047  passing     ← file absent ✓
```

---

## New Validation Codes

### Gap codes

| Code | Severity | Condition |
|---|---|---|
| G009 | high | ADR has `removes` or `deprecates` entries but no linked TC of `type: absence` |

G009 is structural — computed from front-matter without LLM. Caught by
`product gap check` (structural checks) and `product graph check` (W022).

### Warning codes

| Code | Condition |
|---|---|
| W022 | ADR has `removes` or `deprecates` entries with no linked absence TC |
| W023 | Deprecated front-matter field encountered during graph construction |

W022 fires during `product graph check` — same condition as G009 but as a warning
in the structural check stream. G009 is the gap analysis code; W022 is the graph
check code. Both exist because gap check and graph check serve different audiences
(spec quality vs structural validity) and run at different times.

W023 fires when Product reads a field that appears in an accepted ADR's `deprecates`
list. The field is still processed for backward compatibility. W023 is the prompt
to migrate.

---

## Session Tests

```
# Absence TC basics
ST-140  absence-tc-passes-when-thing-gone
ST-141  absence-tc-fails-when-thing-present
ST-142  absence-tc-runs-in-platform-verify

# ADR removes/deprecates field
ST-143  adr-removes-field-parses-correctly
ST-144  adr-deprecates-field-parses-correctly
ST-145  g009-fires-when-removes-no-absence-tc
ST-146  w022-fires-same-condition
ST-147  g009-clear-when-absence-tc-linked

# W023 deprecated field
ST-148  w023-fires-on-deprecated-field
ST-149  deprecated-field-still-processed-for-compat
ST-150  w023-names-deprecating-adr

# Migration lifecycle
ST-151  migration-phase1-deprecation-tc-passes
ST-152  migration-phase2-absence-tc-passes
ST-153  migration-phase2-phase1-tc-unrunnable-no-block
```

---

## Invariants

- Absence TCs are never linked to specific features in `validates.features` — they
  are cross-cutting assertions that validate ADR mandates, not feature behaviours.
- `product verify --platform` runs all absence TCs alongside code quality TCs.
- A failing absence TC is exit code 1 — same severity as a failing scenario TC.
  An ADR-mandated removal that is not enforced is a structural violation.
- The `removes` and `deprecates` fields on ADRs are freeform strings describing
  what is removed. Product never attempts to parse or execute them. They exist
  to drive G009/W022 detection and to make the ADR self-documenting.
- W023 never blocks — a deprecated field is always processed for backward
  compatibility. The warning exists to prompt migration, not to break existing repos.
