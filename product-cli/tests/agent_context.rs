//! The agent-facing context stays in step with the policy table.
//!
//! Three surfaces tell an agent what the What interceptor gates: the MCP
//! `instructions`, the `product_domain_*` tool descriptions, and the
//! `product-what` skill. All three are generated from — or checked
//! against — `ddd_core::what::WHAT_POLICY`, so adding a row cannot leave
//! one of them describing an older table.
//!
//! Regenerate the skill block with `UPDATE_SKILL=1 cargo test -p
//! product-cli --test agent_context`.

use std::path::{Path, PathBuf};

const BEGIN: &str = "<!-- BEGIN GENERATED: what-policy (product_mcp::instructions::skill_section) -->";
const END: &str = "<!-- END GENERATED -->";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

fn skill_path() -> PathBuf {
    repo_root().join(".claude/skills/product-what/SKILL.md")
}

/// The text between the generated markers.
fn block_of(text: &str) -> Option<&str> {
    let start = text.find(BEGIN)? + BEGIN.len();
    let end = text.find(END)?;
    Some(text[start..end].trim())
}

#[test]
fn the_skill_policy_block_matches_the_table() {
    let path = skill_path();
    let text = std::fs::read_to_string(&path).expect("skill file");
    let generated = product_mcp::instructions::skill_section();
    let generated = generated.trim();

    if std::env::var("UPDATE_SKILL").as_deref() == Ok("1") {
        let start = text.find(BEGIN).expect("begin marker") + BEGIN.len();
        let end = text.find(END).expect("end marker");
        let updated = format!("{}\n{generated}\n{}", &text[..start], &text[end..]);
        std::fs::write(&path, updated).expect("write skill");
        return;
    }

    let block = block_of(&text).unwrap_or_else(|| {
        panic!("{} is missing the generated markers", path.display());
    });
    assert_eq!(
        block,
        generated,
        "\nthe skill's policy block is stale — regenerate with:\n  \
         UPDATE_SKILL=1 cargo test -p product-cli --test agent_context\n"
    );
}

#[test]
fn the_skill_names_the_recovery_path() {
    let text = std::fs::read_to_string(skill_path()).expect("skill file");
    for needle in ["product_what_declare", "verdict_knowledge", "rejected"] {
        assert!(text.contains(needle), "the skill never mentions `{needle}`");
    }
}

/// A write tool that can be rejected has to say so where the agent reads it.
#[test]
fn the_intercepted_tool_descriptions_name_the_gate() {
    let registry = product_mcp::ToolRegistry::new(repo_root(), false);
    for name in ["product_domain_new", "product_domain_edit", "product_domain_rm"] {
        let tool = registry
            .tool_list()
            .iter()
            .find(|t| t.name == name)
            .unwrap_or_else(|| panic!("{name} missing from the registry"));
        assert!(
            tool.description.contains("product_what_declare"),
            "{name}'s description never points at the declaration that unblocks it"
        );
    }
    let declare = registry
        .tool_list()
        .iter()
        .find(|t| t.name == "product_what_declare")
        .expect("product_what_declare missing");
    assert!(declare.description.contains("verdict_knowledge"));
}
