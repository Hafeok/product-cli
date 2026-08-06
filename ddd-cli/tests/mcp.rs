//! `ddd serve` stdio smoke: the MCP handshake exposes the ddd_* surface.

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn serve_lists_the_ddd_tool_namespace_over_stdio() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let mut cmd = Command::cargo_bin("ddd").expect("binary");
    cmd.current_dir(tmp.path());
    cmd.arg("init").assert().success();

    let mut serve = Command::cargo_bin("ddd").expect("binary");
    serve.current_dir(tmp.path()).arg("serve").write_stdin(concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#,
        "\n",
    ));
    serve
        .assert()
        .success()
        .stdout(predicate::str::contains("\"ddd-mcp\""))
        .stdout(predicate::str::contains("ddd_apply_edit"))
        .stdout(predicate::str::contains("ddd_declare_seam"))
        .stdout(predicate::str::contains("ddd_find_symbol"))
        .stdout(predicate::str::contains("ddd_warmup"))
        .stdout(predicate::str::contains("ddd_why").and(predicate::str::contains("ddd_accept_risk")));
}

#[test]
fn serve_rejects_unknown_tools_cleanly() {
    let tmp = tempfile::tempdir().expect("tempdir");
    Command::cargo_bin("ddd").expect("binary").current_dir(tmp.path()).arg("init").assert().success();
    let mut serve = Command::cargo_bin("ddd").expect("binary");
    serve.current_dir(tmp.path()).arg("serve").write_stdin(concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"product_domain_new","arguments":{}}}"#,
        "\n",
    ));
    serve.assert().success().stdout(predicate::str::contains("Tool not found"));
}
