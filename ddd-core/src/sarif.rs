//! SARIF 2.1 ingestion — the one place emitted diagnostics are parsed.
//!
//! Detection is unified on SARIF (PRD §7): per-language adapters only know
//! how to *produce* a SARIF file (C#: the compiler `ErrorLog` property with
//! `version=2.1`; Bicep: `bicep lint --diagnostics-format sarif`) and which
//! rule-id namespace it carries — routing lives in `detect`. This module
//! reads any SARIF 2.1 log into runs of emitted diagnostics, keeping each
//! result's suppression state (how `#pragma`-style source suppressions
//! surface: the compiler still logs the result, marked suppressed).

/// One `runs[]` entry: the producing tool plus its results.
#[derive(Debug, Clone)]
pub struct SarifRun {
    /// `tool.driver.name` — e.g. `csc` or `bicep`.
    pub driver: String,
    pub results: Vec<EmittedDiagnostic>,
}

/// One `results[]` entry, reduced to what governance diffing needs.
#[derive(Debug, Clone)]
pub struct EmittedDiagnostic {
    /// The diagnostic id as the toolchain reports it (`CA2007`, `no-unused-params`).
    pub rule_id: String,
    /// The result's `level` (`error | warning | note`); SARIF defaults to `warning`.
    pub level: String,
    /// True when the result carries a non-empty `suppressions` array —
    /// an in-source or in-config suppression the tool honoured but logged.
    pub suppressed: bool,
}

/// Parse a SARIF document into its runs; errors name what was wrong.
pub fn parse(text: &str) -> Result<Vec<SarifRun>, String> {
    let doc: serde_json::Value =
        serde_json::from_str(text).map_err(|e| format!("not valid JSON: {e}"))?;
    let version = doc.get("version").and_then(|v| v.as_str()).unwrap_or("");
    if !version.starts_with("2.") {
        return Err(format!(
            "SARIF version '{version}' is not 2.1 — for C# pass version=2.1 in the ErrorLog property (the compiler defaults to SARIF 1.0)"
        ));
    }
    let runs = doc
        .get("runs")
        .and_then(|r| r.as_array())
        .ok_or_else(|| "no runs[] array".to_string())?;
    Ok(runs.iter().map(parse_run).collect())
}

fn parse_run(run: &serde_json::Value) -> SarifRun {
    let driver = run
        .pointer("/tool/driver/name")
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let results = run
        .get("results")
        .and_then(|r| r.as_array())
        .map(|rs| rs.iter().filter_map(parse_result).collect())
        .unwrap_or_default();
    SarifRun { driver, results }
}

/// A result without a `ruleId` cannot join the manifest — skip it.
fn parse_result(result: &serde_json::Value) -> Option<EmittedDiagnostic> {
    let rule_id = result.get("ruleId").and_then(|r| r.as_str())?.to_string();
    let level =
        result.get("level").and_then(|l| l.as_str()).unwrap_or("warning").to_string();
    let suppressed = result
        .get("suppressions")
        .and_then(|s| s.as_array())
        .map(|s| !s.is_empty())
        .unwrap_or(false);
    Some(EmittedDiagnostic { rule_id, level, suppressed })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROSLYN_SHAPED: &str = r#"{
      "$schema": "http://json.schemastore.org/sarif-2.1.0",
      "version": "2.1.0",
      "runs": [{
        "tool": {"driver": {"name": "csc", "rules": [{"id": "CA2007"}]}},
        "results": [
          {"ruleId": "CA2007", "level": "warning", "message": {"text": "m"}},
          {"ruleId": "CA2000", "message": {"text": "m"},
           "suppressions": [{"kind": "inSource"}]},
          {"ruleId": "CS8618", "level": "error", "message": {"text": "m"}}
        ]
      }]
    }"#;

    #[test]
    fn reads_rule_ids_levels_plus_suppression_state() {
        let runs = parse(ROSLYN_SHAPED).expect("parse");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].driver, "csc");
        let r = &runs[0].results;
        assert_eq!(r[0].rule_id, "CA2007");
        assert!(!r[0].suppressed);
        // No level on the suppressed result — SARIF's default applies.
        assert_eq!(r[1].level, "warning");
        assert!(r[1].suppressed);
        assert_eq!(r[2].level, "error");
    }

    #[test]
    fn a_bicep_shaped_run_parses_by_the_same_path() {
        let text = r#"{"version": "2.1.0", "runs": [{
            "tool": {"driver": {"name": "bicep"}},
            "results": [{"ruleId": "no-unused-params", "level": "warning"}]
        }]}"#;
        let runs = parse(text).expect("parse");
        assert_eq!(runs[0].driver, "bicep");
        assert_eq!(runs[0].results[0].rule_id, "no-unused-params");
    }

    #[test]
    fn sarif_one_is_rejected_naming_the_fix() {
        let err = parse(r#"{"version": "1.0.0", "runs": []}"#).expect_err("v1");
        assert!(err.contains("version=2.1"), "{err}");
    }

    #[test]
    fn non_json_is_rejected() {
        assert!(parse("warning CA2007: blah").is_err());
    }

    #[test]
    fn results_without_a_rule_id_are_skipped() {
        let text = r#"{"version": "2.1.0", "runs": [{
            "tool": {"driver": {"name": "csc"}},
            "results": [{"level": "error"}, {"ruleId": "CA2007"}]
        }]}"#;
        assert_eq!(parse(text).expect("parse")[0].results.len(), 1);
    }
}
