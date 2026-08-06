//! One passing plus one violating case per diff finding kind (PRD §7).

use super::*;
use crate::config::{DddConfig, DiffConfig, FindingSeverity};
use crate::configured::ConfiguredRule;
use crate::decision::{Decision, DecisionKind};
use crate::detect::DetectedRule;
use crate::manifest::{ManifestFile, ManifestRule, ManifestSet, Suppression};

fn detected(ns: &str, rule_id: &str, rule: DetectedRule) -> DetectedState {
    let mut state = DetectedState::default();
    state.sourced.insert(ns.to_string());
    state.rules.insert((ns.to_string(), rule_id.to_string()), rule);
    state
}

fn configured(severity: &str) -> DetectedRule {
    DetectedRule {
        configured: Some(ConfiguredRule {
            rule_id: "CA2007".into(),
            severity: severity.into(),
            disabled: severity == "none",
            source: ".editorconfig".into(),
            scope: Some("*.cs".into()),
        }),
        emitted: None,
    }
}

fn manifest(store: &mut DddStore, ns: &str, rule: ManifestRule) {
    store
        .manifests
        .push(ManifestSet { name: ns.into(), file: ManifestFile { format: 1, rules: vec![rule] } });
}

fn entry(rule_id: &str) -> ManifestRule {
    ManifestRule {
        rule_id: rule_id.into(),
        severity: "warning".into(),
        decision: Some("dec/x/y".into()),
        governance: None,
        suppression: None,
    }
}

fn kinds(report: &DiffReport) -> Vec<(FindingKind, String)> {
    report.findings.iter().map(|f| (f.kind, f.rule_id.clone())).collect()
}

#[test]
fn an_enabled_rule_without_an_entry_is_ungoverned() {
    let store = DddStore::default();
    let report = diff(&store, &detected("analyzers", "CA2007", configured("warning")));
    assert_eq!(kinds(&report), vec![(FindingKind::Ungoverned, "CA2007".into())]);
    assert!(report.findings[0].detail.contains("configured source only"), "{report:?}");
    assert_eq!(report.blocking(), 1);
}

#[test]
fn a_governed_rule_is_no_finding() {
    let mut store = DddStore::default();
    manifest(&mut store, "analyzers", entry("CA2007"));
    let report = diff(&store, &detected("analyzers", "CA2007", configured("warning")));
    assert!(report.findings.is_empty(), "{report:?}");
}

#[test]
fn a_manifest_entry_absent_everywhere_is_stale() {
    let mut store = DddStore::default();
    manifest(&mut store, "analyzers", entry("CA9999"));
    let report = diff(&store, &detected("analyzers", "CA2007", configured("warning")));
    assert!(kinds(&report).contains(&(FindingKind::Stale, "CA9999".into())), "{report:?}");
}

#[test]
fn staleness_is_not_judged_without_a_source() {
    let mut store = DddStore::default();
    manifest(&mut store, "analyzers", entry("CA9999"));
    let report = diff(&store, &DetectedState::default());
    assert!(report.findings.is_empty(), "{report:?}");
    assert!(
        report.notes.iter().any(|n| n.contains("no detection source")),
        "{:?}",
        report.notes
    );
}

#[test]
fn a_config_suppression_without_a_citation_is_uncited() {
    let store = DddStore::default();
    let report = diff(&store, &detected("analyzers", "CA1303", configured("none")));
    // Disabled rule: not UNGOVERNED (it is off), but its suppression is uncited.
    assert_eq!(kinds(&report), vec![(FindingKind::UncitedSuppression, "CA1303".into())]);
}

#[test]
fn a_source_suppression_without_a_citation_is_uncited() {
    let mut store = DddStore::default();
    manifest(&mut store, "analyzers", entry("CA2000"));
    let rule = DetectedRule { configured: None, emitted: Some(("warning".into(), 1, 1)) };
    let report = diff(&store, &detected("analyzers", "CA2000", rule));
    assert_eq!(kinds(&report), vec![(FindingKind::UncitedSuppression, "CA2000".into())]);
    assert!(report.findings[0].detail.contains("in source"), "{report:?}");
}

#[test]
fn a_cited_suppression_is_no_finding() {
    let mut store = DddStore::default();
    let mut e = entry("CA1303");
    e.suppression =
        Some(Suppression { risk_acceptance: Some("risk/cs/loc".into()), reason: None });
    manifest(&mut store, "analyzers", e);
    store.decisions.push(Decision {
        format: 1,
        id: "risk/cs/loc".into(),
        kind: DecisionKind::RiskAcceptance,
        title: "T".into(),
        rationale: "R".into(),
        principal: "Emil".into(),
        based_on: Vec::new(),
        date: None,
        notes: None,
    });
    let report = diff(&store, &detected("analyzers", "CA1303", configured("none")));
    assert!(report.findings.is_empty(), "{report:?}");
}

#[test]
fn a_citation_to_a_plain_decision_stays_uncited() {
    let mut store = DddStore::default();
    let mut e = entry("CA1303");
    e.suppression =
        Some(Suppression { risk_acceptance: Some("dec/x/y".into()), reason: None });
    manifest(&mut store, "analyzers", e);
    store.decisions.push(Decision {
        format: 1,
        id: "dec/x/y".into(),
        kind: DecisionKind::Decision,
        title: "T".into(),
        rationale: "R".into(),
        principal: "Emil".into(),
        based_on: Vec::new(),
        date: None,
        notes: None,
    });
    let report = diff(&store, &detected("analyzers", "CA1303", configured("none")));
    assert_eq!(kinds(&report), vec![(FindingKind::UncitedSuppression, "CA1303".into())]);
}

#[test]
fn severity_thresholds_downgrade_or_drop_kinds() {
    let store = DddStore {
        config: Some(DddConfig {
            format: 2,
            diff: Some(DiffConfig {
                ungoverned: Some(FindingSeverity::Warn),
                stale: None,
                uncited_suppression: Some(FindingSeverity::Off),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut state = detected("analyzers", "CA2007", configured("warning"));
    state
        .rules
        .insert(("analyzers".into(), "CA1303".into()), configured("none"));
    let report = diff(&store, &state);
    assert_eq!(kinds(&report), vec![(FindingKind::Ungoverned, "CA2007".into())]);
    assert_eq!(report.blocking(), 0, "warn does not block");
}
