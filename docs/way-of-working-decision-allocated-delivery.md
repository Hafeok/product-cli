# Decision-Allocated Delivery

**Context& way of working — .NET/C# + Bicep pipelines to Azure.**

Status: **projected.** Clean derivation from the conservation principle; not yet exercised end-to-end by a running team. Every falsifier is stated inline. Nothing below is reported.

**Relationship to canon and tooling.** The framework repo (`decision-driven-design`) is ground truth for the principle; where this document and canon disagree, canon wins. The `ddd` tool (PRD: `docs/ddd-cli-prd.md`) is this process's governance mechanism, and the **Decision Ledger** (PRD: `decision-ledger-prd.md`) is the record substrate: the ledger validator referenced below is `ledger verify` (bootstrapped by `ddd validate` until ledger integration at ddd M8), escape reporting is `ddd report escapes`, and the flat `decisions.yml` is the ledger's bootstrap form. The escape checklists in §6 are the seed content for the predicate catalog (PRD M5). Adoption order §10 steps 1–3 require no tooling and are the engagement entry point.

---

## 0. The one-page premise

Specification demand for a task at a declared assurance level is fixed by the task, the tolerance, and the ground distribution it faces. It does not shrink because transcription got cheap. It is fully allocated across four stores:

| Store | Form | When | Who |
|---|---|---|---|
| **Encoded** | constraint | before the act | extra-actor (compiler, analyzer, policy) |
| **Mechanical verification** | criterion | after the act | extra-actor (test, what-if, assertion) |
| **Judgment** | per-run | during the act | a named accountable actor |
| **Escaped** | — | — | **nobody** |

Escaped is the only forbidden state. Everything in this document exists to make escape *visible and priced* rather than silent.

Two consequences drive the whole design:

1. **Coverage, not pass-rate.** A green board over an unenumerated set means nothing. The denominator is the governing decision set, not lines of code.
2. **Deliberate escape and accidental escape produce identical artifacts.** They diverge only when something goes wrong — one is a decision you can revisit, the other is archaeology. The ledger is what distinguishes them.

---

## 1. Declare (two independent fields)

Before anything else, two declarations. They are orthogonal; collapsing them is the most common error.

### 1.1 Tolerance

The assurance level. Sets the **granularity bound**: a decision is in the governing set iff varying it moves the outcome past tolerance.

| Tier | Meaning | Enumeration | Discharge |
|---|---|---|---|
| `T0` | Consequence trivial, lifetime short | Coarse (5–15 decisions) | Mostly priced escape |
| `T1` | Normal product surface | Moderate (20–60) | Mixed |
| `T2` | Money, identity, data loss, regulatory | Fine | Encoded + mechanical, escape by exception only |

**Cheap work skips discharge, never enumeration.** The ledger is nearly free; the discharge procedures are what cost. A `T0` feature with an enumerated set and priced escapes is legitimate. A `T0` feature with no set is not cheap — it is unmeasured.

**Falsifier:** if late-discovery on `T0` work routinely lands outside its priced band, the tolerance was mis-declared, not the enumeration.

### 1.2 Ground state

Is the ground characterised or not?

- **Uncharacterised** → recon branch. Ship thin into production to buy facts. **The deliverable is discovery records, not the artifact.** Terminates when ground is characterised.
- **Characterised** → the main loop.

**Mechanical check:** if the recon branch is shipping durable production code, it was never recon. It was a price decision wearing an epistemic label.

---

## 2. The loop

```
DECLARE ──▶ [recon if ground uncharacterised] ──▶
  1 Enumerate ─▶ 2 Allocate ─▶ 3 Form ─▶ 4 Produce
      ▲                                       │
      │                                       ▼
  9 Convert ◀─ 8 Reconcile ◀─ 7 Accept ◀─ 6 Discharge ◀─ 5 Detect
```

### 1 — Enumerate

Governing decisions at the declared bound. Model proposes; a named actor owns the set.

### 2 — Allocate

Every decision gets exactly one primary allocation (redundancy permitted, uncovered is not):

- `constraint` — encoded before the act (nullable, analyzer, Azure Policy, type)
- `criterion` — mechanically checked after (test, what-if, arch test, KQL assertion)
- `judgment` — deferred to a named actor per run
- `escaped` — priced, accepted by name, with a stated exposure

**Gate: readiness.** No unallocated decisions. Blocks step 4. This is the single highest-leverage change to a normal pipeline, because it moves the first gate *before* transcription — out of the load-pressured moment where capacity-bound actors shed decisions to their prior.

### 3 — Form

For every `criterion`, author the discharge procedure alongside the claim: runner, ground, environment, what counts as evidence. **Separate the assertion from the harness** so a red result is attributable — otherwise you cannot distinguish "the code is wrong" from "the procedure is broken."

### 4 — Produce

Transcription. Cheap. One rule: **no scaffolding justified only by a weaker actor.** Decompose where the task has joints, not where a previous model needed help. Every seam carries demand — the chain rule splits the total into what the split encodes and what the parts must still resolve, `H(V) = I(V;S) + H(V|S)`; a seam that encodes nothing about the verdict is pure interface cost.

### 5 — Detect

Review as **escape detection against the allocation table**, not line-reading. See §6.

### 6 — Discharge

Run procedures. Environment promotion belongs here, scheduled by **which ground facts each stage exposes** (§5.1). Discharging a decision later than its ground allowed is visible waste.

### 7 — Accept

A **named human**, per decision cluster, timestamped, expiring. Not the runner. Not a model. This is the ledger entry that compounds.

Acceptance may be **precommitted at class level** — that is exactly what an analyzer or an Azure Policy assignment is. Precommitment is how human load falls without accountability leaving; delegation to a model is not.

### 8 — Reconcile

Coverage over the enumerated set. Gaps reported loudly.

### 9 — Convert

Escapes that proved governing get bought back into encoded form — with provenance, supersession, and expiry, in the ledger. **Not appended to a prompt file.**

> **Known overlap.** Steps 5 and 8 overlap: detection finds escapes before discharge, reconcile checks coverage after. They are separate only because enumeration is imperfect. Their combined cost should fall as late-discovery falls. If it does not, one of them has become ceremony.

---

## 3. The ledger artifact

One file per feature, checked in beside the code: `decisions.yml`.

```yaml
feature: FT-104-invoice-export
tolerance: T2
ground: characterised
enumerated-by: claude-opus-5 / 2026-08-01
owned-by: emil@example.com

decisions:
  - id: DEC-001
    statement: Monetary amounts use decimal, never double, across the export path.
    allocation: constraint
    discharge: analyzer:DEC001-no-float-money
    accepted-by: emil@example.com
    accepted-at: 2026-08-01
    expires: 2027-08-01

  - id: DEC-002
    statement: Export is idempotent per (invoiceId, revision); a retry never double-writes.
    allocation: criterion
    discharge: test:ExportIdempotencyTests
    discharge-stage: pr
    accepted-by: emil@example.com
    accepted-at: 2026-08-01

  - id: DEC-003
    statement: Storage account denies public blob access.
    allocation: constraint
    discharge: policy:deny-public-blob   # platform-enforced, extra-actor
    accepted-by: platform-team
    accepted-at: 2026-06-14

  - id: DEC-004
    statement: Failed exports retry 3x with exponential backoff, then dead-letter.
    allocation: criterion
    discharge: test:RetryPolicyTests + otel:dec.004.deadletter
    discharge-stage: prod
    expectation: "dead-letter rate < 0.5% of exports over 14d"
    accepted-by: emil@example.com

  - id: DEC-005
    statement: Export column order matches the legacy CSV contract.
    allocation: escaped
    exposure: "Downstream consumer may break silently. Manual verification at first customer onboarding."
    accepted-by: emil@example.com
    accepted-at: 2026-08-01
    review-by: 2026-09-01

  - id: DEC-006
    statement: Which customers see the new export format first.
    allocation: judgment
    actor: product-owner
```

Schema constraints worth enforcing in CI (`ddd validate` owns these; a small standalone validator suffices until it ships):

- Every decision has exactly one `allocation`.
- `escaped` requires `exposure`, `accepted-by`, `review-by`.
- `criterion` requires `discharge` and `discharge-stage`.
- `judgment` requires `actor`.
- `accepted-by` is never a model identity.
- Expired acceptances fail the build.

---

## 4. C# / .NET setup

The point of this section: **most of what a reviewer currently hunts for is pinnable by value.** A classical actor costs nothing per run and cannot be skipped. Renting a model to find `DateTime.Now` is paying binding-resolution prices for a decision a program can pin.

### 4.1 Turn the mechanical store on

`Directory.Build.props` at repo root:

```xml
<Project>
  <PropertyGroup>
    <Nullable>enable</Nullable>
    <TreatWarningsAsErrors>true</TreatWarningsAsErrors>
    <EnforceCodeStyleInBuild>true</EnforceCodeStyleInBuild>
    <AnalysisLevel>latest-recommended</AnalysisLevel>
    <AnalysisMode>All</AnalysisMode>
    <EnableNETAnalyzers>true</EnableNETAnalyzers>
    <GenerateDocumentationFile>true</GenerateDocumentationFile>
    <WarningsNotAsErrors></WarningsNotAsErrors>
  </PropertyGroup>
</Project>
```

`.editorconfig` — this **is** an encoded store: versioned, diffable, reviewable. Most teams keep it at `suggestion`, which constrains nothing.

```ini
[*.cs]
# Case analysis becomes mechanically complete
dotnet_diagnostic.CS8509.severity = error   # non-exhaustive switch expression
dotnet_diagnostic.CS8524.severity = error

# Culture and comparison — classic silent escapes
dotnet_diagnostic.CA1305.severity = error   # specify IFormatProvider
dotnet_diagnostic.CA1307.severity = error   # specify StringComparison for clarity
dotnet_diagnostic.CA1310.severity = error   # specify StringComparison for correctness
dotnet_diagnostic.CA1309.severity = error   # use ordinal comparison

# Async shape
dotnet_diagnostic.CA2007.severity = error   # ConfigureAwait (libraries)
dotnet_diagnostic.CA2016.severity = error   # forward CancellationToken
dotnet_diagnostic.VSTHRD100.severity = error # avoid async void (Threading.Analyzers)

# Boundary / serialization
dotnet_diagnostic.CA1863.severity = error   # CompositeFormat
dotnet_diagnostic.CA2326.severity = error   # unsafe JSON TypeNameHandling
```

Additional analyzer packages worth adding, each of which converts a review question into a compile error:

- `Microsoft.VisualStudio.Threading.Analyzers` — async-void, sync-over-async, fire-and-forget
- `Meziantou.Analyzer` — culture, equality, `DateTime.Now`, LINQ ordering assumptions
- `SonarAnalyzer.CSharp` — broad correctness net
- `Roslynator.Analyzers`

### 4.2 The two acceptance ledgers .NET already has

These are the closest existing artifacts to what §3 describes. Generalise them; do not reinvent.

**Public API surface** — `Microsoft.CodeAnalysis.PublicApiAnalyzers`:

```
PublicAPI.Shipped.txt
PublicAPI.Unshipped.txt
```

Any public surface change requires an explicit checked-in edit. `git blame` gives you the accepting actor and timestamp for free. This is attributed acceptance of an API decision, already working, in most teams' repos, unused as a ledger.

**Output snapshots** — `Verify` (or ApprovalTests):

```csharp
[Fact]
public Task ExportFormat_MatchesAcceptedShape()
    => Verify(exporter.Render(sampleInvoice));
```

`.approved.txt` in the repo. Same shape: a human accepted this output, recorded, diffable, attributed. Apply it to serialization shapes, generated SQL, ARM output, and public contracts.

### 4.3 Architecture decisions

`NetArchTest.Rules` or `ArchUnitNET` — for decisions no analyzer can reach:

```csharp
[Fact]
public void Domain_DoesNotDependOnInfrastructure()
{
    var result = Types.InAssembly(typeof(Invoice).Assembly)
        .That().ResideInNamespace("Acme.Domain")
        .ShouldNot().HaveDependencyOn("Acme.Infrastructure")
        .GetResult();

    Assert.True(result.IsSuccessful, string.Join(", ", result.FailingTypeNames ?? []));
}
```

### 4.4 The custom analyzer that closes the loop

The built-in rules cover generic escapes. The one worth writing yourself enforces the ledger itself: **every escape site must cite a decision id.**

Target sites: `catch (Exception)`, `!` (null-forgiving), `#pragma warning disable`, `// TODO`.

```csharp
[DiagnosticAnalyzer(LanguageNames.CSharp)]
public sealed class EscapeSiteMustCiteDecisionAnalyzer : DiagnosticAnalyzer
{
    private static readonly DiagnosticDescriptor Rule = new(
        id: "DEC001",
        title: "Escape site must cite a decision id",
        messageFormat: "This site defers a governing decision; annotate with // DEC-nnn from decisions.yml",
        category: "Allocation",
        defaultSeverity: DiagnosticSeverity.Error,
        isEnabledByDefault: true);

    public override ImmutableArray<DiagnosticDescriptor> SupportedDiagnostics => [Rule];

    public override void Initialize(AnalysisContext context)
    {
        context.EnableConcurrentExecution();
        context.ConfigureGeneratedCodeAnalysis(GeneratedCodeAnalysisFlags.None);
        context.RegisterSyntaxNodeAction(AnalyzeCatch, SyntaxKind.CatchClause);
        context.RegisterSyntaxNodeAction(AnalyzeSuppress, SyntaxKind.SuppressNullableWarningExpression);
    }

    private static void AnalyzeCatch(SyntaxNodeAnalysisContext ctx)
    {
        var clause = (CatchClauseSyntax)ctx.Node;
        var caught = clause.Declaration?.Type;
        if (caught is null) { Report(ctx, clause.GetLocation(), clause); return; }

        var symbol = ctx.SemanticModel.GetTypeInfo(caught).Type;
        if (symbol?.ToDisplayString() == "System.Exception")
            Report(ctx, clause.GetLocation(), clause);
    }

    private static void AnalyzeSuppress(SyntaxNodeAnalysisContext ctx)
        => Report(ctx, ctx.Node.GetLocation(), ctx.Node);

    private static void Report(SyntaxNodeAnalysisContext ctx, Location loc, SyntaxNode node)
    {
        if (HasDecisionCitation(node)) return;
        ctx.ReportDiagnostic(Diagnostic.Create(Rule, loc));
    }

    private static bool HasDecisionCitation(SyntaxNode node)
    {
        var trivia = node.GetLeadingTrivia()
            .Concat(node.Parent?.GetLeadingTrivia() ?? default)
            .Where(t => t.IsKind(SyntaxKind.SingleLineCommentTrivia));

        return trivia.Any(t => Regex.IsMatch(t.ToString(), @"DEC-\d{3}"));
    }
}
```

Pair it with a build task that validates every cited `DEC-nnn` exists in `decisions.yml` and is not expired. Now an escape cannot be silent, and a stale acceptance cannot survive a build.

**Rule of practice:** every escape class caught twice in review becomes an analyzer. That ratio — escapes caught by analyzer vs. caught by human — is your descent measure. If it is not moving, the loop is not running.

---

## 5. Bicep / Azure setup

Bicep is a better allocation surface than most teams use it as. Three distinct stores are available.

### 5.1 Environment ground table

Declare, once, what facts each stage exposes. This turns promotion from ritual into a discharge schedule.

| Stage | Ground newly available | Discharge appropriate here |
|---|---|---|
| **PR** | Static only: types, structure, template validity | Analyzers, unit tests, arch tests, `bicep lint`, PSRule, `what-if` against ephemeral RG |
| **Dev** | Real Azure resource behaviour, identity, RBAC, service quirks | Integration tests, deployment smoke, managed-identity auth |
| **Staging** | Prod-like data volume and shape, migration behaviour | Perf assertions, migration rehearsal against restored prod snapshot, load |
| **Prod** | Users, cohorts, real traffic, conversion, failure distribution | OTel-based criteria (§5.5) |

A decision discharged **later** than its ground allowed is waste, and now visible. A decision discharged **earlier** than its ground allows is fiction.

### 5.2 Bicep linter as encoded constraint

`bicepconfig.json` — set to error, not warning:

```json
{
  "analyzers": {
    "core": {
      "enabled": true,
      "rules": {
        "no-hardcoded-env-urls":     { "level": "error" },
        "secure-parameter-default":  { "level": "error" },
        "no-unnecessary-dependson":  { "level": "error" },
        "use-recent-api-versions":   { "level": "error", "maxAgeInDays": 730 },
        "no-unused-params":          { "level": "error" },
        "no-unused-vars":            { "level": "error" },
        "outputs-should-not-contain-secrets": { "level": "error" },
        "use-secure-value-for-secure-inputs": { "level": "error" }
      }
    }
  }
}
```

### 5.3 `what-if` is criterion form

`az deployment group what-if` is the mechanical verification store for infrastructure: it states the outcome before the act, mechanically, from the real target's current state.

```bash
az deployment group what-if \
  --resource-group $RG \
  --template-file ./infra/main.bicep \
  --parameters ./infra/params.$ENV.bicepparam \
  --result-format FullResourcePayload \
  --no-pretty-print > whatif.json
```

Then assert over `whatif.json` structurally — **not with regex**. The changes array is the artifact; query it:

```bash
# fail the stage on any unexpected Delete
jq -e '[.changes[] | select(.changeType == "Delete")] | length == 0' whatif.json
```

For richer assertions, `az bicep build` to ARM JSON and query the object model. Regex over Bicep source is the wrong evaluator: the source is not the deployed shape, and the compiled template is a structured artifact that answers the question directly.

### 5.4 Policy is extra-actor constraint

Azure Policy with `deny` effects is the strongest allocation available for infrastructure: enforced by the platform, before the act, outside the deploying actor entirely. A decision pinned here cannot be escaped by any pipeline, any human, or any agent.

Move these out of Bicep and into Policy wherever possible:

- Public network access, TLS minimums, allowed SKUs, allowed regions
- Required tags (including a `decisionSet` tag pointing at the owning `decisions.yml`)
- Encryption, diagnostic settings, private endpoint requirements

Manage assignments as code (EPAC or plain Bicep at management-group scope) so the policy set itself carries provenance.

**Deployment stacks** add a second extra-actor constraint — deletion protection independent of the template:

```bash
az stack group create \
  --name invoice-export \
  --resource-group $RG \
  --template-file ./infra/main.bicep \
  --deny-settings-mode denyDelete \
  --action-on-unmanage detachAll
```

### 5.5 PSRule for Azure

Criterion-form checks over the compiled template, with baselines pinned per tolerance tier:

```yaml
# ps-rule.yaml
include:
  module: [ PSRule.Rules.Azure ]
configuration:
  AZURE_BICEP_FILE_EXPANSION: true
  AZURE_BICEP_CHECK_TOOL: true
rule:
  includeLocal: true
```

```powershell
Assert-PSRule -InputPath './infra/' -Module PSRule.Rules.Azure `
              -Baseline Azure.Pillar.Security -Outcome Fail,Error
```

Local rules under `.ps-rule/` are where *your* infrastructure decisions live — the ones no vendor baseline knows about.

### 5.6 Production discharge (OpenTelemetry)

Decisions allocated to `criterion` with `discharge-stage: prod` need an event contract declared at enumeration time, and a **stated expectation before shipping**. A dashboard read afterward is unfalsifiable and always confirms.

```csharp
public static class Decisions
{
    public static readonly ActivitySource Source = new("Acme.Decisions");
    private static readonly Meter Meter = new("Acme.Decisions");

    private static readonly Counter<long> Discharge =
        Meter.CreateCounter<long>("decision.discharge");

    public static void Observe(string decisionId, string outcome, params KeyValuePair<string, object?>[] tags)
    {
        var all = new TagList { { "dec.id", decisionId }, { "dec.outcome", outcome } };
        foreach (var t in tags) all.Add(t.Key, t.Value);
        Discharge.Add(1, all);
    }
}

// at the site
Decisions.Observe("DEC-004", exported ? "ok" : "deadletter", new("tenant", tenantId));
```

Reconcile query (Application Insights / Log Analytics KQL):

```kusto
customMetrics
| where timestamp > ago(14d)
| extend decId = tostring(customDimensions["dec.id"]),
         outcome = tostring(customDimensions["dec.outcome"])
| where isnotempty(decId)
| summarize total = count(),
            failures = countif(outcome != "ok"),
            users = dcount(user_Id)
  by decId
| extend failureRate = round(100.0 * failures / total, 3)
```

Then a scheduled job joins this against the `expectation` field in `decisions.yml` and opens a finding where reality and expectation diverge.

**Expect low coverage at first.** Most decisions will have no production discharge — some are unmeasurable in principle, many are measurable only on a cadence slower than the decision cycle. That gap is the exposure. The deliverable is an honest coverage number, not a full dashboard.

---

## 6. How to review for escaped decisions

You cannot find escapes by reading for wrongness. Escaped decisions produce **plausible** code — that is the definition. You read for **choice points**.

**The operative question at each site:** *could this defensibly have been written another way, and would that change the outcome past tolerance?* If yes, it is a governing decision, and it is either in the table or it escaped.

### 6.1 Protocol

1. **Read the table first.** Is the enumeration complete at this tolerance? Is every decision allocated? Are escapes named, priced, and accepted?
2. **Then read the code with one job:** find decisions *not in the table*. Each becomes either a new row with an allocation, or a recorded judgment that it is not governing at this tolerance. Both are ledger entries.
3. **Convert repeats.** Any escape class caught twice becomes an analyzer or a PSRule rule. It never reaches review again.

Reviewer effort now scales with decision count, not diff size — which is the actual fix for review overwhelm. A 900-line diff can be nearly decision-free; a 40-line one can carry twenty governing decisions.

### 6.2 C# escape checklist

- **Omitted overload arguments** — `string.Equals` without `StringComparison`; `Parse`/`ToString` without `CultureInfo`; `DateTime.Now` vs `UtcNow`; `double` where money is meant
- **Every `catch`** — failure behaviour is always governing; `catch (Exception) { _logger.LogError(e, "..."); }` is an escape essentially always
- **Every `!` and `#pragma warning disable`** — literally marked escape sites
- **Ordering assumptions** — `.First()` / `.Single()` / `.Take(n)` without an `OrderBy`
- **Boundary mapping** — DTO ↔ domain; convention-based mappers (AutoMapper) are mass escape generators: mapping decided by convention means decided by nobody
- **EF Core** — tracking vs. no-tracking, `SaveChanges` placement, transaction boundary, cascade behaviour, N+1 shape, `IsolationLevel`
- **Process boundaries** — timeout, retry count, backoff, idempotency, partial-failure semantics, poison-message handling
- **Serialization** — casing policy, enum-as-string, unknown-member handling, null handling, date format
- **Async shape** — `async void`, fire-and-forget, `HttpClient` lifetime, missing `CancellationToken`
- **Concurrency** — optimistic vs. pessimistic, conflict resolution, last-write-wins by default
- **Authorization** — every endpoint's default when no attribute is present

### 6.3 Bicep escape checklist

- **Every default parameter value** — a default is a decision made once for all callers
- **Every hardcoded SKU, tier, capacity** — cost and performance decisions
- **RBAC assignment scope** — subscription vs. RG vs. resource is always governing
- **Public network access, private endpoints, firewall rules**
- **Diagnostic settings and retention** — absent means "decided by nobody"
- **`existing` references** — an assumption about deployment order and ownership
- **Secret handling** — Key Vault reference vs. parameter vs. output
- **API versions** — an implicit decision about behaviour and available features
- **Idempotency of the deployment itself** — what happens on re-run

---

## 7. Pipeline shape

```yaml
# GitHub Actions — stage names mapped to the ground table in §5.1
jobs:
  readiness:                       # GATE 1 — before transcription is even reviewed
    steps:
      - run: ddd validate                # until PRD M8 lands, a standalone validator
        # no unallocated decisions; no expired acceptances;
        # every escape has exposure + accepted-by + review-by

  static:                          # PR ground
    steps:
      - run: dotnet build -warnaserror
      - run: dotnet test --filter Category!=Integration
      - run: az bicep lint --file infra/main.bicep
      - run: pwsh -c "Assert-PSRule -InputPath ./infra/ -Outcome Fail,Error"
      - run: ./scripts/whatif-assert.sh ephemeral

  dev:                             # service-behaviour ground
    steps:
      - run: az stack group create ... --deny-settings-mode denyDelete
      - run: dotnet test --filter Category=Integration

  staging:                         # scale + data ground
    steps:
      - run: ./scripts/restore-prod-snapshot.sh
      - run: dotnet test --filter Category=Migration
      - run: ./scripts/perf-assert.sh

  production:
    steps:
      - run: ./scripts/whatif-assert.sh prod    # no unexpected Delete
      - run: az stack group create ...
      - run: ./scripts/register-prod-discharges.sh   # arms the §5.5 expectations

  reconcile:                       # GATE 2 — scheduled, not per-deploy
    schedule: "0 6 * * 1"
    steps:
      - run: ./scripts/reconcile-kql.sh   # expectation vs. observed, opens findings
```

Two gates carry the process: **readiness** (nothing unallocated, before produce) and **completeness** (every enumerated decision has a disposition, before release). Pass-rate is not a gate — pass-rate over an incomplete set is the metric that will get optimised for.

---

## 8. Standing instruments

Four numbers. Without them this is ceremony.

| Instrument | Definition | Reads on |
|---|---|---|
| **Late-discovery rate** | Decisions found during produce/discharge that were not enumerated, by stage | Enumeration quality. The headline number. Should fall with practice on a task type; if it does not, tolerance or actor is wrong. |
| **Escape-conversion rate** | Priced escapes that proved governing and were converted to encoded form | Whether the ledger is alive or just an accreting footgun list |
| **Analyzer-vs-human catch ratio** | Escape classes caught mechanically vs. in review | Descent measure of the encode-exercise loop |
| **Actor-substitution delta** | Swap the model, re-run the suite, measure degradation | Demand riding on a model prior rather than sitting in a store. Periodic audit; the only instrument that catches this failure mode. |

The last one has no equivalent in current practice and is the cheapest to run. A short prompt that still works after a model swap was backed by an encoded store. One that degrades was riding on that model's prior — pinned by binding, expiring silently at the next version bump, unattributable afterward.

---

## 9. What cannot be automated

Most of this loop can be run by a model actor: enumeration, allocation proposals, discharge authoring, detection, conversion. Two constraints are structural, not craft.

**Acceptance cannot be delegated.** The residual must land on an actor pinnable finely enough for the assurance level. Route acceptance to a model and the composite becomes pinnable only distributionally — at which point outcome-accountability is *unavailable*, not merely absent. No configuration recovers it. The detector proposes; a named human disposes. If that gate is bypassed for throughput, everything upstream becomes ceremony.

**The signal must be uncorrelated.** A model reviewing model-authored code shares the prior that produced it; decisions made from prior look correct to a detector with the same prior. Use a different model family for detection than for authoring, and treat the correlation diagnostic as owed rather than settled. Production telemetry is the genuinely uncorrelated signal — it is reality pushing back — but *which* telemetry is itself a decision inside the loop.

**Where autonomy is legitimate:** exactly where the acceptance predicate closes over available ground. Bug fix: closes. Performance regression: closes. Conversion-optimising change: closes over a proxy, which is fine if the proxy is the objective. Feature strategy, pricing, trust, anything with slow or unmeasured consequence: does not close, and running the loop there produces confident convergence on the measured thing.

---

## 10. Adoption order

Do not adopt this whole document. In order of leverage:

1. **`Directory.Build.props` + `.editorconfig` at error severity.** One afternoon. Converts a large fraction of the review checklist into compile errors immediately.
2. **`decisions.yml` per feature, checked in.** No tooling required. The file diffing is most of the value.
3. **Review protocol: read the table before the diff.** A process change, zero cost.
4. **The `DEC001` analyzer + ledger validator.** Makes escape non-silent.
5. **`what-if` structural assertion + PSRule in the PR stage.**
6. **The environment ground table**, written down and argued about once.
7. **Production discharge for the handful of decisions that warrant it.** Expect coverage under 20% at first, and say so.
8. **Instruments.** Late-discovery rate first; the rest follow.

Steps 1–3 are available Monday and carry most of the benefit. Everything after 4 should be justified by a number from step 8.

---

## 11. Open slots

Named rather than filled, per the framework's own discipline:

- **Enumeration completeness has no mechanical check.** Coverage is measured against the enumerated set, and nothing verifies the set itself. Late-discovery rate is a lagging proxy. This is the load-bearing gap.
- **Tolerance tiers `T0/T1/T2` are stipulated, not derived.** A derivation from consequence would need a cost model this document does not have.
- **The correlation coefficient between authoring and detecting model actors is unmeasured.** All claims about model-based detection are contingent on it.
- **`decisions.yml` as flat file does not carry supersession or provenance graphs.** It is the bootstrap form. The destination is the **Decision Ledger** (`decision-ledger-prd.md`): repo-independent `dec:` identity, content-hashed versions, acceptance signing the hash, per-decision merge, coverage over the declared set — integrated with the `ddd` tool at its M8. Absence becomes computable rather than requiring inspection. This slot is now scheduled, not open.
- **Steps 5 and 8 overlap** (§2). Whether they collapse as enumeration improves is a prediction, not a result.
