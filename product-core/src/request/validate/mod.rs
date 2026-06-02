//! Request validation (FT-041, ADR-038).
//!
//! Every finding is reported in one pass.

pub mod artifacts;
pub mod changes;
pub mod helpers;

use super::types::*;
use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use regex::Regex;
use std::collections::HashMap;

pub struct ValidationContext<'a> {
    pub config: &'a ProductConfig,
    pub graph: &'a KnowledgeGraph,
}

pub fn validate_request(request: &Request, ctx: &ValidationContext<'_>) -> Vec<Finding> {
    let mut findings = Vec::new();
    check_reason(request, &mut findings);
    check_section_coherence(request, &mut findings);
    let refs = collect_refs(request, &mut findings);
    for a in &request.artifacts {
        artifacts::validate_artifact(a, &refs, ctx, &mut findings);
    }
    for c in &request.changes {
        changes::validate_change(c, &refs, ctx, &mut findings);
    }
    // FT-064 — deletion targets must exist and must have no inbound links
    // (cascade is out of scope for this feature).
    for d in &request.deletions {
        validate_deletion(d, ctx, &mut findings);
    }
    // FT-058 / E022: any change that promotes a feature into a status
    // requiring runner config must carry it for every linked TC.
    check_tc_runner_required_on_status_change(request, ctx, &mut findings);
    findings
}

/// FT-064 — validate a `deletions[]` entry. Two checks:
///   1. `E002` if the target ID does not exist in the graph.
///   2. `E027` if other artifacts link to the target (refuses orphaning).
fn validate_deletion(
    d: &DeletionSpec,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    if d.target.trim().is_empty() {
        findings.push(Finding::error(
            "E001",
            "deletion target must not be empty",
            format!("$.deletions[{}].target", d.index),
        ));
        return;
    }
    if !ctx.graph.all_ids().contains(&d.target) {
        findings.push(Finding::error(
            "E002",
            format!("deletion target '{}' does not exist in the graph", d.target),
            format!("$.deletions[{}].target", d.index),
        ));
        return;
    }
    let referrers = find_referrers(ctx.graph, &d.target);
    if !referrers.is_empty() {
        let mut shown = referrers.clone();
        shown.sort();
        shown.dedup();
        let summary = if shown.len() > 8 {
            format!("{} (and {} more)", shown[..8].join(", "), shown.len() - 8)
        } else {
            shown.join(", ")
        };
        findings.push(
            Finding::error(
                "E027",
                format!(
                    "cannot delete '{}' — {} other artifact(s) still link to it: {}",
                    d.target,
                    shown.len(),
                    summary
                ),
                format!("$.deletions[{}].target", d.index),
            )
            .with_hint(
                "remove the inbound links first (e.g. `op: remove` on each \
                 referrer's list) or open a separate request that supersedes \
                 the link",
            ),
        );
    }
}

/// Return every artifact ID that has an outgoing link to `target` in the graph.
/// Used by `validate_deletion` to refuse silently orphaning live references.
fn find_referrers(
    graph: &crate::graph::KnowledgeGraph,
    target: &str,
) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (id, f) in &graph.features {
        if f.front.adrs.iter().any(|x| x == target)
            || f.front.tests.iter().any(|x| x == target)
            || f.front.depends_on.iter().any(|x| x == target)
        {
            out.push(id.clone());
        }
    }
    for (id, a) in &graph.adrs {
        if a.front.features.iter().any(|x| x == target)
            || a.front.supersedes.iter().any(|x| x == target)
            || a.front.superseded_by.iter().any(|x| x == target)
        {
            out.push(id.clone());
        }
    }
    for (id, t) in &graph.tests {
        if t.front.validates.features.iter().any(|x| x == target)
            || t.front.validates.adrs.iter().any(|x| x == target)
        {
            out.push(id.clone());
        }
    }
    for (id, d) in &graph.dependencies {
        if d.front.adrs.iter().any(|x| x == target)
            || d.front.features.iter().any(|x| x == target)
            || d.front.supersedes.iter().any(|x| x == target)
        {
            out.push(id.clone());
        }
    }
    out.sort();
    out.dedup();
    out
}

/// FT-058 / E022 — refuse a `set status: in-progress|complete` mutation
/// when the target feature's linked TCs lack runner config.
fn check_tc_runner_required_on_status_change(
    request: &Request,
    ctx: &ValidationContext<'_>,
    findings: &mut Vec<Finding>,
) {
    use crate::tc::runner_required;
    use crate::types::FeatureStatus;
    use serde_yaml::Value;
    use std::str::FromStr;

    for c in &request.changes {
        // Only existing-graph targets; refs to artifacts created in the
        // same request would not yet have linked TCs anyway.
        if c.target.starts_with("ref:") {
            continue;
        }
        if !ctx.graph.features.contains_key(&c.target) {
            continue;
        }
        for m in &c.mutations {
            if m.field.trim() != "status" {
                continue;
            }
            let MutationOp::Set = m.op else { continue };
            let Some(Value::String(s)) = &m.value else { continue };
            let Ok(target_status) = FeatureStatus::from_str(s.trim()) else {
                continue;
            };
            if !runner_required::status_requires_runner(target_status) {
                continue;
            }
            let offenders =
                runner_required::find_offenders(ctx.graph, &c.target, target_status);
            if offenders.is_empty() {
                continue;
            }
            findings.push(Finding::error(
                "E022",
                format!(
                    "TC runner configuration missing — {} TC(s) linked to {} lack `runner` and/or `runner-args`: {}",
                    offenders.len(),
                    c.target,
                    offenders.join(", "),
                ),
                format!("$.changes[{}].mutations[{}].value", c.index, m.index),
            ));
        }
    }
}

fn check_reason(request: &Request, findings: &mut Vec<Finding>) {
    if request.reason.trim().is_empty() {
        findings.push(Finding::error(
            "E011",
            "request 'reason' is required and must not be empty",
            "$.reason",
        ));
    }
}

fn check_section_coherence(request: &Request, findings: &mut Vec<Finding>) {
    match request.request_type {
        RequestType::Create => {
            if request.artifacts.is_empty() {
                findings.push(Finding::error("E006", "'type: create' requires at least one artifact", "$.artifacts"));
            }
            if !request.changes.is_empty() {
                findings.push(Finding::error("E006", "'type: create' must not contain a 'changes' section", "$.changes"));
            }
            if !request.deletions.is_empty() {
                findings.push(Finding::error("E006", "'type: create' must not contain a 'deletions' section", "$.deletions"));
            }
        }
        RequestType::Change => {
            if request.changes.is_empty() {
                findings.push(Finding::error("E006", "'type: change' requires at least one change", "$.changes"));
            }
            if !request.artifacts.is_empty() {
                findings.push(Finding::error("E006", "'type: change' must not contain an 'artifacts' section", "$.artifacts"));
            }
            if !request.deletions.is_empty() {
                findings.push(Finding::error("E006", "'type: change' must not contain a 'deletions' section", "$.deletions"));
            }
        }
        RequestType::CreateAndChange => {
            if request.artifacts.is_empty() && request.changes.is_empty() {
                findings.push(Finding::error(
                    "E006",
                    "'type: create-and-change' requires at least one artifact or change",
                    "$",
                ));
            }
            if !request.deletions.is_empty() {
                findings.push(Finding::error("E006", "'type: create-and-change' must not contain a 'deletions' section", "$.deletions"));
            }
        }
        RequestType::Delete => {
            if request.deletions.is_empty() {
                findings.push(Finding::error(
                    "E006",
                    "'type: delete' requires at least one entry in 'deletions'",
                    "$.deletions",
                ));
            }
            if !request.artifacts.is_empty() {
                findings.push(Finding::error("E006", "'type: delete' must not contain an 'artifacts' section", "$.artifacts"));
            }
            if !request.changes.is_empty() {
                findings.push(Finding::error("E006", "'type: delete' must not contain a 'changes' section", "$.changes"));
            }
        }
    }
}

fn collect_refs(
    request: &Request,
    findings: &mut Vec<Finding>,
) -> HashMap<String, (ArtifactType, usize)> {
    let ref_re = Regex::new(r"^[a-z][a-z0-9-]*$").expect("constant regex");
    let mut refs: HashMap<String, (ArtifactType, usize)> = HashMap::new();
    for a in &request.artifacts {
        if let Some(ref name) = a.ref_name {
            if !ref_re.is_match(name) {
                findings.push(Finding::error(
                    "E001",
                    format!("invalid ref name '{}' — must match ^[a-z][a-z0-9-]*$", name),
                    format!("$.artifacts[{}].ref", a.index),
                ));
            } else if refs.contains_key(name) {
                findings.push(Finding::error(
                    "E001",
                    format!("duplicate ref name '{}'", name),
                    format!("$.artifacts[{}].ref", a.index),
                ));
            } else {
                refs.insert(name.clone(), (a.artifact_type, a.index));
            }
        }
    }
    refs
}

/// Placeholder hook — currently unused but reserved for cross-artifact cycle checks.
pub fn validate_against_graph(
    _request: &Request,
    _ctx: &ValidationContext<'_>,
) -> Vec<Finding> {
    Vec::new()
}

// Re-exports for the apply pipeline.
pub use helpers::strip_ref_prefix;

/// Post-validation pass: every DEP must have at least one governing ADR (E013)
/// either on its own `adrs` field or referenced by another ADR's `governs` field.
pub fn check_dep_governance(
    request: &Request,
    _refs: &HashMap<String, (ArtifactType, usize)>,
    _graph: &KnowledgeGraph,
    findings: &mut Vec<Finding>,
) {
    use serde_yaml::Value;
    for a in &request.artifacts {
        if a.artifact_type != ArtifactType::Dep {
            continue;
        }
        let own_adrs = a.fields.get(Value::String("adrs".into()));
        let has_own = matches!(own_adrs, Some(Value::Sequence(s)) if !s.is_empty());
        if has_own { continue; }

        let this_ref = a.ref_name.as_deref();
        let mut found = false;
        for other in &request.artifacts {
            if other.artifact_type != ArtifactType::Adr { continue; }
            if let Some(Value::Sequence(seq)) = other.fields.get(Value::String("governs".into())) {
                for item in seq {
                    if let Value::String(s) = item {
                        if let Some(r) = strip_ref_prefix(s) {
                            if Some(r) == this_ref {
                                found = true;
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !found {
            findings.push(Finding::error(
                "E013",
                "dependency has no governing ADR in the request — add a 'governs: [ref:...]' on an ADR or declare 'adrs: [ADR-NNN]' on the dep",
                format!("$.artifacts[{}]", a.index),
            ));
        }
    }
}
