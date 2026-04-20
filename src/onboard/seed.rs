//! Phase 3: Seed — convert confirmed candidates into ADR files plus feature stubs (ADR-027)

use crate::error::Result;
use crate::fileops;
use crate::parser;
use crate::types::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::path::Path;

use super::types::*;

/// Plan the seed phase: determine what files would be created.
pub fn plan_seed(
    triage_output: &TriageOutput,
    existing_adr_ids: &[String],
    existing_feature_ids: &[String],
    adr_prefix: &str,
    feature_prefix: &str,
) -> SeedResult {
    let confirmed: Vec<&TriagedCandidate> = triage_output
        .candidates
        .iter()
        .filter(|c| c.triage_status == TriageStatus::Confirmed)
        .collect();

    // Assign ADR IDs
    let mut adr_ids_used: Vec<String> = existing_adr_ids.to_vec();
    let mut proposed_adrs = Vec::new();

    for tc in &confirmed {
        let adr_id = parser::next_id(adr_prefix, &adr_ids_used);
        adr_ids_used.push(adr_id.clone());

        let filename = parser::id_to_filename(&adr_id, &tc.candidate.title);
        proposed_adrs.push(ProposedAdr {
            id: adr_id,
            title: tc.candidate.title.clone(),
            observation: tc.candidate.observation.clone(),
            evidence: tc.candidate.evidence.clone(),
            hypothesised_consequence: tc.candidate.hypothesised_consequence.clone(),
            filename,
        });
    }

    // Group candidates into feature stubs by evidence file proximity
    let features = group_into_features(&proposed_adrs, existing_feature_ids, feature_prefix);

    SeedResult {
        adrs: proposed_adrs,
        features,
    }
}

/// Group ADRs into feature stubs based on evidence file proximity.
///
/// ADRs whose evidence files share the same parent directory are grouped together.
pub(crate) fn group_into_features(
    adrs: &[ProposedAdr],
    existing_feature_ids: &[String],
    feature_prefix: &str,
) -> Vec<ProposedFeatureStub> {
    if adrs.is_empty() {
        return Vec::new();
    }

    // Build a map: directory -> ADR IDs
    let mut dir_to_adrs: HashMap<String, Vec<String>> = HashMap::new();
    for adr in adrs {
        // Get the primary evidence directory
        let primary_dir = adr
            .evidence
            .first()
            .map(|ev| {
                let p = PathBuf::from(&ev.file);
                p.parent()
                    .map(|d| d.to_string_lossy().to_string())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        dir_to_adrs
            .entry(if primary_dir.is_empty() {
                "root".to_string()
            } else {
                primary_dir
            })
            .or_default()
            .push(adr.id.clone());
    }

    // Create one feature stub per directory cluster
    let mut feature_ids_used: Vec<String> = existing_feature_ids.to_vec();
    let mut features = Vec::new();

    let mut dirs: Vec<_> = dir_to_adrs.into_iter().collect();
    dirs.sort_by(|a, b| a.0.cmp(&b.0));

    for (dir, adr_ids) in dirs {
        let feature_id = parser::next_id(feature_prefix, &feature_ids_used);
        feature_ids_used.push(feature_id.clone());

        let title = format!("Onboarded decisions — {}", dir);
        let filename = parser::id_to_filename(&feature_id, &title);

        features.push(ProposedFeatureStub {
            id: feature_id,
            title,
            adr_ids,
            filename,
        });
    }

    features
}

/// Execute the seed phase: write ADR files to disk plus feature stubs.
pub fn execute_seed(
    seed_result: &SeedResult,
    adrs_dir: &Path,
    features_dir: &Path,
) -> Result<()> {
    // Write ADR files
    for adr in &seed_result.adrs {
        let path = adrs_dir.join(&adr.filename);
        let content = render_seeded_adr(adr);
        fileops::write_file_atomic(&path, &content)?;
        println!("  created {}", path.display());
    }

    // Write feature stubs
    for feature in &seed_result.features {
        let path = features_dir.join(&feature.filename);
        let content = render_feature_stub(feature);
        fileops::write_file_atomic(&path, &content)?;
        println!("  created {}", path.display());
    }

    Ok(())
}

/// Render an ADR file from a proposed ADR.
fn render_seeded_adr(adr: &ProposedAdr) -> String {
    let front = AdrFrontMatter {
        id: adr.id.clone(),
        title: adr.title.clone(),
        status: AdrStatus::Proposed,
        features: Vec::new(),
        supersedes: Vec::new(),
        superseded_by: Vec::new(),
        domains: Vec::new(),
        scope: AdrScope::FeatureSpecific,
        content_hash: None,
        amendments: vec![],
        source_files: vec![],
        removes: vec![],
        deprecates: vec![],
    };

    let mut body = String::new();

    // Context section
    body.push_str("## Context\n\n");
    body.push_str(&adr.observation);
    body.push('\n');
    if !adr.evidence.is_empty() {
        body.push_str("\n**Evidence:**\n");
        for ev in &adr.evidence {
            body.push_str(&format!("- `{}:{}` — {}\n", ev.file, ev.line, ev.snippet));
        }
    }
    body.push('\n');

    // Decision section
    body.push_str("## Decision\n\n");
    body.push_str(&adr.title);
    body.push_str(".\n\n");

    // Rationale
    body.push_str("## Rationale\n\n");
    body.push_str("<!-- TODO: add rationale -->\n\n");

    // Consequence
    body.push_str("## Consequence\n\n");
    body.push_str(&adr.hypothesised_consequence);
    body.push_str("\n\n");

    // Rejected alternatives
    body.push_str("**Rejected alternatives:**\n\n");
    body.push_str("<!-- TODO: add rejected alternatives -->\n");

    parser::render_adr(&front, &body)
}

/// Render a feature stub from a proposed feature.
fn render_feature_stub(feature: &ProposedFeatureStub) -> String {
    let front = FeatureFrontMatter {
        id: feature.id.clone(),
        title: feature.title.clone(),
        phase: 1,
        status: FeatureStatus::Planned,
        depends_on: Vec::new(),
        adrs: feature.adr_ids.clone(),
        tests: Vec::new(),
        domains: Vec::new(),
        domains_acknowledged: std::collections::HashMap::new(),
        bundle: None,
    };

    let body = format!(
        "Feature stub created by codebase onboarding.\n\nLinked ADRs: {}\n",
        feature.adr_ids.join(", ")
    );

    parser::render_feature(&front, &body)
}
