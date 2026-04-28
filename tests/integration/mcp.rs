//! Integration tests — mcp.

#![allow(clippy::unwrap_used)]

use super::harness::*;

#[test]
fn mcp_001_stdio_initialize() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("protocolVersion"), "initialize should return protocolVersion: {}", out);
}

#[test]
fn mcp_002_stdio_tools_list() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    let out = run_mcp_stdio(&h, input);
    let count = out.matches("\"name\"").count();
    assert!(count >= 10, "should list >=10 tools, got {}: {}", count, &out[..200.min(out.len())]);
}

#[test]
fn mcp_003_stdio_feature_list() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_feature_list","arguments":{}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("FT-001"), "should contain FT-001: {}", out);
}

#[test]
fn mcp_004_stdio_context() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"product_context","arguments":{"id":"FT-001","depth":1}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("Bundle"), "should contain Bundle: {}", out);
}

#[test]
fn mcp_005_stdio_graph_check() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"product_graph_check","arguments":{}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("errors") || out.contains("warnings"), "should contain errors or warnings: {}", out);
}

#[test]
fn mcp_006_stdio_impact() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"product_impact","arguments":{"id":"ADR-001"}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("seed") || out.contains("direct"), "should contain seed: {}", out);
}

#[test]
fn mcp_007_stdio_feature_new_write() {
    let h = Harness::new();
    // Enable write
    h.write("product.toml", &format!("{}\n[mcp]\nwrite = true\n", MINIMAL_CONFIG));
    let input = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"product_feature_new","arguments":{"title":"MCP Feature","phase":1}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("FT-001") || out.contains("path"), "should create feature: {}", out);
    // Verify file exists
    let entries: Vec<_> = std::fs::read_dir(h.dir.path().join("docs/features"))
        .unwrap_or_else(|_| panic!("features dir"))
        .collect();
    assert!(!entries.is_empty(), "feature file should be created");
}

#[test]
fn mcp_008_stdio_write_disabled() {
    let h = fixture_minimal();
    // No [mcp] section → write disabled by default
    let input = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"product_feature_new","arguments":{"title":"Blocked"}}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("disabled") || out.contains("error"), "write should be blocked: {}", out);
}

#[test]
fn mcp_009_stdio_unknown_method() {
    let h = fixture_minimal();
    let input = r#"{"jsonrpc":"2.0","id":9,"method":"nonexistent","params":{}}"#;
    let out = run_mcp_stdio(&h, input);
    assert!(out.contains("Method not found") || out.contains("error"), "should error: {}", out);
}

#[test]
fn tc_099_mcp_stdio_tool_call() {
    let h = fixture_minimal();

    // Send a valid JSON-RPC tools/call request over stdin
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_list","arguments":{}}}"#;
    let output = run_mcp_stdio(&h, input);

    // Response should be valid JSON-RPC
    assert!(output.contains("jsonrpc"), "Response should be JSON-RPC format: {}", output);
    assert!(output.contains("\"id\""), "Response should include request id: {}", output);

    // Response should contain tool result with feature data
    assert!(output.contains("FT-001"), "Response should contain FT-001 from fixture: {}", output);

    // Should not contain an error
    let parsed: serde_json::Value = output.lines()
        .filter(|l| l.contains("jsonrpc"))
        .next()
        .and_then(|l| serde_json::from_str(l).ok())
        .expect("Should parse JSON-RPC response");
    assert!(parsed.get("result").is_some(), "Response should have result field, not error: {}", output);
}

#[test]
fn tc_100_mcp_http_tool_call() {
    let h = fixture_minimal();
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "test-token-100"]);

    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_list","arguments":{}}}"#;
    let (status, _headers, resp_body) = http_post(port, body, Some("Bearer test-token-100"));

    // Kill the server
    let _ = child.kill();
    let _ = child.wait();

    assert!(status.contains("200"), "Expected 200, got: {}", status);
    assert!(resp_body.contains("FT-001"), "Response should contain FT-001: {}", resp_body);
    assert!(resp_body.contains("jsonrpc"), "Response should be JSON-RPC: {}", resp_body);
}

#[test]
fn tc_101_mcp_http_no_token_401() {
    let h = fixture_minimal();
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "secret-token-101"]);

    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let (status, _headers, _resp_body) = http_post(port, body, None);

    let _ = child.kill();
    let _ = child.wait();

    assert!(status.contains("401"), "Expected 401 without token, got: {}", status);
}

#[test]
fn tc_102_mcp_http_wrong_token_401() {
    let h = fixture_minimal();
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "correct-token-102"]);

    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let (status, _headers, _resp_body) = http_post(port, body, Some("Bearer wrong-token"));

    let _ = child.kill();
    let _ = child.wait();

    assert!(status.contains("401"), "Expected 401 with wrong token, got: {}", status);
}

#[test]
fn tc_103_mcp_http_write_disabled() {
    let h = Harness::new();
    // Explicitly set write = false (the default, but be explicit)
    h.write("product.toml", &format!("{}\n[mcp]\nwrite = false\n", MINIMAL_CONFIG));
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test Feature\nstatus: draft\nphase: 1\n---\n");
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "write-test-103"]);

    // Call a write tool (product_feature_new) which requires write to be enabled
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_new","arguments":{"title":"Should Fail"}}}"#;
    let (status, _headers, resp_body) = http_post(port, body, Some("Bearer write-test-103"));

    let _ = child.kill();
    let _ = child.wait();

    // Should return HTTP 200 (not an HTTP error — the error is at the tool level)
    assert!(status.contains("200"), "Expected HTTP 200 (tool error, not HTTP error), got: {}", status);

    // The JSON-RPC response should contain an error about write tools being disabled
    assert!(
        resp_body.contains("Write tools are disabled") || resp_body.contains("write") && resp_body.contains("disabled"),
        "Expected write-disabled error in response: {}",
        resp_body
    );

    // The response should be a JSON-RPC error, not a result
    assert!(
        resp_body.contains("\"error\""),
        "Response should contain JSON-RPC error field: {}",
        resp_body
    );
}

#[test]
fn tc_104_mcp_http_concurrent_writes() {
    let h = Harness::new();
    h.write("product.toml", &format!("{}\n[mcp]\nwrite = true\n", MINIMAL_CONFIG));
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "write-token-104", "--write"]);

    // Create a lock file held by a live process (this test process) to simulate concurrency
    let lock_path = h.dir.path().join(".product.lock");
    std::fs::write(
        &lock_path,
        format!("pid={}\nstarted=2026-04-13T00:00:00Z\n", std::process::id()),
    ).expect("write lock");

    let body = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_new","arguments":{"title":"Concurrent Test"}}}"#;
    let (status, _headers, resp_body) = http_post(port, body, Some("Bearer write-token-104"));

    // Remove the lock
    let _ = std::fs::remove_file(&lock_path);

    let _ = child.kill();
    let _ = child.wait();

    // The request should return 200 (HTTP level) with a tool error about the lock
    assert!(status.contains("200"), "Expected 200 HTTP status, got: {}", status);
    // The JSON-RPC response should contain an error about the lock
    assert!(
        resp_body.contains("lock") || resp_body.contains("error") || resp_body.contains("pid"),
        "Expected lock-held error in response: {}",
        resp_body
    );
}

#[test]
fn tc_105_mcp_http_graceful_shutdown() {
    use std::process::Command;

    let h = fixture_minimal();
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &["--token", "shutdown-token-105"]);

    // Send a request to verify server is working
    let body = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#;
    let (status, _headers, _resp_body) = http_post(port, body, Some("Bearer shutdown-token-105"));
    assert!(status.contains("200"), "Server should be responding before SIGTERM: {}", status);

    // Send SIGTERM
    #[cfg(unix)]
    {
        let pid = child.id();
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }

        // Wait for process to exit (with timeout)
        let start = std::time::Instant::now();
        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process exited — graceful shutdown worked
                    assert!(status.success() || status.code() == Some(0),
                        "Server should exit cleanly after SIGTERM, got: {:?}", status);
                    break;
                }
                Ok(None) => {
                    if start.elapsed() > std::time::Duration::from_secs(15) {
                        let _ = child.kill();
                        let _ = child.wait();
                        panic!("Server did not exit within 15 seconds after SIGTERM");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    panic!("Error checking process status: {}", e);
                }
            }
        }
    }

    #[cfg(not(unix))]
    {
        let _ = child.kill();
        let _ = child.wait();
    }
}

#[test]
fn tc_107_mcp_cors_header() {
    let h = Harness::new();
    h.write("product.toml", &format!("{}\n[mcp]\nwrite = false\ncors-origins = [\"https://claude.ai\"]\n", MINIMAL_CONFIG));
    let port = unique_port();
    let mut child = start_mcp_http(&h, port, &[]);

    let (status, headers, _body) = http_options(port, "https://claude.ai");

    let _ = child.kill();
    let _ = child.wait();

    assert!(status.contains("200"), "Preflight should return 200, got: {}", status);
    let headers_lower = headers.to_lowercase();
    assert!(
        headers_lower.contains("access-control-allow-origin"),
        "Should have CORS allow-origin header: {}", headers
    );
    assert!(
        headers.contains("https://claude.ai"),
        "Should allow claude.ai origin: {}", headers
    );
    assert!(
        headers_lower.contains("access-control-allow-methods"),
        "Should have CORS allow-methods header: {}", headers
    );
}

#[test]
fn tc_425_mcp_write_tools_cannot_modify_accepted_adr_body() {
    let h = Harness::new();
    // Write product.toml with MCP write enabled
    h.write(
        "product.toml",
        r#"name = "test"
schema-version = "1"
[paths]
features = "docs/features"
adrs = "docs/adrs"
tests = "docs/tests"
graph = "docs/graph"
checklist = "docs/checklist.md"
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"
[mcp]
write = true
"#,
    );

    // Create an accepted ADR with a valid content-hash
    let adr_body = "This is the decision body.\n";
    let hash = compute_adr_content_hash("Accepted ADR", adr_body.trim());
    h.write(
        "docs/adrs/ADR-001-accepted.md",
        &format!(
            "---\nid: ADR-001\ntitle: Accepted ADR\nstatus: accepted\ncontent-hash: {}\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\n{}", hash, adr_body
        ),
    );
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-001]\ntests: []\n---\n\nBody.\n",
    );

    // Try to modify the accepted ADR body via MCP product_body_update — should fail
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_body_update","arguments":{"id":"ADR-001","body":"Modified body"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        out.contains("Cannot modify body of accepted ADR"),
        "MCP should reject body update of accepted ADR.\nGot: {}",
        out
    );

    // Verify product_adr_status (front-matter only) still works via MCP for
    // non-accepted transitions. FT-046 made `accepted` CLI-only (E020), so
    // this test exercises `abandoned` which preserves the content-hash and
    // only touches the mutable `status` field.
    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_adr_status","arguments":{"id":"ADR-001","status":"abandoned"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        !out.contains("\"error\""),
        "product_adr_status should work on accepted ADR for non-accepted transitions.\nGot: {}",
        out
    );

    // Verify product_feature_link (modifies feature front-matter, excluded from hash) still works
    let input = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_feature_link","arguments":{"id":"FT-001","adr":"ADR-001"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        !out.contains("Cannot modify"),
        "product_feature_link should work.\nGot: {}",
        out
    );
}

#[test]
fn tc_323_mcp_prompts_list_tool() {
    let h = fixture_minimal();

    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_prompts_list","arguments":{}}}"#;
    let out = run_mcp_stdio(&h, input);

    // Response should contain prompt entries
    assert!(
        out.contains("author-feature"),
        "MCP response should list author-feature prompt.\nGot: {}",
        out
    );
    assert!(
        out.contains("author-adr"),
        "MCP response should list author-adr prompt.\nGot: {}",
        out
    );
    assert!(
        out.contains("author-review"),
        "MCP response should list author-review prompt.\nGot: {}",
        out
    );
    assert!(
        out.contains("prompts"),
        "Response should contain 'prompts' key.\nGot: {}",
        out
    );
}

#[test]
fn tc_324_mcp_prompts_get_tool() {
    let h = fixture_minimal();

    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_prompts_get","arguments":{"name":"author-feature"}}}"#;
    let out = run_mcp_stdio(&h, input);

    // Response should contain the prompt content
    assert!(
        out.contains("product_feature_list") || out.contains("feature"),
        "MCP response should contain prompt content.\nGot: {}",
        out
    );
    assert!(
        out.contains("author-feature"),
        "Response should contain prompt name.\nGot: {}",
        out
    );
}

#[test]
fn tc_469_mcp_tools_mirror_cli_for_all_field_mutations() {
    let h = fixture_with_domains();
    h.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    h.write("docs/adrs/ADR-001-test.md", "---\nid: ADR-001\ntitle: Test ADR\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");
    h.write("docs/adrs/ADR-002-test.md", "---\nid: ADR-002\ntitle: Test ADR 2\nstatus: proposed\nfeatures: []\nsupersedes: []\nsuperseded-by: []\n---\n\nBody.\n");
    h.write("docs/tests/TC-001-test.md", "---\nid: TC-001\ntitle: Test TC\ntype: scenario\nstatus: unimplemented\nvalidates:\n  features: []\n  adrs: []\nphase: 1\n---\n\nDesc.\n");

    // Test product_feature_domain via MCP
    let req = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_feature_domain","arguments":{"id":"FT-001","add":["api"]}}}"#;
    let out = h.run_with_stdin(&["mcp"], req);
    assert!(out.stdout.contains("api"), "MCP feature_domain should add api domain");
    let content = h.read("docs/features/FT-001-test.md");
    assert!(content.contains("api"), "feature file should have api domain");

    // Test product_feature_acknowledge via MCP
    let req2 = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_feature_acknowledge","arguments":{"id":"FT-001","domain":"security","reason":"No trust boundaries"}}}"#;
    let out2 = h.run_with_stdin(&["mcp"], req2);
    assert!(!out2.stdout.contains("error"), "MCP feature_acknowledge should succeed: {}", out2.stdout);

    // Test product_adr_domain via MCP
    let req3 = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_adr_domain","arguments":{"id":"ADR-001","add":["error-handling"]}}}"#;
    let out3 = h.run_with_stdin(&["mcp"], req3);
    assert!(out3.stdout.contains("error-handling"), "MCP adr_domain should add error-handling");

    // Test product_adr_scope via MCP
    let req4 = r#"{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"product_adr_scope","arguments":{"id":"ADR-001","scope":"cross-cutting"}}}"#;
    let out4 = h.run_with_stdin(&["mcp"], req4);
    assert!(out4.stdout.contains("cross-cutting"), "MCP adr_scope should set cross-cutting");

    // Test product_adr_supersede via MCP
    let req5 = r#"{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"product_adr_supersede","arguments":{"id":"ADR-002","supersedes":"ADR-001"}}}"#;
    let out5 = h.run_with_stdin(&["mcp"], req5);
    assert!(out5.stdout.contains("added"), "MCP adr_supersede should add link: {}", out5.stdout);

    // Test product_adr_source_files via MCP
    let req6 = r#"{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"product_adr_source_files","arguments":{"id":"ADR-001","add":["src/test.rs"]}}}"#;
    let out6 = h.run_with_stdin(&["mcp"], req6);
    assert!(out6.stdout.contains("src/test.rs"), "MCP adr_source_files should add path");

    // Test product_test_runner via MCP
    let req7 = r#"{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"product_test_runner","arguments":{"id":"TC-001","runner":"cargo-test","args":"tc_001_test"}}}"#;
    let out7 = h.run_with_stdin(&["mcp"], req7);
    assert!(out7.stdout.contains("cargo-test"), "MCP test_runner should set runner");

    // Test that write tools require mcp.write = true
    let h2 = Harness::new(); // default harness has no [mcp] section (write=false)
    h2.write("docs/features/FT-001-test.md", "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: []\ntests: []\n---\n\nBody.\n");
    let req_write = r#"{"jsonrpc":"2.0","id":8,"method":"tools/call","params":{"name":"product_feature_domain","arguments":{"id":"FT-001","add":["api"]}}}"#;
    let out_write = h2.run_with_stdin(&["mcp"], req_write);
    assert!(out_write.stdout.contains("Write tools are disabled") || out_write.stdout.contains("error"),
        "Write tools should be disabled without mcp.write=true: {}", out_write.stdout);
}

#[test]
fn tc_620_mcp_body_update_rewrites_dep_body() {
    let h = Harness::new();

    // Original dep with a fully populated front-matter and a known body.
    let front = "---\n\
                 id: DEP-001\n\
                 title: openraft\n\
                 type: library\n\
                 source: crates.io\n\
                 version: \">=0.9,<1.0\"\n\
                 status: active\n\
                 features:\n  - FT-001\n\
                 adrs:\n  - ADR-002\n\
                 supersedes: []\n\
                 availability-check: ~\n\
                 breaking-change-risk: medium\n\
                 ---\n\n";
    let original_body = "Original rationale text.\n";
    let original = format!("{}{}", front, original_body);
    h.write("docs/dependencies/DEP-001-openraft.md", &original);

    // A feature that links to the dep so graph check sees a well-formed graph.
    h.write(
        "docs/features/FT-001-test.md",
        "---\nid: FT-001\ntitle: Test\nphase: 1\nstatus: planned\ndepends-on: []\nadrs: [ADR-002]\ntests: []\n---\n\nBody.\n",
    );
    h.write(
        "docs/adrs/ADR-002-raft.md",
        "---\nid: ADR-002\ntitle: Raft\nstatus: proposed\nfeatures: [FT-001]\nsupersedes: []\nsuperseded-by: []\n---\n\n**Context:** X\n**Decision:** Y\n**Rationale:** Z\n**Rejected alternatives:** none\n",
    );

    // Invoke product_body_update on DEP-001 with a new body.
    let new_body = "Replacement rationale — now with migration plan.";
    let input = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"product_body_update","arguments":{{"id":"DEP-001","body":{}}}}}}}"#,
        serde_json::to_string(new_body).unwrap()
    );
    let out = run_mcp_stdio_write(&h, &input);

    // The tool result announces success. The response embeds the tool JSON
    // as an escaped string inside `result.content[0].text`, so we assert on
    // substrings rather than the exact byte sequence.
    assert!(
        out.contains("\\\"updated\\\": true") || out.contains("\"updated\": true"),
        "MCP should report updated=true; got: {}",
        out
    );
    assert!(
        out.contains("DEP-001"),
        "Response should include DEP-001; got: {}",
        out
    );
    assert!(!out.contains("\"error\":"), "No error expected; got: {}", out);

    // Reading the file back: body is replaced, front-matter is preserved.
    let on_disk = h.read("docs/dependencies/DEP-001-openraft.md");
    assert!(
        on_disk.contains("Replacement rationale"),
        "body should be replaced; got: {}",
        on_disk
    );
    assert!(
        !on_disk.contains("Original rationale text."),
        "old body must be gone; got: {}",
        on_disk
    );
    // Every populated front-matter field is still present. (Fields with
    // null / empty defaults — availability-check: ~, supersedes: [] — are
    // serialized with skip_serializing_if and so round-trip to absent, which
    // matches the behaviour of the other three artifact types.)
    for field in [
        "id: DEP-001",
        "title: openraft",
        "type: library",
        "source: crates.io",
        ">=0.9,<1.0",
        "status: active",
        "- FT-001",
        "- ADR-002",
        "breaking-change-risk: medium",
    ] {
        assert!(
            on_disk.contains(field),
            "front-matter field {:?} missing after body_update; got:\n{}",
            field,
            on_disk
        );
    }

    // The graph still parses cleanly after the rewrite (no E-class errors).
    let check = h.run(&["graph", "check"]);
    assert!(
        check.exit_code == 0 || check.exit_code == 2,
        "graph check should not emit E-class errors after DEP body update; exit={}, stdout={}, stderr={}",
        check.exit_code, check.stdout, check.stderr
    );
}

#[test]
fn tc_621_mcp_body_update_dep_error_paths() {
    let h = Harness::new();

    // Record the pre-call state of the dependencies directory.
    let deps_dir = h.dir.path().join("docs/dependencies");
    let before: Vec<String> = std::fs::read_dir(&deps_dir)
        .map(|r| {
            r.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();

    // 1) Valid prefix, unknown ID — error names DEP-999.
    let input = r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_body_update","arguments":{"id":"DEP-999","body":"anything"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        out.contains("DEP-999"),
        "error should name DEP-999; got: {}",
        out
    );
    assert!(
        out.to_lowercase().contains("not found"),
        "error should mirror 'not found' wording; got: {}",
        out
    );

    // 2) Unknown prefix — the existing fallback error is preserved.
    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_body_update","arguments":{"id":"FOO-001","body":"anything"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        out.contains("Unknown artifact ID prefix: FOO-001"),
        "error must be the unchanged fallback string; got: {}",
        out
    );

    // Neither call mutated a file: the dependencies directory listing is
    // identical before and after.
    let after: Vec<String> = std::fs::read_dir(&deps_dir)
        .map(|r| {
            r.filter_map(|e| e.ok())
                .map(|e| e.file_name().to_string_lossy().to_string())
                .collect()
        })
        .unwrap_or_default();
    assert_eq!(
        before, after,
        "dependencies directory should not change after failed body_update calls"
    );
}

#[test]
fn tc_622_mcp_body_update_dep_exit() {
    let h = Harness::new();

    // 1) Tool description lists DEP-NNN alongside the other prefixes.
    let input = r#"{"jsonrpc":"2.0","id":0,"method":"tools/list"}"#;
    let listing = run_mcp_stdio_write(&h, input);
    assert!(
        listing.contains("product_body_update"),
        "tools/list must include product_body_update; got: {}",
        listing
    );
    assert!(
        listing.contains("DEP-NNN"),
        "product_body_update tool schema/description must mention DEP-NNN; got: {}",
        listing
    );

    // Seed a dep file.
    let front = "---\n\
                 id: DEP-001\n\
                 title: openraft\n\
                 type: library\n\
                 source: crates.io\n\
                 version: \">=0.9\"\n\
                 status: active\n\
                 features: []\n\
                 adrs: []\n\
                 supersedes: []\n\
                 availability-check: ~\n\
                 breaking-change-risk: medium\n\
                 ---\n\n";
    h.write(
        "docs/dependencies/DEP-001-openraft.md",
        &format!("{}Original.\n", front),
    );

    // 2) Valid DEP update succeeds and rewrites the body on disk.
    let new_body = "Rewritten rationale for DEP-001.";
    let input = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{{"name":"product_body_update","arguments":{{"id":"DEP-001","body":{}}}}}}}"#,
        serde_json::to_string(new_body).unwrap()
    );
    let out = run_mcp_stdio_write(&h, &input);
    assert!(
        out.contains("\\\"updated\\\": true") || out.contains("\"updated\": true"),
        "valid DEP body update should report success; got: {}",
        out
    );
    let on_disk = h.read("docs/dependencies/DEP-001-openraft.md");
    assert!(
        on_disk.contains("Rewritten rationale for DEP-001."),
        "body should be replaced; got: {}",
        on_disk
    );
    assert!(
        on_disk.contains("id: DEP-001") && on_disk.contains("title: openraft"),
        "front-matter must survive; got: {}",
        on_disk
    );

    // 3) Unknown DEP — dep-specific "not found" wording.
    let input = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"product_body_update","arguments":{"id":"DEP-999","body":"x"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        out.contains("DEP-999") && out.to_lowercase().contains("not found"),
        "unknown DEP must produce a 'not found' error naming it; got: {}",
        out
    );

    // 4) Unknown prefix — the unchanged fallback is returned.
    let input = r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"product_body_update","arguments":{"id":"FOO-001","body":"x"}}}"#;
    let out = run_mcp_stdio_write(&h, input);
    assert!(
        out.contains("Unknown artifact ID prefix: FOO-001"),
        "unknown prefix must hit the fallback unchanged; got: {}",
        out
    );
}

