//! The configured-rule shape shared by the per-format config parsers.
//!
//! `editorconfig`, `bicepconfig`, plus `cargolints` each reduce their format
//! to this one record: a rule id, the configured severity, whether that
//! severity disables the rule (a *config suppression* — it must cite a
//! risk-acceptance record), and where it was configured.

/// One rule as a config file declares it.
#[derive(Debug, Clone)]
pub struct ConfiguredRule {
    /// The diagnostic id as the toolchain knows it (`CA2007`,
    /// `no-unused-params`, `clippy::unwrap_used`).
    pub rule_id: String,
    /// The configured severity, verbatim from the file.
    pub severity: String,
    /// True when the severity turns the rule off — a config suppression.
    pub disabled: bool,
    /// Repo-relative path of the config file that declared it.
    pub source: String,
    /// The `.editorconfig` section glob the line sits under; recorded for
    /// the report, never evaluated (documented parser limit).
    pub scope: Option<String>,
}
