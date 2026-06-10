//! Clause evaluation for the Two Pillars conformance report (ADR-052).

use crate::feature::body_sections::parse_body_sections;
use crate::graph::KnowledgeGraph;
use crate::types::{
    Adr, AdrScope, AdrStatus, Feature, FeatureStatus, TestCriterion, TestStatus,
};

use super::{
    ClauseMode, ClauseOutcome, ClauseSeverity, ConformanceFinding, ConformanceReport,
    ConformanceSummary, ProjectDeclarations, SPEC_ID,
};

/// Clause registry: identifier, short title, evaluation mode. Order is the
/// report order. `ByConstruction` clauses pass once the graph loads — the
/// loader rejects dependency or supersession cycles (E003/E004) before any
/// check runs.
const CLAUSES: &[(&str, &str, ClauseMode)] = &[
    ("SPEC-SPLIT-1", "What plus How are separate artifacts", ClauseMode::ByConstruction),
    ("SPEC-WHAT-1", "Single declared system identity", ClauseMode::Checked),
    ("SPEC-WHAT-2", "Declared purpose for the system", ClauseMode::Checked),
    ("SPEC-WHAT-4", "Behaviours declare error handling", ClauseMode::Checked),
    ("SPEC-WHAT-5", "Non-empty out-of-scope declaration", ClauseMode::Checked),
    ("SPEC-WHAT-8", "Acceptance criterion per behaviour", ClauseMode::Checked),
    ("SPEC-HOW-2.1", "One responsibility per decision", ClauseMode::Checked),
    ("SPEC-HOW-2.2", "Acyclic dependency graph", ClauseMode::ByConstruction),
    ("SPEC-HOW-5", "Decisions record rejected alternatives", ClauseMode::Checked),
    ("SPEC-DERIVE-3", "No undeclared product decisions", ClauseMode::Checked),
    ("EXEC-CLOSE-4", "Output judged before acceptance", ClauseMode::Checked),
];

/// Evaluate the knowledge graph against the checkable clause subset of the
/// Two Pillars specification, Level 3 profile.
pub fn check(graph: &KnowledgeGraph, project: &ProjectDeclarations) -> ConformanceReport {
    let mut findings = Vec::new();

    check_split_1(project, &mut findings);
    check_what_1(project, &mut findings);
    check_what_2(project, &mut findings);
    for feature in sorted_features(graph) {
        check_what_4(feature, &mut findings);
        check_what_5(feature, &mut findings);
        check_what_8(graph, feature, &mut findings);
        check_exec_close_4(graph, feature, &mut findings);
    }
    for adr in sorted_adrs(graph) {
        check_how_2_1(adr, &mut findings);
        check_how_5(adr, &mut findings);
        check_derive_3(graph, adr, &mut findings);
    }

    build_report(findings)
}

fn build_report(findings: Vec<ConformanceFinding>) -> ConformanceReport {
    let clauses: Vec<ClauseOutcome> = CLAUSES
        .iter()
        .map(|(id, title, mode)| {
            let count = findings.iter().filter(|f| f.clause == *id).count();
            let violated = findings
                .iter()
                .any(|f| f.clause == *id && f.severity == ClauseSeverity::Violation);
            ClauseOutcome {
                clause: (*id).to_string(),
                title: (*title).to_string(),
                mode: *mode,
                // Advisories (disregarded SHOULDs) never fail a clause.
                passed: !violated,
                findings: count,
            }
        })
        .collect();

    let violations = findings.iter().filter(|f| f.severity == ClauseSeverity::Violation).count();
    let advisories = findings.iter().filter(|f| f.severity == ClauseSeverity::Advisory).count();
    let summary = ConformanceSummary {
        clauses_checked: clauses.len(),
        clauses_passed: clauses.iter().filter(|c| c.passed).count(),
        violations,
        advisories,
    };
    let profile = if violations == 0 { "level-3" } else { "below-level-3" };

    ConformanceReport {
        spec: SPEC_ID.to_string(),
        profile: profile.to_string(),
        scope: "checkable-subset".to_string(),
        clauses,
        findings,
        summary,
    }
}

fn sorted_features(graph: &KnowledgeGraph) -> Vec<&Feature> {
    let mut features: Vec<&Feature> = graph.features.values().collect();
    features.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    features
}

fn sorted_adrs(graph: &KnowledgeGraph) -> Vec<&Adr> {
    let mut adrs: Vec<&Adr> = graph.adrs.values().collect();
    adrs.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    adrs
}

/// All TCs linked to a feature: declared in `tests:` or validating it.
fn linked_tcs<'a>(graph: &'a KnowledgeGraph, feature: &Feature) -> Vec<&'a TestCriterion> {
    let mut tcs: Vec<&TestCriterion> = graph
        .tests
        .values()
        .filter(|t| {
            feature.front.tests.contains(&t.front.id)
                || t.front.validates.features.contains(&feature.front.id)
        })
        .collect();
    tcs.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    tcs
}

fn finding(
    clause: &str,
    severity: ClauseSeverity,
    artifact: Option<&str>,
    description: String,
    action: &str,
) -> ConformanceFinding {
    ConformanceFinding {
        clause: clause.to_string(),
        severity,
        artifact: artifact.map(str::to_string),
        description,
        suggested_action: action.to_string(),
    }
}

/// SPEC-SPLIT-1 — What artifacts (features) live apart from How artifacts
/// (ADRs). Separation is the artifact model itself; the only configuration
/// that can collapse it is pointing both kinds at the same directory.
fn check_split_1(project: &ProjectDeclarations, findings: &mut Vec<ConformanceFinding>) {
    if !project.features_path.is_empty() && project.features_path == project.adrs_path {
        findings.push(finding(
            "SPEC-SPLIT-1",
            ClauseSeverity::Violation,
            None,
            "features (What) and ADRs (How) are configured to the same directory".to_string(),
            "Point [paths].features and [paths].adrs at distinct directories.",
        ));
    }
}

/// SPEC-WHAT-1 — the system declares a single identity (`name` in config).
fn check_what_1(project: &ProjectDeclarations, findings: &mut Vec<ConformanceFinding>) {
    if project.name.trim().is_empty() {
        findings.push(finding(
            "SPEC-WHAT-1",
            ClauseSeverity::Violation,
            None,
            "no system identity declared — `name` in product config is empty".to_string(),
            "Set `name` in the product config to the system's single identity.",
        ));
    }
}

/// SPEC-WHAT-2 — the system declares its purpose
/// (`[product].responsibility` in config).
fn check_what_2(project: &ProjectDeclarations, findings: &mut Vec<ConformanceFinding>) {
    let empty = project.responsibility.as_deref().map(str::trim).unwrap_or("").is_empty();
    if empty {
        findings.push(finding(
            "SPEC-WHAT-2",
            ClauseSeverity::Violation,
            None,
            "no purpose declared — `[product].responsibility` is missing or empty".to_string(),
            "Declare the product responsibility: what problem it solves, for whom.",
        ));
    }
}

fn what_pillar_exempt(feature: &Feature) -> bool {
    feature.front.status == FeatureStatus::Abandoned
}

/// SPEC-WHAT-4 — every behaviour states its exception conditions. In the
/// feature body that is the `Functional Specification` section with
/// non-empty `Behaviour` plus `Error handling` subsections.
fn check_what_4(feature: &Feature, findings: &mut Vec<ConformanceFinding>) {
    if what_pillar_exempt(feature) {
        return;
    }
    let sections = parse_body_sections(&feature.body);
    let fs = "Functional Specification";
    let missing: Vec<&str> = if !sections.h2_has_content(fs) {
        vec![fs]
    } else {
        ["Behaviour", "Error handling"]
            .into_iter()
            .filter(|sub| !sections.h3_has_content(fs, sub))
            .collect()
    };
    if missing.is_empty() {
        return;
    }
    findings.push(finding(
        "SPEC-WHAT-4",
        ClauseSeverity::Violation,
        Some(&feature.front.id),
        format!(
            "{} does not declare its behaviour with exception conditions (missing: {})",
            feature.front.id,
            missing.join(", ")
        ),
        "Fill the Functional Specification's Behaviour plus Error handling subsections.",
    ));
}

/// SPEC-WHAT-5 — a non-empty out-of-scope list per What unit.
fn check_what_5(feature: &Feature, findings: &mut Vec<ConformanceFinding>) {
    if what_pillar_exempt(feature) {
        return;
    }
    let sections = parse_body_sections(&feature.body);
    if sections.h2_has_content("Out of scope") {
        return;
    }
    findings.push(finding(
        "SPEC-WHAT-5",
        ClauseSeverity::Violation,
        Some(&feature.front.id),
        format!("{} has no non-empty `Out of scope` section", feature.front.id),
        "Declare what the feature explicitly does not do under `## Out of scope`.",
    ));
}

/// SPEC-WHAT-8 — at least one testable acceptance criterion per behaviour.
/// In Product, acceptance criteria are TCs linked to the feature.
fn check_what_8(graph: &KnowledgeGraph, feature: &Feature, findings: &mut Vec<ConformanceFinding>) {
    if what_pillar_exempt(feature) {
        return;
    }
    if !linked_tcs(graph, feature).is_empty() {
        return;
    }
    findings.push(finding(
        "SPEC-WHAT-8",
        ClauseSeverity::Violation,
        Some(&feature.front.id),
        format!("{} has no linked test criterion (acceptance criterion)", feature.front.id),
        "Create a TC whose `validates.features` lists this feature.",
    ));
}

/// EXEC-CLOSE-4 — output is judged before acceptance: a feature may be
/// `complete` only when every linked TC verdict is `passing`, or
/// `unrunnable` — the acknowledged platform-skip verdict `product verify`
/// itself accepts for completion.
fn check_exec_close_4(
    graph: &KnowledgeGraph,
    feature: &Feature,
    findings: &mut Vec<ConformanceFinding>,
) {
    if feature.front.status != FeatureStatus::Complete {
        return;
    }
    let unjudged: Vec<String> = linked_tcs(graph, feature)
        .iter()
        .filter(|t| {
            t.front.status != TestStatus::Passing && t.front.status != TestStatus::Unrunnable
        })
        .map(|t| format!("{} ({})", t.front.id, t.front.status))
        .collect();
    if unjudged.is_empty() {
        return;
    }
    findings.push(finding(
        "EXEC-CLOSE-4",
        ClauseSeverity::Violation,
        Some(&feature.front.id),
        format!(
            "{} is complete but accepted output lacks a passing verdict: {}",
            feature.front.id,
            unjudged.join(", ")
        ),
        "Run `product verify` so every linked TC passes before completion.",
    ));
}

fn how_pillar_exempt(adr: &Adr) -> bool {
    adr.front.status != AdrStatus::Accepted
}

/// SPEC-HOW-2.1 — exactly one responsibility per How unit. A top-level
/// " and " in an accepted ADR title suggests two fused decisions.
fn check_how_2_1(adr: &Adr, findings: &mut Vec<ConformanceFinding>) {
    if how_pillar_exempt(adr) {
        return;
    }
    if !crate::graph::responsibility::contains_top_level_conjunction(&adr.front.title) {
        return;
    }
    findings.push(finding(
        "SPEC-HOW-2.1",
        ClauseSeverity::Advisory,
        Some(&adr.front.id),
        format!(
            "{} title contains a top-level \" and \" — possibly two fused decisions",
            adr.front.id
        ),
        "Split the decision, or reword the title to one responsibility.",
    ));
}

/// SPEC-HOW-5 — every decision documents its rejected alternatives.
fn check_how_5(adr: &Adr, findings: &mut Vec<ConformanceFinding>) {
    if how_pillar_exempt(adr) {
        return;
    }
    let has_section = adr.body.contains("Rejected alternatives")
        || adr.body.contains("rejected alternatives")
        || adr.body.contains("**Rejected");
    if has_section {
        return;
    }
    findings.push(finding(
        "SPEC-HOW-5",
        ClauseSeverity::Violation,
        Some(&adr.front.id),
        format!("{} documents no rejected alternatives", adr.front.id),
        "Add a **Rejected alternatives** section: options considered, why rejected.",
    ));
}

/// SPEC-DERIVE-3 — a How element with no What anchor is an undeclared
/// product decision. An accepted, feature-specific ADR must be reachable
/// from at least one feature; broader scopes anchor at the system level.
fn check_derive_3(graph: &KnowledgeGraph, adr: &Adr, findings: &mut Vec<ConformanceFinding>) {
    if how_pillar_exempt(adr) || adr.front.scope != AdrScope::FeatureSpecific {
        return;
    }
    let anchored = !adr.front.features.is_empty()
        || graph.features.values().any(|f| f.front.adrs.contains(&adr.front.id));
    if anchored {
        return;
    }
    findings.push(finding(
        "SPEC-DERIVE-3",
        ClauseSeverity::Violation,
        Some(&adr.front.id),
        format!(
            "{} is feature-specific but anchors to no feature — undeclared product decision",
            adr.front.id
        ),
        "Link the ADR to the feature requiring it, or widen its declared scope.",
    ));
}
