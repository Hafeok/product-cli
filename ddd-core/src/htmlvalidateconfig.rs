//! html-validate detection sources — configured rules plus emitted findings.
//!
//! Configured: `.htmlvalidate.json` (`rules` map; `"off"`/`0` disables,
//! `[severity, options]` arrays carry the severity first). Emitted:
//! `html-validate -f json` output — per-file results whose `messages`
//! carry `ruleId` plus a numeric severity (2 error, 1 warning). Both
//! reduce to the shared M2 shapes and join the manifest machinery on the
//! `htmlvalidate` namespace.

use serde_json::Value;

use crate::configured::ConfiguredRule;

/// Parse a `.htmlvalidate.json` rules map.
pub fn parse_config(text: &str, source: &str) -> Result<Vec<ConfiguredRule>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    let Some(rules) = root.get("rules").and_then(Value::as_object) else {
        return Ok(out);
    };
    for (rule_id, config) in rules {
        let severity = configured_severity(config);
        out.push(ConfiguredRule {
            rule_id: rule_id.clone(),
            disabled: severity == "off",
            severity,
            source: source.to_string(),
            scope: None,
        });
    }
    Ok(out)
}

fn configured_severity(config: &Value) -> String {
    let head = config.as_array().and_then(|a| a.first()).unwrap_or(config);
    match head {
        Value::String(s) => s.clone(),
        Value::Number(n) => match n.as_u64() {
            Some(2) => "error".to_string(),
            Some(1) => "warn".to_string(),
            _ => "off".to_string(),
        },
        _ => "error".to_string(),
    }
}

/// Parse `html-validate -f json` output into emitted (rule id, level) pairs.
pub fn parse_output(text: &str) -> Result<Vec<(String, String)>, String> {
    let root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let files = root.as_array().ok_or("html-validate output is not an array")?;
    let mut out = Vec::new();
    for file in files {
        for m in file.get("messages").and_then(Value::as_array).into_iter().flatten() {
            let rule = m.get("ruleId").and_then(Value::as_str).unwrap_or_default();
            let level = match m.get("severity").and_then(Value::as_u64) {
                Some(2) => "error",
                Some(1) => "warning",
                _ => "note",
            };
            if !rule.is_empty() {
                out.push((rule.to_string(), level.to_string()));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_reads_string_number_and_array_severities() {
        let rules = parse_config(
            r#"{"rules": {"no-dup-class": "error", "no-inline-style": ["warn", {}], "void-style": 0}}"#,
            ".htmlvalidate.json",
        )
        .expect("parse");
        let by_id = |id: &str| rules.iter().find(|r| r.rule_id == id).expect(id);
        assert_eq!(by_id("no-dup-class").severity, "error");
        assert_eq!(by_id("no-inline-style").severity, "warn");
        assert!(by_id("void-style").disabled);
    }

    #[test]
    fn output_maps_numeric_severities() {
        let out = parse_output(
            r#"[{"filePath":"page.html","messages":[{"ruleId":"no-dup-class","severity":2,"message":"x"}]}]"#,
        )
        .expect("parse");
        assert_eq!(out, vec![("no-dup-class".to_string(), "error".to_string())]);
    }
}
