//! Minimal `bicepconfig.json` reading: the `analyzers.core.rules` levels.
//!
//! Extracts each linter rule's configured `level` (`off | info | warning |
//! error`) — `off` is a config suppression. Everything else in the file
//! (module aliases, formatting, experimental features) is ignored; rules
//! the linter enables by default with no config line surface via the
//! emitted (SARIF) source only.

use crate::configured::ConfiguredRule;

/// Extract every `analyzers.core.rules.<name>.level` entry.
pub fn parse(text: &str, source: &str) -> Result<Vec<ConfiguredRule>, String> {
    let doc: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("not valid JSON: {e}"))?;
    let Some(rules) = doc.pointer("/analyzers/core/rules").and_then(|r| r.as_object()) else {
        return Ok(Vec::new());
    };
    let mut out = Vec::new();
    for (name, rule) in rules {
        let level = rule
            .get("level")
            .and_then(|l| l.as_str())
            .unwrap_or("warning")
            .to_ascii_lowercase();
        out.push(ConfiguredRule {
            rule_id: name.clone(),
            disabled: level == "off",
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

    const SAMPLE: &str = r#"{
      "analyzers": {
        "core": {
          "enabled": true,
          "rules": {
            "no-unused-params": {"level": "warning"},
            "no-hardcoded-env-urls": {"level": "off"},
            "secure-parameter-default": {"level": "error"}
          }
        }
      }
    }"#;

    #[test]
    fn extracts_rule_levels_with_off_as_suppression() {
        let mut rules = parse(SAMPLE, "bicepconfig.json").expect("parse");
        rules.sort_by(|a, b| a.rule_id.cmp(&b.rule_id));
        assert_eq!(rules.len(), 3);
        assert!(rules[0].disabled, "level off is a config suppression");
        assert_eq!(rules[1].rule_id, "no-unused-params");
        assert_eq!(rules[1].severity, "warning");
        assert_eq!(rules[2].severity, "error");
    }

    #[test]
    fn a_config_without_linter_rules_yields_nothing() {
        assert!(parse(r#"{"moduleAliases": {}}"#, "x").expect("parse").is_empty());
    }

    #[test]
    fn invalid_json_is_an_error() {
        assert!(parse("{not json", "x").is_err());
    }
}
