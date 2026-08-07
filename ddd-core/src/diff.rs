//! Governance diffing — declared manifest entries vs. detected rules.
//!
//! The M2 finding taxonomy (PRD §7): `UNGOVERNED` (configured or emitted,
//! no manifest entry), `STALE` (manifest entry, rule absent from config
//! plus emissions), `UNCITED_SUPPRESSION` (a config or in-source
//! suppression with no risk-acceptance citation). Staleness is only judged
//! for namespaces that had a detection source; a rule covered by a single
//! source says so in its finding, since config parsing is deliberately
//! partial (package-default rules never appear as configured).

use serde::Serialize;

use crate::config::{DiffConfig, FindingSeverity};
use crate::decision::DecisionKind;
use crate::detect::{DetectedRule, DetectedState};
use crate::store::DddStore;

/// The three M2 finding kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingKind {
    Ungoverned,
    Stale,
    UncitedSuppression,
}

impl FindingKind {
    /// The config key this kind's severity is read from.
    pub fn key(self) -> &'static str {
        match self {
            Self::Ungoverned => "ungoverned",
            Self::Stale => "stale",
            Self::UncitedSuppression => "uncited_suppression",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Ungoverned => "UNGOVERNED",
            Self::Stale => "STALE",
            Self::UncitedSuppression => "UNCITED_SUPPRESSION",
        }
    }
}

/// One governance escape, named by manifest entry id.
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub kind: FindingKind,
    /// The manifest namespace (file stem) the rule belongs to.
    pub namespace: String,
    pub rule_id: String,
    pub severity: FindingSeverity,
    /// What was seen, including which source(s) covered the rule.
    pub detail: String,
}

/// The whole diff: findings plus non-finding observations.
#[derive(Debug, Default, Serialize)]
pub struct DiffReport {
    pub findings: Vec<Finding>,
    pub notes: Vec<String>,
}

impl DiffReport {
    /// Findings that block (severity `error`).
    pub fn blocking(&self) -> usize {
        self.findings.iter().filter(|f| f.severity == FindingSeverity::Error).count()
    }
}

/// Join the store's manifests against the detected state.
pub fn diff(store: &DddStore, detected: &DetectedState) -> DiffReport {
    let cfg = store.config.as_ref().and_then(|c| c.diff.clone()).unwrap_or_default();
    let mut report = DiffReport { findings: Vec::new(), notes: detected.notes.clone() };
    detected_side(store, detected, &cfg, &mut report);
    manifest_side(store, detected, &cfg, &mut report);
    for ns in manifest_namespaces(store) {
        if !detected.sourced.contains(&ns) {
            report.notes.push(format!(
                "namespace {ns}: no detection source seen — staleness not evaluated (provide SARIF via detect.sarif or --sarif)"
            ));
        }
    }
    report.findings.sort_by(|a, b| {
        (a.kind.label(), &a.namespace, &a.rule_id).cmp(&(b.kind.label(), &b.namespace, &b.rule_id))
    });
    report
}

fn push(report: &mut DiffReport, cfg: &DiffConfig, kind: FindingKind, ns: &str, rule_id: &str, detail: String) {
    let severity = cfg.severity_of(kind.key());
    if severity == FindingSeverity::Off {
        return;
    }
    report.findings.push(Finding {
        kind,
        namespace: ns.to_string(),
        rule_id: rule_id.to_string(),
        severity,
        detail,
    });
}

/// Detected rules with no (or an incomplete) manifest side.
fn detected_side(store: &DddStore, detected: &DetectedState, cfg: &DiffConfig, report: &mut DiffReport) {
    for ((ns, rule_id), rule) in &detected.rules {
        let entry = manifest_entry(store, ns, rule_id);
        if entry.is_none() && rule.enabled() {
            push(report, cfg, FindingKind::Ungoverned, ns, rule_id, sources_line(rule));
        }
        let suppressed = rule.config_suppressed() || rule.source_suppressed();
        if suppressed && !cited_suppression(store, entry) {
            let what = match (rule.config_suppressed(), rule.source_suppressed()) {
                (true, true) => "suppressed in config plus in source".to_string(),
                (true, false) => format!(
                    "suppressed in config ({})",
                    rule.configured.as_ref().map(|c| c.source.as_str()).unwrap_or("?")
                ),
                _ => "suppressed in source (pragma/attribute logged by the build)".to_string(),
            };
            push(
                report,
                cfg,
                FindingKind::UncitedSuppression,
                ns,
                rule_id,
                format!("{what}; no risk-acceptance citation in the manifest"),
            );
        }
    }
}

/// Manifest entries whose rule detection never saw.
fn manifest_side(store: &DddStore, detected: &DetectedState, cfg: &DiffConfig, report: &mut DiffReport) {
    for set in &store.manifests {
        if !detected.sourced.contains(&set.name) {
            continue;
        }
        for rule in &set.file.rules {
            let key = (set.name.clone(), rule.rule_id.clone());
            if !detected.rules.contains_key(&key) {
                push(
                    report,
                    cfg,
                    FindingKind::Stale,
                    &set.name,
                    &rule.rule_id,
                    "in the manifest, absent from config plus emissions".to_string(),
                );
            }
        }
    }
}

fn manifest_entry<'a>(
    store: &'a DddStore,
    ns: &str,
    rule_id: &str,
) -> Option<&'a crate::manifest::ManifestRule> {
    store
        .manifests
        .iter()
        .find(|s| s.name == ns)
        .and_then(|s| s.file.rules.iter().find(|r| r.rule_id == rule_id))
}

/// A detected suppression is cited when its manifest entry names a
/// risk-acceptance record that exists (ontology rule 4's diff-side half).
fn cited_suppression(store: &DddStore, entry: Option<&crate::manifest::ManifestRule>) -> bool {
    let Some(id) = entry
        .and_then(|r| r.suppression.as_ref())
        .and_then(|s| s.risk_acceptance.as_deref())
    else {
        return false;
    };
    store
        .decisions
        .iter()
        .any(|d| d.id == id && d.kind == DecisionKind::RiskAcceptance)
}

/// Name which source(s) saw the rule; single-source coverage is stated
/// because config parsing is deliberately partial (PRD §7).
/// Find a detected rule by fully-qualified (`analyzers/CA2007`) or bare
/// (`CA2007`) id, returning its namespace, rule id, and the same
/// how-detected line `diff` prints — so `why` and `diff` cannot disagree
/// about detection (`dec/ddd/why-resolves-three-ways`).
pub fn find_detected(detected: &DetectedState, id: &str) -> Option<(String, String, String)> {
    detected
        .rules
        .iter()
        .find(|((ns, rule_id), _)| id == rule_id || id == format!("{ns}/{rule_id}"))
        .map(|((ns, rule_id), rule)| (ns.clone(), rule_id.clone(), sources_line(rule)))
}

fn sources_line(rule: &DetectedRule) -> String {
    match (&rule.configured, &rule.emitted) {
        (Some(c), Some((level, n, s))) => format!(
            "configured {} in {}{}; emitted {n}x as {level} ({s} suppressed)",
            c.severity,
            c.source,
            scope_of(c)
        ),
        (Some(c), None) => format!(
            "configured {} in {}{} — configured source only (no emission ingested)",
            c.severity,
            c.source,
            scope_of(c)
        ),
        (None, Some((level, n, s))) => format!(
            "emitted {n}x as {level} ({s} suppressed) — emitted source only (no config line; may be an analyzer-package default)"
        ),
        (None, None) => "detected by no source".to_string(),
    }
}

fn scope_of(c: &crate::configured::ConfiguredRule) -> String {
    c.scope.as_ref().map(|s| format!(" [{s}]")).unwrap_or_default()
}

fn manifest_namespaces(store: &DddStore) -> Vec<String> {
    store
        .manifests
        .iter()
        .filter(|s| !s.file.rules.is_empty())
        .map(|s| s.name.clone())
        .collect()
}

#[cfg(test)]
#[path = "diff_tests.rs"]
mod tests;
