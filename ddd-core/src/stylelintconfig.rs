//! stylelint detection sources — configured rules plus emitted findings.
//!
//! Configured: `.stylelintrc.json` (`rules` map; `null` disables a rule,
//! secondary options may carry `severity`). Emitted: `stylelint -f json`
//! output — an array of per-file results whose `warnings` carry the rule
//! id, written to **stderr** since stylelint 16 (`2> out.json` is the
//! documented capture). Both reduce to the shared M2 shapes and join the
//! manifest machinery on the `stylelint` namespace.

use serde_json::Value;

use crate::configured::ConfiguredRule;

/// Parse a `.stylelintrc.json` rules map.
pub fn parse_config(text: &str, source: &str) -> Result<Vec<ConfiguredRule>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let Some(rules) = root.get("rules").and_then(Value::as_object) else {
        return Ok(out);
    };
    for (rule_id, config) in rules {
        let disabled = config.is_null() || matches!(config, Value::Bool(false));
        let severity = if disabled { "off".to_string() } else { configured_severity(config) };
        out.push(ConfiguredRule {
            rule_id: rule_id.clone(),
            severity,
            disabled,
            source: source.to_string(),
            scope: None,
        });
    }
    Ok(out)
}

/// stylelint's default severity is `error`; secondary options may override.
fn configured_severity(config: &Value) -> String {
    let secondary = config.as_array().and_then(|a| a.get(1));
    secondary
        .and_then(|s| s.get("severity"))
        .and_then(Value::as_str)
        .unwrap_or("error")
        .to_string()
}

/// Parse `stylelint -f json` output into emitted (rule id, level) pairs.
pub fn parse_output(text: &str) -> Result<Vec<(String, String)>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let files = root.as_array().ok_or("stylelint output is not an array")?;
    let mut out = Vec::new();
    for file in files {
        for w in file.get("warnings").and_then(Value::as_array).into_iter().flatten() {
            let rule = w.get("rule").and_then(Value::as_str).unwrap_or_default();
            let severity = w.get("severity").and_then(Value::as_str).unwrap_or("error");
            if !rule.is_empty() {
                out.push((rule.to_string(), severity.to_string()));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_reads_severity_disabled_and_defaults() {
        let rules = parse_config(
            r#"{"rules": {"color-no-hex": null, "declaration-property-value-disallowed-list": {"/color$/": ["/#/"]}, "max-nesting-depth": [2, {"severity": "warning"}]}}"#,
            ".stylelintrc.json",
        )
        .expect("parse");
        let by_id = |id: &str| rules.iter().find(|r| r.rule_id == id).expect(id);
        assert!(by_id("color-no-hex").disabled);
        assert_eq!(by_id("declaration-property-value-disallowed-list").severity, "error");
        assert_eq!(by_id("max-nesting-depth").severity, "warning");
    }

    #[test]
    fn output_reads_rule_ids_with_levels() {
        let out = parse_output(
            r#"[{"source":"style.css","warnings":[{"rule":"declaration-property-value-disallowed-list","severity":"error","text":"x"}]}]"#,
        )
        .expect("parse");
        assert_eq!(out, vec![("declaration-property-value-disallowed-list".to_string(), "error".to_string())]);
    }
}
