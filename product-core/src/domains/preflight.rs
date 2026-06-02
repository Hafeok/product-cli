//! Pre-flight analysis for concern domain coverage (ADR-025, ADR-026)

use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use crate::types::*;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Preflight types
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct PreflightResult {
    pub feature_id: String,
    pub feature_domains: Vec<String>,
    pub cross_cutting_gaps: Vec<CrossCuttingGap>,
    pub domain_gaps: Vec<DomainGap>,
    /// FT-067: platform-scoped ADRs surfaced as informational only. They never
    /// contribute to the gap count or affect the exit code; this list is
    /// rendered as a *Platform Invariants* section so authors can still see
    /// which platform invariants exist for the repo.
    pub platform_invariants: Vec<PlatformInvariant>,
    pub is_clean: bool,
}

#[derive(Debug)]
pub struct CrossCuttingGap {
    pub adr_id: String,
    pub adr_title: String,
    pub adr_domains: Vec<String>,
    pub status: CoverageStatus,
}

/// FT-067: informational record for a platform-scoped ADR. Carries no
/// `status` enum — the status is always "informational, enforced by the
/// platform itself."
#[derive(Debug)]
pub struct PlatformInvariant {
    pub adr_id: String,
    pub adr_title: String,
    pub adr_domains: Vec<String>,
}

#[derive(Debug)]
pub struct DomainGap {
    pub domain: String,
    pub adr_count: usize,
    pub status: CoverageStatus,
    pub top_adrs: Vec<(String, String)>, // (id, title)
}

#[derive(Debug, Clone, PartialEq)]
pub enum CoverageStatus {
    Linked,
    Acknowledged(String), // reason
    DefaultAcknowledged,  // FT-104: from config default-acknowledged-cross-cutting
    Rejected(String),     // FT-104: from feature adrs-rejected with reason
    Gap,
}

impl std::fmt::Display for CoverageStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linked => write!(f, "linked"),
            Self::Acknowledged(r) => write!(f, "acknowledged: {}", r),
            Self::DefaultAcknowledged => write!(f, "default-acknowledged"),
            Self::Rejected(r) => write!(f, "intentional: {}", r),
            Self::Gap => write!(f, "gap"),
        }
    }
}

/// Run preflight analysis on a feature
pub fn preflight(
    graph: &KnowledgeGraph,
    feature_id: &str,
    _domain_vocab: &HashMap<String, String>,
    default_acknowledged: &[String],
) -> std::result::Result<PreflightResult, ProductError> {
    let feature = graph.features.get(feature_id).ok_or_else(|| {
        ProductError::NotFound(format!("feature {}", feature_id))
    })?;

    let mut cross_cutting_gaps = Vec::new();
    let mut domain_gaps = Vec::new();
    let mut platform_invariants = Vec::new();

    // Check all cross-cutting ADRs (per-feature attention required) and collect
    // platform ADRs into the informational section (FT-067).
    for adr in graph.adrs.values() {
        match adr.front.scope {
            AdrScope::CrossCutting => {
                let status = if feature.front.adrs.contains(&adr.front.id) {
                    CoverageStatus::Linked
                } else if let Some(rejection) = feature.front.adrs_rejected.iter().find(|r| r.id == adr.front.id) {
                    // FT-104: explicit rejection with reason
                    CoverageStatus::Rejected(rejection.reason.clone())
                } else if default_acknowledged.contains(&adr.front.id) {
                    // FT-104: default-acknowledged from config
                    CoverageStatus::DefaultAcknowledged
                } else if let Some(reason) =
                    find_acknowledgement(feature, &adr.front.id, &adr.front.domains)
                {
                    CoverageStatus::Acknowledged(reason)
                } else {
                    CoverageStatus::Gap
                };
                cross_cutting_gaps.push(CrossCuttingGap {
                    adr_id: adr.front.id.clone(),
                    adr_title: adr.front.title.clone(),
                    adr_domains: adr.front.domains.clone(),
                    status,
                });
            }
            AdrScope::Platform => {
                platform_invariants.push(PlatformInvariant {
                    adr_id: adr.front.id.clone(),
                    adr_title: adr.front.title.clone(),
                    adr_domains: adr.front.domains.clone(),
                });
            }
            _ => {}
        }
    }

    // Check domain coverage for each domain the feature declares. Both
    // CrossCutting and Platform scopes are excluded from the per-domain gap
    // computation — Platform ADRs are enforced project-wide, not per-feature,
    // and CrossCutting ADRs are already reported separately.
    let centrality = graph.betweenness_centrality();
    for domain in &feature.front.domains {
        let domain_adrs: Vec<&Adr> = graph.adrs.values()
            .filter(|a| a.front.domains.contains(domain) && !a.front.scope.is_platform_wide())
            .collect();

        if domain_adrs.is_empty() {
            continue; // No ADRs for this domain — not applicable
        }

        let any_linked = domain_adrs.iter().any(|a| feature.front.adrs.contains(&a.front.id));
        let acknowledged = feature.front.domains_acknowledged.get(domain);

        let status = if any_linked {
            CoverageStatus::Linked
        } else if let Some(reason) = acknowledged {
            CoverageStatus::Acknowledged(reason.clone())
        } else {
            CoverageStatus::Gap
        };

        // Top-2 ADRs by centrality for this domain
        let mut ranked: Vec<_> = domain_adrs.iter()
            .map(|a| (a.front.id.clone(), a.front.title.clone(), centrality.get(&a.front.id).copied().unwrap_or(0.0)))
            .collect();
        ranked.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
        let top_adrs: Vec<(String, String)> = ranked.into_iter().take(2).map(|(id, title, _)| (id, title)).collect();

        domain_gaps.push(DomainGap {
            domain: domain.clone(),
            adr_count: domain_adrs.len(),
            status,
            top_adrs,
        });
    }

    let is_clean = cross_cutting_gaps.iter().all(|g| !matches!(g.status, CoverageStatus::Gap | CoverageStatus::Rejected(_)))
        && domain_gaps.iter().all(|g| !matches!(g.status, CoverageStatus::Gap));

    // Platform invariants are stable ordering for deterministic output.
    platform_invariants.sort_by(|a, b| a.adr_id.cmp(&b.adr_id));

    Ok(PreflightResult {
        feature_id: feature_id.to_string(),
        feature_domains: feature.front.domains.clone(),
        cross_cutting_gaps,
        domain_gaps,
        platform_invariants,
        is_clean,
    })
}

pub(crate) fn find_acknowledgement(feature: &Feature, adr_id: &str, adr_domains: &[String]) -> Option<String> {
    // Check if any of the ADR's domains are acknowledged by the feature
    for domain in adr_domains {
        if let Some(reason) = feature.front.domains_acknowledged.get(domain) {
            if !reason.trim().is_empty() {
                return Some(reason.clone());
            }
        }
    }
    // Also check direct ADR acknowledgement (stored as adr ID key)
    feature.front.domains_acknowledged.get(adr_id)
        .filter(|r| !r.trim().is_empty())
        .cloned()
}

/// Add a domain acknowledgement to a feature's front-matter
pub fn acknowledge_domain(
    feature: &Feature,
    domain: &str,
    reason: &str,
) -> Result<FeatureFrontMatter> {
    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "E011: acknowledgement requires a non-empty reason".to_string(),
        ));
    }
    let mut front = feature.front.clone();
    front.domains_acknowledged.insert(domain.to_string(), reason.to_string());
    Ok(front)
}

/// Add an ADR acknowledgement (stored under the ADR's domains)
pub fn acknowledge_adr(
    feature: &Feature,
    adr: &Adr,
    reason: &str,
) -> Result<FeatureFrontMatter> {
    if reason.trim().is_empty() {
        return Err(ProductError::ConfigError(
            "E011: acknowledgement requires a non-empty reason".to_string(),
        ));
    }
    let mut front = feature.front.clone();
    // Store under the ADR ID as key
    front.domains_acknowledged.insert(adr.front.id.clone(), reason.to_string());
    Ok(front)
}

/// Render a preflight result report
pub fn render_preflight(result: &PreflightResult) -> String {
    let mut out = String::new();
    out.push_str(&format!("Pre-flight analysis: {}\n", result.feature_id));
    if !result.feature_domains.is_empty() {
        out.push_str(&format!("Feature domains: {}\n", result.feature_domains.join(", ")));
    }
    out.push('\n');

    render_cross_cutting_section(&mut out, &result.cross_cutting_gaps);
    render_platform_invariants_section(&mut out, &result.platform_invariants);
    render_domain_section(&mut out, &result.domain_gaps);
    render_summary_line(&mut out, result);

    out
}

/// FT-067: render the informational *Platform Invariants* section. One line
/// per ADR, no status symbol, no failure semantics.
fn render_platform_invariants_section(out: &mut String, invariants: &[PlatformInvariant]) {
    if invariants.is_empty() {
        return;
    }
    out.push_str("Platform Invariants:\n");
    for inv in invariants {
        out.push_str(&format!(
            "    \u{2022}  {:<10} {}\n",
            inv.adr_id, inv.adr_title
        ));
    }
    out.push('\n');
}

/// Render the cross-cutting ADRs section.
fn render_cross_cutting_section(out: &mut String, gaps: &[CrossCuttingGap]) {
    if gaps.is_empty() {
        return;
    }
    out.push_str("Cross-Cutting ADRs:\n");
    for gap in gaps {
        let symbol = match &gap.status {
            CoverageStatus::Linked => "\u{2713}",
            CoverageStatus::Acknowledged(_) => "~",
            CoverageStatus::DefaultAcknowledged => "\u{2713}",
            CoverageStatus::Rejected(_) => "\u{2717}",
            CoverageStatus::Gap => "\u{2717}",
        };
        let label = match &gap.status {
            CoverageStatus::Linked => "linked".to_string(),
            CoverageStatus::Acknowledged(r) => format!("acknowledged: {}", r.chars().take(30).collect::<String>()),
            CoverageStatus::DefaultAcknowledged => "default-acknowledged".to_string(),
            CoverageStatus::Rejected(r) => format!("INTENTIONAL: {}", r.chars().take(30).collect::<String>()),
            CoverageStatus::Gap => "NOT COVERED".to_string(),
        };
        out.push_str(&format!("  {}  {:<10} {:<40} [{}]\n", symbol, gap.adr_id, gap.adr_title, label));
    }
    out.push('\n');
}

/// Render the domain coverage section.
fn render_domain_section(out: &mut String, gaps: &[DomainGap]) {
    if gaps.is_empty() {
        return;
    }
    out.push_str("Domain Coverage:\n");
    for gap in gaps {
        let symbol = match &gap.status {
            CoverageStatus::Linked => "\u{2713}",
            CoverageStatus::Acknowledged(_) => "~",
            CoverageStatus::DefaultAcknowledged => "\u{2713}",
            CoverageStatus::Rejected(_) => "\u{2717}",
            CoverageStatus::Gap => "\u{2717}",
        };
        let adrs_str = gap.top_adrs.iter()
            .map(|(id, _)| id.as_str())
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("  {}  {:<15} {} ADR(s) — top: {}\n", symbol, gap.domain, gap.adr_count, adrs_str));
    }
    out.push('\n');
}

/// Render the final summary line (CLEAN or gap counts).
fn render_summary_line(out: &mut String, result: &PreflightResult) {
    if result.is_clean {
        out.push_str("Pre-flight: CLEAN\n");
    } else {
        let cc_gaps = result.cross_cutting_gaps.iter().filter(|g| matches!(g.status, CoverageStatus::Gap | CoverageStatus::Rejected(_))).count();
        let d_gaps = result.domain_gaps.iter().filter(|g| g.status == CoverageStatus::Gap).count();
        out.push_str(&format!("Pre-flight: {} cross-cutting gap(s), {} domain gap(s)\n", cc_gaps, d_gaps));
    }
}
