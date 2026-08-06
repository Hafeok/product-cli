//! Minimal `.editorconfig` reading: `dotnet_diagnostic` severity lines only.
//!
//! Deliberately not an editorconfig engine (PRD §7): it tracks `[section]`
//! headers so each line's scope can be *recorded*, and extracts
//! `dotnet_diagnostic.<ID>.severity = <value>` pairs — nothing else. Section
//! globs are never evaluated, `root`/`unset` semantics are ignored, and
//! rules enabled by analyzer-package defaults (no config line) do not appear
//! here at all — they surface via the emitted (SARIF) source only.

use crate::configured::ConfiguredRule;

/// Severities that turn a rule off entirely — config suppressions.
/// (`silent` still runs the rule without surfacing it; not a suppression.)
const DISABLING: &[&str] = &["none"];

/// Extract every `dotnet_diagnostic.<ID>.severity` line.
pub fn parse(text: &str, source: &str) -> Vec<ConfiguredRule> {
    let mut out = Vec::new();
    let mut section: Option<String> = None;
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if line.starts_with('[') && line.ends_with(']') {
            section = Some(line[1..line.len() - 1].to_string());
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let (key, value) = (key.trim(), value.trim());
        // editorconfig keys are case-insensitive; the rule id keeps its case.
        let lower = key.to_ascii_lowercase();
        let Some(rest) = lower.strip_prefix("dotnet_diagnostic.") else {
            continue;
        };
        if !rest.ends_with(".severity") {
            continue;
        }
        let id_len = rest.len() - ".severity".len();
        let rule_id = key["dotnet_diagnostic.".len()..][..id_len].to_string();
        if rule_id.is_empty() {
            continue;
        }
        let severity = value.to_ascii_lowercase();
        out.push(ConfiguredRule {
            rule_id,
            disabled: DISABLING.contains(&severity.as_str()),
            severity,
            source: source.to_string(),
            scope: section.clone(),
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "\
root = true

[*.cs]
# analyzer severities
dotnet_diagnostic.CA2007.severity = warning
dotnet_diagnostic.CA1303.severity = none
DOTNET_DIAGNOSTIC.IDE0005.SEVERITY = Error
csharp_style_var_elsewhere = true

[*.{vb}]
dotnet_diagnostic.CA2007.severity = silent
";

    #[test]
    fn extracts_severity_lines_with_their_section() {
        let rules = parse(SAMPLE, ".editorconfig");
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].rule_id, "CA2007");
        assert_eq!(rules[0].severity, "warning");
        assert_eq!(rules[0].scope.as_deref(), Some("*.cs"));
        assert!(!rules[0].disabled);
        assert!(rules[1].disabled, "severity none is a config suppression");
        // Case-insensitive key, rule id case preserved as written.
        assert_eq!(rules[2].rule_id, "IDE0005");
        assert_eq!(rules[2].severity, "error");
        // silent is not a suppression; the vb section is still recorded.
        assert!(!rules[3].disabled);
        assert_eq!(rules[3].scope.as_deref(), Some("*.{vb}"));
    }

    #[test]
    fn non_diagnostic_lines_are_ignored() {
        assert!(parse("indent_size = 4\nroot = true\n", "x").is_empty());
    }
}
