//! FT-046 — MCP parity for ADR lifecycle operations.
//!
//! Drives `product_adr_amend` and `product_adr_status` MCP tools directly
//! against a fresh temp repo and asserts on the post-write front-matter,
//! amendments array, and content-hash. Covers TC-577 through TC-585.

#![allow(clippy::unwrap_used)]

use super::harness::Session;
use product_lib::hash as hashlib;
use product_lib::mcp::ToolRegistry;

/// Seal an accepted ADR on disk: writes a sealed accepted ADR with a valid
/// content-hash computed over the body.
fn write_accepted_adr(s: &Session, id: &str, title: &str, body: &str) -> String {
    let hash = hashlib::compute_adr_hash(title, body);
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: accepted\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [api]\nscope: domain\ncontent-hash: {hash}\namendments: []\nsource-files: []\n---\n\n{body}\n",
        id = id,
        title = title,
        hash = hash,
        body = body,
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
    hash
}

fn write_proposed_adr(s: &Session, id: &str, title: &str, body: &str) {
    let content = format!(
        "---\nid: {id}\ntitle: {title}\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\ndomains: [api]\nscope: domain\n---\n\n{body}\n",
        id = id, title = title, body = body,
    );
    let slug = title.to_lowercase().replace(' ', "-");
    s.write(&format!("docs/adrs/{}-{}.md", id, slug), &content);
}

fn registry(s: &Session) -> ToolRegistry {
    ToolRegistry::new(s.dir.path().to_path_buf(), true)
}

fn file_path(_s: &Session, id: &str, title: &str) -> String {
    let slug = title.to_lowercase().replace(' ', "-");
    format!("docs/adrs/{}-{}.md", id, slug)
}

// ---------------------------------------------------------------------------
// TC-577 — amend via MCP atomically replaces body + updates hash + appends amendment
// ---------------------------------------------------------------------------
#[test]
fn tc_577_adr_amend_via_mcp_body_and_reason_atomic() {
    let s = Session::new();
    let original_body = "**Status:** Accepted\n\n**Context:** original context.\n\n**Decision:** original decision.\n\n**Rationale:** original rationale.\n\n**Rejected alternatives:** none.";
    let original_hash = write_accepted_adr(&s, "ADR-019", "Test Decision", original_body);
    let path = file_path(&s, "ADR-019", "Test Decision");

    let new_body = "**Status:** Accepted\n\n**Context:** new context per amendment.\n\n**Decision:** revised decision.\n\n**Rationale:** revised rationale.\n\n**Rejected alternatives:** none.";

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-019",
                "reason": "Remove internal LLM call per ADR-040",
                "body": new_body,
            }),
        )
        .expect("product_adr_amend should succeed");

    // Response shape assertions.
    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("ADR-019"));
    assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("accepted"));
    let new_hash = result
        .get("content-hash")
        .and_then(|v| v.as_str())
        .expect("content-hash in response");
    assert_ne!(new_hash, original_hash, "content-hash must change");
    let amendments = result.get("amendments").and_then(|v| v.as_array()).expect("amendments array");
    assert_eq!(amendments.len(), 1);
    assert_eq!(
        amendments[0].get("reason").and_then(|v| v.as_str()),
        Some("Remove internal LLM call per ADR-040")
    );
    assert_eq!(
        amendments[0].get("previous-hash").and_then(|v| v.as_str()),
        Some(original_hash.as_str())
    );

    // On-disk front-matter assertions.
    s.assert_frontmatter(&path, "content-hash", new_hash);
    s.assert_frontmatter(&path, "status", "accepted");

    // Body on disk matches the supplied body (modulo the parser's single trailing newline).
    let disk = s.read(&path);
    assert!(
        disk.contains("new context per amendment"),
        "expected new body content on disk; got:\n{}",
        disk
    );
    assert!(
        !disk.contains("original context"),
        "expected original body to be replaced on disk; got:\n{}",
        disk
    );

    // Graph check clean.
    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-578 — amend refuses status and leaves file unchanged
// ---------------------------------------------------------------------------
#[test]
fn tc_578_adr_amend_via_mcp_refuses_to_change_status() {
    let s = Session::new();
    let body = "**Status:** Accepted\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.";
    let original_hash = write_accepted_adr(&s, "ADR-077", "Guarded Decision", body);
    let path = file_path(&s, "ADR-077", "Guarded Decision");
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-077",
                "reason": "sneak a status change",
                "status": "abandoned",
            }),
        )
        .expect_err("product_adr_amend must reject payloads carrying status");
    assert!(err.contains("E019"), "expected E019; got: {}", err);
    assert!(
        err.contains("amendment-carries-status"),
        "expected named code; got: {}",
        err
    );

    // File unchanged (byte-for-byte digest).
    let post_digest = s.docs_digest();
    assert_eq!(pre_digest, post_digest, "file must be unchanged after rejected amend");

    // Front-matter sanity: hash and status preserved.
    s.assert_frontmatter(&path, "status", "accepted");
    s.assert_frontmatter(&path, "content-hash", &original_hash);
}

// ---------------------------------------------------------------------------
// TC-579 — amend rejects byte-identical body with E017
// ---------------------------------------------------------------------------
#[test]
fn tc_579_adr_amend_via_mcp_rejects_identical_body() {
    let s = Session::new();
    let body = "**Status:** Accepted\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.";
    write_accepted_adr(&s, "ADR-050", "Stable Decision", body);
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-050",
                "reason": "typo fix",
                "body": body,
            }),
        )
        .expect_err("amend with identical body must be rejected");
    assert!(err.contains("E017"), "expected E017; got: {}", err);
    assert!(
        err.contains("amendment-nothing-changed"),
        "expected named code; got: {}",
        err
    );

    let post_digest = s.docs_digest();
    assert_eq!(pre_digest, post_digest, "identical-body amend must not touch disk");

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-580 — status via MCP writes non-accepted transitions (proposed -> abandoned)
// ---------------------------------------------------------------------------
#[test]
fn tc_580_adr_status_via_mcp_writes_non_accepted_transitions() {
    let s = Session::new();
    write_proposed_adr(&s, "ADR-042", "Abandoning Decision", "**Status:** Proposed\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.");
    let path = file_path(&s, "ADR-042", "Abandoning Decision");

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-042", "status": "abandoned"}),
        )
        .expect("abandon should succeed");

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("ADR-042"));
    assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("abandoned"));
    assert!(
        result.get("note").is_none(),
        "response must not advise CLI fallback; got: {}",
        result
    );

    s.assert_frontmatter(&path, "status", "abandoned");
    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-581 — status accepted is CLI-only (E020), file unchanged
// ---------------------------------------------------------------------------
#[test]
fn tc_581_adr_status_via_mcp_rejects_accepted_transition() {
    let s = Session::new();
    write_proposed_adr(&s, "ADR-099", "Pending Acceptance", "**Status:** Proposed\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.");
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-099", "status": "accepted"}),
        )
        .expect_err("accepted transition must be rejected over MCP");
    assert!(err.contains("E020"), "expected E020; got: {}", err);
    assert!(
        err.contains("status-accepted-is-manual"),
        "expected named code; got: {}",
        err
    );
    assert!(
        err.contains("product adr status ADR-099 accepted"),
        "error must name the exact CLI command; got: {}",
        err
    );

    let post_digest = s.docs_digest();
    assert_eq!(pre_digest, post_digest, "rejected accepted must not touch disk");

    let path = file_path(&s, "ADR-099", "Pending Acceptance");
    s.assert_frontmatter(&path, "status", "proposed");
}

// ---------------------------------------------------------------------------
// TC-582 — status superseded writes bidirectional link atomically
// ---------------------------------------------------------------------------
#[test]
fn tc_582_adr_status_via_mcp_writes_superseded_with_bidirectional_link() {
    let s = Session::new();
    let old_body = "**Status:** Accepted\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.";
    let old_hash = write_accepted_adr(&s, "ADR-019", "Old Decision", old_body);
    write_accepted_adr(
        &s,
        "ADR-040",
        "New Decision",
        "**Status:** Accepted\n\n**Context:** different ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.",
    );
    let old_path = file_path(&s, "ADR-019", "Old Decision");
    let new_path = file_path(&s, "ADR-040", "New Decision");

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-019", "status": "superseded", "by": "ADR-040"}),
        )
        .expect("supersede should succeed");

    assert_eq!(result.get("id").and_then(|v| v.as_str()), Some("ADR-019"));
    assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("superseded"));
    let superseded_by = result
        .get("superseded-by")
        .and_then(|v| v.as_array())
        .expect("superseded-by array");
    assert!(
        superseded_by.iter().any(|v| v.as_str() == Some("ADR-040")),
        "response must list ADR-040 in superseded-by"
    );
    assert_eq!(
        result.get("content-hash").and_then(|v| v.as_str()),
        Some(old_hash.as_str()),
        "content-hash preserved on supersession"
    );

    // Both files updated atomically.
    s.assert_frontmatter(&old_path, "status", "superseded");
    s.assert_array_contains(&old_path, "superseded-by", "ADR-040");
    s.assert_frontmatter(&old_path, "content-hash", &old_hash);

    s.assert_array_contains(&new_path, "supersedes", "ADR-019");

    s.assert_graph_clean();
}

// ---------------------------------------------------------------------------
// TC-583 — status abandoned preserves content-hash on accepted ADR
// ---------------------------------------------------------------------------
#[test]
fn tc_583_adr_status_via_mcp_writes_abandoned() {
    let s = Session::new();
    let body = "**Status:** Accepted\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.";
    let hash = write_accepted_adr(&s, "ADR-099", "Decision To Abandon", body);
    let path = file_path(&s, "ADR-099", "Decision To Abandon");

    let reg = registry(&s);
    let result = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-099", "status": "abandoned"}),
        )
        .expect("abandon should succeed");

    assert_eq!(result.get("status").and_then(|v| v.as_str()), Some("abandoned"));
    assert_eq!(
        result.get("content-hash").and_then(|v| v.as_str()),
        Some(hash.as_str()),
        "content-hash preserved on abandonment"
    );

    s.assert_frontmatter(&path, "status", "abandoned");
    s.assert_frontmatter(&path, "content-hash", &hash);
}

// ---------------------------------------------------------------------------
// TC-584 — demotion from accepted -> proposed is forbidden (E021)
// ---------------------------------------------------------------------------
#[test]
fn tc_584_adr_status_via_mcp_rejects_demotion_from_accepted() {
    let s = Session::new();
    let body = "**Status:** Accepted\n\n**Context:** ctx.\n\n**Decision:** dec.\n\n**Rationale:** why.\n\n**Rejected alternatives:** none.";
    write_accepted_adr(&s, "ADR-077", "Sealed Decision", body);
    let pre_digest = s.docs_digest();

    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-077", "status": "proposed"}),
        )
        .expect_err("demotion must be rejected");
    assert!(err.contains("E021"), "expected E021; got: {}", err);
    assert!(
        err.contains("status-cannot-demote-accepted"),
        "expected named code; got: {}",
        err
    );

    let post_digest = s.docs_digest();
    assert_eq!(pre_digest, post_digest, "rejected demotion must not touch disk");
}

// ---------------------------------------------------------------------------
// TC-585 — exit-criteria: every FT-046 scenario passes in one session
// ---------------------------------------------------------------------------
#[test]
fn tc_585_mcp_parity_adr_lifecycle_exit() {
    // End-to-end smoke: exercise every lifecycle path the feature claims to
    // cover, in a single session. A clean pass here means FT-046 meets its
    // exit criteria.
    let s = Session::new();

    // 1. Amend with body on an accepted ADR.
    let body1 = "**Status:** Accepted\n\n**Context:** one.\n\n**Decision:** one.\n\n**Rationale:** one.\n\n**Rejected alternatives:** none.";
    write_accepted_adr(&s, "ADR-100", "Lifecycle One", body1);
    let reg = registry(&s);
    let r = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-100",
                "reason": "smoke-test amend",
                "body": "**Status:** Accepted\n\n**Context:** replaced.\n\n**Decision:** replaced.\n\n**Rationale:** replaced.\n\n**Rejected alternatives:** none.",
            }),
        )
        .expect("amend with body");
    assert!(r.get("content-hash").is_some());
    assert_eq!(
        r.get("amendments").and_then(|v| v.as_array()).map(|a| a.len()).unwrap_or(0),
        1
    );

    // 2. proposed -> abandoned via MCP writes the file.
    write_proposed_adr(
        &s,
        "ADR-101",
        "Lifecycle Two",
        "**Status:** Proposed\n\n**Context:** two.\n\n**Decision:** two.\n\n**Rationale:** two.\n\n**Rejected alternatives:** none.",
    );
    let reg = registry(&s);
    reg.call_tool(
        "product_adr_status",
        &serde_json::json!({"id": "ADR-101", "status": "abandoned"}),
    )
    .expect("abandon proposed");
    s.assert_frontmatter(&file_path(&s, "ADR-101", "Lifecycle Two"), "status", "abandoned");

    // 3. accepted -> superseded via MCP with bidirectional link.
    write_accepted_adr(
        &s,
        "ADR-102",
        "Lifecycle Three",
        "**Status:** Accepted\n\n**Context:** three.\n\n**Decision:** three.\n\n**Rationale:** three.\n\n**Rejected alternatives:** none.",
    );
    write_accepted_adr(
        &s,
        "ADR-103",
        "Lifecycle Four",
        "**Status:** Accepted\n\n**Context:** four.\n\n**Decision:** four.\n\n**Rationale:** four.\n\n**Rejected alternatives:** none.",
    );
    let reg = registry(&s);
    reg.call_tool(
        "product_adr_status",
        &serde_json::json!({"id": "ADR-102", "status": "superseded", "by": "ADR-103"}),
    )
    .expect("supersede");
    s.assert_array_contains(&file_path(&s, "ADR-102", "Lifecycle Three"), "superseded-by", "ADR-103");
    s.assert_array_contains(&file_path(&s, "ADR-103", "Lifecycle Four"), "supersedes", "ADR-102");

    // 4. accepted over MCP rejected with E020 naming CLI command.
    write_proposed_adr(
        &s,
        "ADR-104",
        "Lifecycle Five",
        "**Status:** Proposed\n\n**Context:** five.\n\n**Decision:** five.\n\n**Rationale:** five.\n\n**Rejected alternatives:** none.",
    );
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-104", "status": "accepted"}),
        )
        .expect_err("accepted must be rejected");
    assert!(err.contains("E020"));
    assert!(err.contains("product adr status ADR-104 accepted"));

    // 5. accepted -> proposed rejected with E021.
    write_accepted_adr(
        &s,
        "ADR-105",
        "Lifecycle Six",
        "**Status:** Accepted\n\n**Context:** six.\n\n**Decision:** six.\n\n**Rationale:** six.\n\n**Rejected alternatives:** none.",
    );
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_status",
            &serde_json::json!({"id": "ADR-105", "status": "proposed"}),
        )
        .expect_err("demotion must be rejected");
    assert!(err.contains("E021"));

    // 6. amend with identical body -> E017.
    write_accepted_adr(
        &s,
        "ADR-106",
        "Lifecycle Seven",
        "**Status:** Accepted\n\n**Context:** seven.\n\n**Decision:** seven.\n\n**Rationale:** seven.\n\n**Rejected alternatives:** none.",
    );
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-106",
                "reason": "noop",
                "body": "**Status:** Accepted\n\n**Context:** seven.\n\n**Decision:** seven.\n\n**Rationale:** seven.\n\n**Rejected alternatives:** none.",
            }),
        )
        .expect_err("identical body must be rejected");
    assert!(err.contains("E017"));

    // 7. amend carrying status -> E019.
    let reg = registry(&s);
    let err = reg
        .call_tool(
            "product_adr_amend",
            &serde_json::json!({
                "id": "ADR-106",
                "reason": "sneaky",
                "status": "abandoned",
            }),
        )
        .expect_err("amend carrying status must be rejected");
    assert!(err.contains("E019"));

    // Final invariant: graph still clean after the whole lifecycle dance.
    s.assert_graph_clean();
}
