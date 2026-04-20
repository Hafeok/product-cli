//! Thin adapters for ADR conflict-check + conflict-bundle commands (FT-045, ADR-040).

use product_lib::{adr, error::ProductError};

use super::{load_graph_typed, BoxResult, CmdResult, Output};

pub fn adr_check_conflicts(id: Option<String>, _fmt: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let targets: Vec<String> = match id {
        Some(x) => vec![x],
        None => adr::conflicts::all_adr_ids(&graph),
    };
    let findings = adr::check_conflicts(&graph, &targets)?;
    let has_error = findings.iter().any(|f| f.code.is_error());
    let text = render_findings_text(&findings);
    let json = serde_json::to_value(&findings).unwrap_or(serde_json::Value::Null);

    if has_error {
        // Preserve the exit-code semantics of the legacy implementation by
        // printing the text output here and returning an error.
        println!("{}", text.trim_end_matches('\n'));
        return Err(ProductError::Internal(
            "structural conflict errors detected".to_string(),
        ));
    }
    Ok(Output::both(text, json))
}

/// Delegate to the existing `gap::conflict::bundle_for_adr` pure function.
/// Left on `BoxResult` for this phase — no behaviour change from legacy.
pub fn adr_conflict_bundle(id: &str, format: &str) -> BoxResult {
    let (_, root, graph) = super::load_graph()?;
    let markdown = product_lib::gap::conflict::bundle_for_adr(id, &graph, &root)
        .ok_or_else(|| ProductError::NotFound(format!("ADR {}", id)))?;
    if format == "json" {
        let v = serde_json::json!({ "bundle": markdown });
        println!("{}", serde_json::to_string_pretty(&v).unwrap_or_default());
    } else {
        print!("{}", markdown);
    }
    Ok(())
}

fn render_findings_text(findings: &[adr::ConflictFinding]) -> String {
    if findings.is_empty() {
        return "No structural conflicts detected.".to_string();
    }
    let mut out = String::new();
    for f in findings {
        out.push_str(&format!("{}: {} — {}\n", f.code.as_str(), f.adr, f.message));
    }
    out
}
