//! Minimal `Cargo.toml` reading: the `[workspace.lints.clippy]` table.
//!
//! The dogfood source (PRD §12): this workspace's own clippy gates are
//! decisions, so `ddd diff` needs to see them as configured rules. Reads
//! `workspace.lints.clippy` (falling back to package-level `lints.clippy`)
//! into `clippy::<name>` rule ids; a lint set to `allow` is explicitly
//! turned off — a config suppression. Lints inherited from clippy defaults
//! or CI flags never appear here; only what the manifest declares.

use crate::configured::ConfiguredRule;

/// Extract every clippy lint the manifest declares a level for.
pub fn parse(text: &str, source: &str) -> Result<Vec<ConfiguredRule>, String> {
    let doc: toml::Value = text.parse().map_err(|e| format!("not valid TOML: {e}"))?;
    let table = doc
        .get("workspace")
        .and_then(|w| w.get("lints"))
        .or_else(|| doc.get("lints"))
        .and_then(|l| l.get("clippy"))
        .and_then(|c| c.as_table());
    let Some(table) = table else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (name, value) in table {
        // Either `unwrap_used = "deny"` or `{ level = "deny", priority = 1 }`.
        let level = match value {
            toml::Value::String(s) => s.clone(),
            toml::Value::Table(t) => t
                .get("level")
                .and_then(|l| l.as_str())
                .unwrap_or("warn")
                .to_string(),
            _ => continue,
        };
        out.push(ConfiguredRule {
            rule_id: format!("clippy::{name}"),
            disabled: level == "allow",
            severity: level,
            source: source.to_string(),
            scope: None,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"
[workspace]
members = ["a"]

[workspace.lints.clippy]
unwrap_used = "deny"
expect_used = { level = "warn", priority = 1 }
needless_return = "allow"
"#;

    #[test]
    fn reads_workspace_clippy_lints_as_prefixed_rule_ids() {
        let mut rules = parse(SAMPLE, "Cargo.toml").expect("parse");
        rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
        assert_eq!(rules.len(), 3);
        assert_eq!(rules[1].rule_id, "clippy::needless_return");
        assert!(rules[1].disabled, "allow is a config suppression");
        assert_eq!(rules[2].rule_id, "clippy::unwrap_used");
        assert_eq!(rules[2].severity, "deny");
        assert_eq!(rules[0].severity, "warn");
    }

    #[test]
    fn package_level_lints_are_the_fallback() {
        let rules =
            parse("[lints.clippy]\ndbg_macro = \"deny\"\n", "Cargo.toml").expect("parse");
        assert_eq!(rules[0].rule_id, "clippy::dbg_macro");
    }

    #[test]
    fn a_manifest_without_lints_yields_nothing() {
        assert!(parse("[package]\nname = \"x\"\n", "Cargo.toml").expect("parse").is_empty());
    }
}
