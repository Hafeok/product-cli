//! Aggregate bundle metrics summary — shared between `product graph stats`
//! and `product context --measure-all` (FT-040, ADR-024).
//!
//! Reads the `bundle` block written by `product context --measure` from each
//! feature's front-matter, then computes repository-wide statistics (mean,
//! median, p95, max, min) plus threshold breach counts.

use crate::config::ProductConfig;
use crate::graph::KnowledgeGraph;
use crate::metrics::ThresholdConfig;
use crate::types::{BundleMetrics, Feature};

/// Default thresholds when not configured in product.toml (see ADR-024).
pub const DEFAULT_BUNDLE_TOKENS_MAX: usize = 12_000;
pub const DEFAULT_BUNDLE_DEPTH1_ADR_MAX: usize = 8;
pub const DEFAULT_BUNDLE_DOMAINS_MAX: usize = 6;

/// One measured feature, for aggregation.
#[derive(Debug, Clone)]
pub struct MeasuredFeature {
    pub id: String,
    pub tokens_approx: usize,
    pub depth_1_adrs: usize,
    pub domains: usize,
}

impl MeasuredFeature {
    fn from_bundle(id: &str, b: &BundleMetrics) -> Self {
        Self {
            id: id.to_string(),
            tokens_approx: b.tokens_approx,
            depth_1_adrs: b.depth_1_adrs,
            domains: b.domains.len(),
        }
    }
}

/// Aggregate statistics derived from measured features.
#[derive(Debug, Clone)]
pub struct BundleSummary {
    pub total_features: usize,
    pub measured: Vec<MeasuredFeature>,
    pub unmeasured: Vec<String>,
    pub tokens_max_threshold: usize,
    pub adr_max_threshold: usize,
    pub domains_max_threshold: usize,
}

impl BundleSummary {
    pub fn is_empty(&self) -> bool {
        self.measured.is_empty()
    }

    /// Mean token count across measured features.
    pub fn mean_tokens(&self) -> usize {
        if self.measured.is_empty() {
            return 0;
        }
        let sum: usize = self.measured.iter().map(|m| m.tokens_approx).sum();
        sum / self.measured.len()
    }

    /// Median token count across measured features.
    pub fn median_tokens(&self) -> usize {
        if self.measured.is_empty() {
            return 0;
        }
        let mut values: Vec<usize> = self.measured.iter().map(|m| m.tokens_approx).collect();
        values.sort_unstable();
        let mid = values.len() / 2;
        if values.len().is_multiple_of(2) {
            (values[mid - 1] + values[mid]) / 2
        } else {
            values[mid]
        }
    }

    /// 95th percentile token count. For small samples this is the max.
    pub fn p95_tokens(&self) -> usize {
        if self.measured.is_empty() {
            return 0;
        }
        let mut values: Vec<usize> = self.measured.iter().map(|m| m.tokens_approx).collect();
        values.sort_unstable();
        // Simple nearest-rank percentile.
        let rank = ((0.95 * values.len() as f64).ceil() as usize).max(1);
        values[rank.min(values.len()) - 1]
    }

    /// Feature with highest token count, if any measurements exist.
    pub fn max_feature(&self) -> Option<&MeasuredFeature> {
        self.measured.iter().max_by_key(|m| m.tokens_approx)
    }

    /// Feature with lowest token count, if any measurements exist.
    pub fn min_feature(&self) -> Option<&MeasuredFeature> {
        self.measured.iter().min_by_key(|m| m.tokens_approx)
    }

    /// Features whose `tokens-approx` exceeds the configured ceiling.
    pub fn over_token_threshold(&self) -> Vec<&MeasuredFeature> {
        self.measured
            .iter()
            .filter(|m| m.tokens_approx > self.tokens_max_threshold)
            .collect()
    }

    /// Features whose `depth-1-adrs` exceeds the configured ceiling.
    pub fn over_adr_threshold(&self) -> Vec<&MeasuredFeature> {
        self.measured
            .iter()
            .filter(|m| m.depth_1_adrs > self.adr_max_threshold)
            .collect()
    }

    /// Features whose domain count exceeds the configured ceiling.
    pub fn over_domains_threshold(&self) -> Vec<&MeasuredFeature> {
        self.measured
            .iter()
            .filter(|m| m.domains > self.domains_max_threshold)
            .collect()
    }
}

/// Compute the bundle summary for the whole graph.
pub fn compute_summary(graph: &KnowledgeGraph, config: &ProductConfig) -> BundleSummary {
    // Collect features in ID order so max/min tie-breaks are deterministic.
    let mut features: Vec<&Feature> = graph.features.values().collect();
    features.sort_by(|a, b| a.front.id.cmp(&b.front.id));

    let mut measured: Vec<MeasuredFeature> = Vec::new();
    let mut unmeasured: Vec<String> = Vec::new();

    for f in &features {
        match &f.front.bundle {
            Some(b) => measured.push(MeasuredFeature::from_bundle(&f.front.id, b)),
            None => unmeasured.push(f.front.id.clone()),
        }
    }

    BundleSummary {
        total_features: features.len(),
        measured,
        unmeasured,
        tokens_max_threshold: threshold_usize(
            config,
            "bundle_tokens_max",
            DEFAULT_BUNDLE_TOKENS_MAX,
        ),
        adr_max_threshold: threshold_usize(
            config,
            "bundle_depth1_adr_max",
            DEFAULT_BUNDLE_DEPTH1_ADR_MAX,
        ),
        domains_max_threshold: threshold_usize(
            config,
            "bundle_domains_max",
            DEFAULT_BUNDLE_DOMAINS_MAX,
        ),
    }
}

fn threshold_usize(config: &ProductConfig, name: &str, default: usize) -> usize {
    config
        .metrics
        .as_ref()
        .and_then(|m| m.thresholds.get(name))
        .and_then(|t: &ThresholdConfig| t.max)
        .map(|v| v as usize)
        .unwrap_or(default)
}

/// Format a count with thousands separators (e.g. 12345 → "12,345").
fn fmt_thousands(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

/// Render the bundle summary table (used by both `graph stats` and
/// `context --measure-all`). Returns a text block with a trailing newline.
pub fn render_summary(summary: &BundleSummary) -> String {
    let mut out = String::new();
    out.push_str("Bundle size (tokens-approx):\n");
    if summary.is_empty() {
        out.push_str("  No bundle measurements \u{2014} run `product context --measure-all`\n");
        return out;
    }
    render_measured_line(summary, &mut out);
    render_token_stats(summary, &mut out);
    render_extremes(summary, &mut out);
    out.push('\n');
    render_thresholds(summary, &mut out);
    render_unmeasured(summary, &mut out);
    out
}

fn render_measured_line(summary: &BundleSummary, out: &mut String) {
    let u = summary.unmeasured.len();
    if u > 0 {
        out.push_str(&format!(
            "  measured:    {} / {} features  ({} unmeasured \u{2014} W012)\n",
            summary.measured.len(), summary.total_features, u
        ));
    } else {
        out.push_str(&format!(
            "  measured:    {} / {} features\n",
            summary.measured.len(), summary.total_features
        ));
    }
}

fn render_token_stats(summary: &BundleSummary, out: &mut String) {
    out.push_str(&format!("  mean:        {} tokens\n", fmt_thousands(summary.mean_tokens())));
    out.push_str(&format!("  median:      {} tokens\n", fmt_thousands(summary.median_tokens())));
    out.push_str(&format!("  p95:         {} tokens\n", fmt_thousands(summary.p95_tokens())));
}

fn render_extremes(summary: &BundleSummary, out: &mut String) {
    if let Some(m) = summary.max_feature() {
        out.push_str(&format!("  max:         {} tokens  {}\n", fmt_thousands(m.tokens_approx), m.id));
    }
    if let Some(m) = summary.min_feature() {
        out.push_str(&format!("  min:         {} tokens   {}\n", fmt_thousands(m.tokens_approx), m.id));
    }
}

fn render_thresholds(summary: &BundleSummary, out: &mut String) {
    let over_tokens = summary.over_token_threshold();
    out.push_str(&format!(
        "  Over token threshold (>{}):   {} features",
        fmt_thousands(summary.tokens_max_threshold),
        over_tokens.len()
    ));
    if !over_tokens.is_empty() {
        let ids: Vec<&str> = over_tokens.iter().map(|m| m.id.as_str()).collect();
        out.push_str(&format!("  \u{2014} {}", ids.join(", ")));
    }
    out.push('\n');
    let over_adrs = summary.over_adr_threshold();
    out.push_str(&format!(
        "  Over ADR threshold (>{} ADRs):     {} feature{}",
        summary.adr_max_threshold,
        over_adrs.len(),
        if over_adrs.len() == 1 { "" } else { "s" }
    ));
    if !over_adrs.is_empty() {
        let ids: Vec<&str> = over_adrs.iter().map(|m| m.id.as_str()).collect();
        out.push_str(&format!("  \u{2014} {}", ids.join(", ")));
    }
    out.push('\n');
}

fn render_unmeasured(summary: &BundleSummary, out: &mut String) {
    if summary.unmeasured.is_empty() {
        return;
    }
    let plural = if summary.unmeasured.len() == 1 { "" } else { "s" };
    out.push_str(&format!(
        "  Unmeasured:                       {} feature{}  \u{2014} {}\n",
        summary.unmeasured.len(),
        plural,
        summary.unmeasured.join(", ")
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mk_feature(id: &str, bundle: Option<BundleMetrics>) -> (String, BundleMetrics) {
        let b = bundle.unwrap_or(BundleMetrics {
            depth_1_adrs: 0,
            tcs: 0,
            domains: vec![],
            tokens_approx: 0,
            measured_at: "2026-01-01T00:00:00Z".into(),
        });
        (id.to_string(), b)
    }

    fn summary_from(measured: Vec<MeasuredFeature>, unmeasured: Vec<String>) -> BundleSummary {
        BundleSummary {
            total_features: measured.len() + unmeasured.len(),
            measured,
            unmeasured,
            tokens_max_threshold: DEFAULT_BUNDLE_TOKENS_MAX,
            adr_max_threshold: DEFAULT_BUNDLE_DEPTH1_ADR_MAX,
            domains_max_threshold: DEFAULT_BUNDLE_DOMAINS_MAX,
        }
    }

    #[test]
    fn empty_summary_renders_no_measurements() {
        let s = summary_from(vec![], vec!["FT-001".into(), "FT-002".into()]);
        let rendered = render_summary(&s);
        assert!(rendered.contains("No bundle measurements"));
        assert!(rendered.contains("product context --measure-all"));
    }

    #[test]
    fn summary_contains_core_metrics() {
        let s = summary_from(
            vec![
                MeasuredFeature { id: "FT-001".into(), tokens_approx: 2_100, depth_1_adrs: 1, domains: 1 },
                MeasuredFeature { id: "FT-002".into(), tokens_approx: 5_200, depth_1_adrs: 3, domains: 2 },
                MeasuredFeature { id: "FT-003".into(), tokens_approx: 11_200, depth_1_adrs: 9, domains: 5 },
            ],
            vec![],
        );
        let out = render_summary(&s);
        assert!(out.contains("Bundle size"));
        assert!(out.contains("measured:"));
        assert!(out.contains("mean:"));
        assert!(out.contains("median:"));
        assert!(out.contains("p95:"));
        assert!(out.contains("max:"));
        assert!(out.contains("FT-003"));
        assert!(out.contains("min:"));
        assert!(out.contains("FT-001"));
        assert!(out.contains("Over ADR threshold"));
    }

    #[test]
    fn mean_median_p95_computation() {
        let s = summary_from(
            vec![
                MeasuredFeature { id: "FT-001".into(), tokens_approx: 100, depth_1_adrs: 0, domains: 0 },
                MeasuredFeature { id: "FT-002".into(), tokens_approx: 200, depth_1_adrs: 0, domains: 0 },
                MeasuredFeature { id: "FT-003".into(), tokens_approx: 300, depth_1_adrs: 0, domains: 0 },
            ],
            vec![],
        );
        assert_eq!(s.mean_tokens(), 200);
        assert_eq!(s.median_tokens(), 200);
        assert_eq!(s.p95_tokens(), 300);
    }

    #[test]
    fn unmeasured_reported_with_ids() {
        let s = summary_from(
            vec![MeasuredFeature { id: "FT-001".into(), tokens_approx: 1_000, depth_1_adrs: 0, domains: 0 }],
            vec!["FT-002".into(), "FT-003".into()],
        );
        let out = render_summary(&s);
        assert!(out.contains("unmeasured"));
        assert!(out.contains("FT-002"));
        assert!(out.contains("FT-003"));
    }

    #[test]
    fn fmt_thousands_basic() {
        assert_eq!(fmt_thousands(0), "0");
        assert_eq!(fmt_thousands(999), "999");
        assert_eq!(fmt_thousands(1_000), "1,000");
        assert_eq!(fmt_thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn mk_feature_compiles_for_tests() {
        // Keep helper referenced to satisfy clippy::dead_code.
        let (id, b) = mk_feature("FT-001", None);
        assert_eq!(id, "FT-001");
        assert_eq!(b.tokens_approx, 0);
    }
}
