//! End-to-end tests: bridged dispatch through the phase-gated workflow
//! transport, exactly as the SDK host wires it.

use std::sync::Arc;

use product_core::pf::workflow::{Phase, WorkflowSession};
use serde_json::json;

use super::{bridge, full_tool_list, workflow_rpc_handler};
use crate::registry::ToolRegistry;
use github_copilot_sdk::types::ToolResult;

fn text_of(result: &ToolResult) -> String {
    match result {
        ToolResult::Text(t) => t.clone(),
        ToolResult::Expanded(e) => e.text_result_for_llm.clone(),
        _ => panic!("unexpected tool result shape"),
    }
}

fn is_failure(result: &ToolResult) -> bool {
    matches!(result, ToolResult::Expanded(e) if e.result_type == "failure")
}

/// A scaffolded workflow session in a temp repo, plus the RPC handler the
/// SDK host would build for it.
fn harness() -> (tempfile::TempDir, bridge::RpcHandler) {
    let tmp = tempfile::tempdir().expect("tempdir");
    let session_root = tmp.path().join(".product").join("sessions").join("demo-1");
    let session = WorkflowSession::new("demo-1", "demo", "copilot", Phase::Build, "t0".into());
    session.save(&session_root).expect("save session");
    let registry = Arc::new(ToolRegistry::new(tmp.path().to_path_buf(), true));
    let handler = workflow_rpc_handler(registry, "demo-1", tmp.path());
    (tmp, handler)
}

#[test]
fn full_tool_list_covers_every_family_plus_the_controls() {
    let registry = ToolRegistry::new(std::env::temp_dir(), true);
    let names: Vec<String> =
        full_tool_list(&registry).iter().map(|t| t.name.clone()).collect();
    // One representative per phase family…
    assert!(names.iter().any(|n| n == "product_domain_new"), "What family present");
    assert!(names.iter().any(|n| n == "product_how_add"), "How family present");
    assert!(names.iter().any(|n| n == "product_build_run"), "Build family present");
    // …and the always-visible session controls.
    for control in ["product_workflow_status", "product_workflow_advance", "product_session_finalize"] {
        assert!(names.iter().any(|n| n == control), "{control} present");
    }
}

#[test]
fn workflow_status_flows_back_through_the_bridge() {
    let (_tmp, handler) = harness();
    let result = bridge::dispatch(&handler, "product_workflow_status", json!({}), "c1");
    assert!(!is_failure(&result), "status should succeed: {}", text_of(&result));
    let text = text_of(&result);
    assert!(text.contains("what"), "reports the current phase: {text}");
}

#[test]
fn out_of_phase_writes_are_rejected_with_the_gate_message() {
    let (_tmp, handler) = harness();
    // A Build-phase write while the session is still in What.
    let result = bridge::dispatch(
        &handler,
        "product_feature_new",
        json!({"id": "f1", "anchors": ["x"]}),
        "c2",
    );
    assert!(is_failure(&result), "gate must reject");
    let text = text_of(&result);
    assert!(text.contains("phase"), "explains the phase lock: {text}");
    assert!(text.contains("product_workflow_advance"), "points at the advance tool: {text}");
}

#[test]
fn phase_advance_unlocks_the_next_family() {
    let (_tmp, handler) = harness();
    let advanced = bridge::dispatch(&handler, "product_workflow_advance", json!({}), "c3");
    assert!(!is_failure(&advanced), "advance should succeed: {}", text_of(&advanced));
    assert!(text_of(&advanced).contains("how"));
    // A How write is now in phase (it may still fail domain validation —
    // what matters is that the phase gate no longer rejects it).
    let result = bridge::dispatch(&handler, "product_how_init", json!({}), "c4");
    let text = text_of(&result);
    assert!(!text.contains("belongs to the"), "no phase-gate rejection: {text}");
}
