//! MCP write handlers for ADR lifecycle operations (FT-046).
//!
//! Actually-writing implementations of `product_adr_amend` and
//! `product_adr_status` that bring MCP to parity with the CLI for every
//! lifecycle transition except `accepted` (ADR-032 sealing governance).

use crate::graph::KnowledgeGraph;
use serde_json::Value;

// ---------------------------------------------------------------------------
// product_adr_amend — optional atomic body replace + amendment entry
// ---------------------------------------------------------------------------

/// FT-046: handle `product_adr_amend` with optional `body` parameter.
///
/// - `body` omitted: legacy path — record an amendment against whatever body
///   is already on disk (must differ from the stored hash).
/// - `body` present: atomically replace the body, recompute the hash, and
///   append the amendment entry in one MCP call.
pub(crate) fn handle_adr_amend(args: &Value, graph: &KnowledgeGraph) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "id is required".to_string())?;
    let reason = args
        .get("reason")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "reason is required for amendments".to_string())?;

    reject_forbidden_fields(args)?;
    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| format!("ADR {} not found", id))?;
    if a.front.status != crate::types::AdrStatus::Accepted {
        return Err(format!(
            "E018: amendment-not-accepted \u{2014} {} has status '{}'; amendments only apply to accepted ADRs",
            id, a.front.status
        ));
    }

    let (effective_path, effective_body) = pick_effective_body(id, args, a)?;
    let staged = crate::types::Adr {
        front: a.front.clone(),
        body: effective_body.clone(),
        path: effective_path.clone(),
    };
    let (new_hash, amendment) = run_amend(id, reason, &staged)?;

    let mut front = a.front.clone();
    front.content_hash = Some(new_hash.clone());
    front.amendments.push(amendment);

    let content = crate::parser::render_adr(&front, &effective_body);
    crate::fileops::write_file_atomic(&effective_path, &content)
        .map_err(|e| format!("{}", e))?;

    Ok(build_amend_response(id, &front, &new_hash))
}

fn reject_forbidden_fields(args: &Value) -> Result<(), String> {
    if args.get("status").is_some() {
        return Err(
            "E019: amendment-carries-status \u{2014} status transitions go through product_adr_status (accepted requires CLI). Drop the 'status' field from this call."
                .to_string(),
        );
    }
    if args.get("amendments").is_some() {
        return Err(
            "E019: amendment-carries-status \u{2014} the 'amendments' audit trail is written by Product and cannot be supplied by the caller."
                .to_string(),
        );
    }
    Ok(())
}

fn pick_effective_body(
    id: &str,
    args: &Value,
    a: &crate::types::Adr,
) -> Result<(std::path::PathBuf, String), String> {
    match args.get("body").and_then(|v| v.as_str()) {
        Some(new_body) => {
            let candidate_hash = crate::hash::compute_adr_hash(&a.front.title, new_body);
            if let Some(ref stored) = a.front.content_hash {
                if *stored == candidate_hash {
                    return Err(format!(
                        "E017: amendment-nothing-changed \u{2014} supplied body matches stored content-hash for {}",
                        id
                    ));
                }
            }
            Ok((a.path.clone(), new_body.to_string()))
        }
        None => Ok((a.path.clone(), a.body.clone())),
    }
}

fn run_amend(
    id: &str,
    reason: &str,
    staged: &crate::types::Adr,
) -> Result<(String, crate::types::Amendment), String> {
    match crate::hash::amend_adr(staged, reason) {
        Ok(t) => Ok(t),
        Err(e) => {
            let msg = format!("{}", e);
            if msg.contains("nothing to amend") {
                Err(format!(
                    "E017: amendment-nothing-changed \u{2014} on-disk body for {} matches stored content-hash",
                    id
                ))
            } else {
                Err(msg)
            }
        }
    }
}

fn build_amend_response(
    id: &str,
    front: &crate::types::AdrFrontMatter,
    new_hash: &str,
) -> Value {
    let amendments_json: Vec<Value> = front
        .amendments
        .iter()
        .map(|am| {
            serde_json::json!({
                "date": am.date,
                "reason": am.reason,
                "previous-hash": am.previous_hash,
            })
        })
        .collect();
    serde_json::json!({
        "id": id,
        "status": front.status.to_string(),
        "content-hash": new_hash,
        "amendments": amendments_json,
    })
}

// ---------------------------------------------------------------------------
// product_adr_status — writes every non-accepted transition
// ---------------------------------------------------------------------------

/// FT-046: handle `product_adr_status` with real write-through behaviour.
pub(crate) fn handle_adr_status_write(
    args: &Value,
    graph: &KnowledgeGraph,
) -> Result<Value, String> {
    let id = args
        .get("id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "id is required".to_string())?;
    let status_str = args
        .get("status")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "status is required".to_string())?;
    let by = args.get("by").and_then(|v| v.as_str());

    let a = graph
        .adrs
        .get(id)
        .ok_or_else(|| format!("ADR {} not found", id))?;

    let new_status: crate::types::AdrStatus = status_str
        .parse()
        .map_err(|e: String| format!("E001: {}", e))?;

    // E020: accepting an ADR is CLI-only.
    if new_status == crate::types::AdrStatus::Accepted {
        return Err(format!(
            "E020: status-accepted-is-manual \u{2014} Accepting an ADR is a manual step. Run: product adr status {} accepted",
            id
        ));
    }

    // E021: accepted ADRs cannot demote to proposed.
    if a.front.status == crate::types::AdrStatus::Accepted
        && new_status == crate::types::AdrStatus::Proposed
    {
        return Err(format!(
            "E021: status-cannot-demote-accepted \u{2014} {} is accepted and cannot return to 'proposed'. Use supersede or abandon.",
            id
        ));
    }

    if new_status == crate::types::AdrStatus::Superseded {
        let target_id = by
            .ok_or_else(|| "superseded transition requires 'by' parameter".to_string())?;
        return do_supersede(id, target_id, graph);
    }

    // proposed / abandoned: plain status write.
    let mut front = a.front.clone();
    front.status = new_status;

    let content = crate::parser::render_adr(&front, &a.body);
    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;

    let mut result = serde_json::json!({
        "id": id,
        "status": new_status.to_string(),
    });
    if let Some(ref h) = front.content_hash {
        result["content-hash"] = serde_json::json!(h);
    }
    if !front.superseded_by.is_empty() {
        result["superseded-by"] = serde_json::json!(front.superseded_by);
    }
    Ok(result)
}

fn do_supersede(id: &str, target_id: &str, graph: &KnowledgeGraph) -> Result<Value, String> {
    let old = graph
        .adrs
        .get(id)
        .ok_or_else(|| format!("ADR {} not found", id))?;
    let new = graph
        .adrs
        .get(target_id)
        .ok_or_else(|| format!("E002: ADR {} not found", target_id))?;

    let mut old_front = old.front.clone();
    old_front.status = crate::types::AdrStatus::Superseded;
    if !old_front.superseded_by.contains(&target_id.to_string()) {
        old_front.superseded_by.push(target_id.to_string());
    }

    let mut new_front = new.front.clone();
    if !new_front.supersedes.contains(&id.to_string()) {
        new_front.supersedes.push(id.to_string());
    }

    check_supersession_cycle(graph, id, target_id, old, new, &old_front, &new_front)?;

    let content_old = crate::parser::render_adr(&old_front, &old.body);
    let content_new = crate::parser::render_adr(&new_front, &new.body);
    let writes: Vec<(&std::path::Path, &str)> = vec![
        (&old.path, &content_old),
        (&new.path, &content_new),
    ];
    crate::fileops::write_batch_atomic(&writes).map_err(|e| format!("{}", e))?;

    let mut result = serde_json::json!({
        "id": id,
        "status": "superseded",
        "superseded-by": old_front.superseded_by,
    });
    if let Some(ref h) = old_front.content_hash {
        result["content-hash"] = serde_json::json!(h);
    }
    Ok(result)
}

fn check_supersession_cycle(
    graph: &KnowledgeGraph,
    id: &str,
    target_id: &str,
    old: &crate::types::Adr,
    new: &crate::types::Adr,
    old_front: &crate::types::AdrFrontMatter,
    new_front: &crate::types::AdrFrontMatter,
) -> Result<(), String> {
    let mut test_adrs: Vec<crate::types::Adr> = graph.adrs.values().cloned().collect();
    test_adrs.retain(|ai| ai.front.id != id && ai.front.id != target_id);
    test_adrs.push(crate::types::Adr {
        front: old_front.clone(),
        body: old.body.clone(),
        path: old.path.clone(),
    });
    test_adrs.push(crate::types::Adr {
        front: new_front.clone(),
        body: new.body.clone(),
        path: new.path.clone(),
    });
    let test_graph = KnowledgeGraph::build(vec![], test_adrs, vec![]);
    if let Some(cycle) = test_graph.detect_supersession_cycle() {
        return Err(format!(
            "E004: supersession cycle detected: {}",
            cycle.join(" -> ")
        ));
    }
    Ok(())
}
