//! Authoring sessions — graph-aware specification writing (ADR-022)
//!
//! `product author feature/adr/review` starts Claude Code with a versioned
//! system prompt and Product MCP active.

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use std::path::Path;
use std::process::Command;

/// Session types for authoring
pub enum SessionType {
    Feature,
    Adr,
    Review,
}

impl std::fmt::Display for SessionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Feature => write!(f, "feature"),
            Self::Adr => write!(f, "adr"),
            Self::Review => write!(f, "review"),
        }
    }
}

/// Start an authoring session
pub fn start_session(
    session_type: SessionType,
    _config: &ProductConfig,
    root: &Path,
) -> Result<()> {
    let prompt_name = match session_type {
        SessionType::Feature => "author-feature-v1.md",
        SessionType::Adr => "author-adr-v1.md",
        SessionType::Review => "author-review-v1.md",
    };

    let prompt_path = root.join("benchmarks/prompts").join(prompt_name);
    let base_prompt = if prompt_path.exists() {
        std::fs::read_to_string(&prompt_path).unwrap_or_default()
    } else {
        default_prompt(&session_type)
    };
    let prompt = format!("{}\n\n{}", base_prompt, schema_prompt());

    // Write prompt to temp file for agent
    let tmp_dir = std::env::temp_dir();
    let tmp_path = tmp_dir.join(format!("product-author-{}-{}.md", session_type, chrono::Utc::now().timestamp()));
    std::fs::write(&tmp_path, &prompt).map_err(|e| ProductError::WriteError {
        path: tmp_path.clone(),
        message: e.to_string(),
    })?;

    println!("Starting {} authoring session...", session_type);
    println!("  System prompt: {}", if prompt_path.exists() { prompt_path.display().to_string() } else { "(default)".to_string() });
    println!("  Repo: {}", root.display());
    println!();

    // Build inline MCP config using the current executable
    let exe = std::env::current_exe()
        .unwrap_or_else(|_| "product".into());
    let mcp_config = serde_json::json!({
        "mcpServers": {
            "product": {
                "command": exe.display().to_string(),
                "args": ["mcp", "--write"],
                "cwd": root.display().to_string()
            }
        }
    });
    let mcp_json = serde_json::to_string(&mcp_config).unwrap_or_default();

    // Invoke Claude Code with restricted tool access:
    // - --system-prompt-file: custom system prompt for the authoring session
    // - --tools "Read": only built-in Read tool (no Bash/Edit/Write)
    // - --allowedTools: auto-approve Read + all product MCP tools
    // - --strict-mcp-config + --mcp-config: only the product MCP server
    // NOTE: We intentionally do NOT use --bare, because it disables
    // OAuth/keychain auth (only ANTHROPIC_API_KEY works in bare mode).
    let status = Command::new("claude")
        .args([
            "--system-prompt-file", &tmp_path.display().to_string(),
            "--tools", "Read",
            "--allowedTools", "Read,mcp__product__*",
            "--mcp-config", &mcp_json,
            "--strict-mcp-config",
        ])
        .current_dir(root)
        .status();

    match status {
        Ok(s) if s.success() => {
            println!();
            println!("Authoring session complete.");
            auto_commit(&session_type, root);
        }
        Ok(s) => {
            eprintln!("Agent exited with status: {}", s);
        }
        Err(e) => {
            eprintln!("Could not start Claude Code: {}", e);
            eprintln!("Ensure 'claude' is in your PATH.");
            eprintln!();
            eprintln!("System prompt written to: {}", tmp_path.display());
            eprintln!("You can use it manually with any agent.");
        }
    }

    Ok(())
}

/// Review staged ADR files (pre-commit hook)
pub fn review_staged(root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--cached", "--name-only"])
        .current_dir(root)
        .output()
        .map_err(|e| ProductError::IoError(format!("git: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let staged_adrs: Vec<&str> = stdout
        .lines()
        .filter(|l| l.contains("adrs/") && l.ends_with(".md"))
        .collect();

    if staged_adrs.is_empty() {
        return Ok(vec!["No staged ADR files found.".to_string()]);
    }

    let mut findings = Vec::new();

    for adr_path in &staged_adrs {
        let full_path = root.join(adr_path);
        if !full_path.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&full_path).unwrap_or_default();

        // Structural checks (local, instant)
        let required_sections = [
            ("Context", "**Context:**"),
            ("Decision", "**Decision:**"),
            ("Rationale", "**Rationale:**"),
            ("Rejected alternatives", "**Rejected alternatives:**"),
            ("Test coverage", "**Test coverage:**"),
        ];

        for (name, marker) in &required_sections {
            if !content.contains(marker) && !content.to_lowercase().contains(&name.to_lowercase()) {
                findings.push(format!(
                    "warning: {} missing required section: {}",
                    adr_path, name
                ));
            }
        }

        // Check for status field
        if !content.contains("status:") {
            findings.push(format!(
                "warning: {} missing status field in front-matter",
                adr_path
            ));
        }

        // Check for linked features
        if content.contains("features: []") || !content.contains("features:") {
            findings.push(format!(
                "warning: {} has no linked features",
                adr_path
            ));
        }
    }

    if findings.is_empty() {
        findings.push(format!(
            "Reviewed {} staged ADR(s) — no structural issues found.",
            staged_adrs.len()
        ));
    }

    Ok(findings)
}

/// Auto-commit changed docs after a successful authoring session
fn auto_commit(session_type: &SessionType, root: &Path) {
    // Check for changes in docs/
    let status_output = Command::new("git")
        .args(["status", "--porcelain", "docs/"])
        .current_dir(root)
        .output();

    let changed_files = match status_output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let files: Vec<String> = stdout
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.to_string())
                .collect();
            files
        }
        Err(_) => return,
    };

    if changed_files.is_empty() {
        println!("No artifact changes to commit.");
        return;
    }

    // Extract artifact IDs from changed filenames for the commit message
    let mut ids: Vec<String> = changed_files.iter()
        .filter_map(|line| {
            // porcelain format: "XY filename" — extract the filename part
            let path = line.get(3..)?.trim();
            let fname = std::path::Path::new(path).file_stem()?.to_str()?;
            // ID is the prefix before the first dash-separated word (e.g. FT-030 from FT-030-title)
            let parts: Vec<&str> = fname.splitn(3, '-').collect();
            if parts.len() >= 2 {
                Some(format!("{}-{}", parts[0], parts[1]))
            } else {
                None
            }
        })
        .collect();
    ids.sort();
    ids.dedup();

    let id_summary = if ids.len() <= 5 {
        ids.join(", ")
    } else {
        format!("{} artifacts", ids.len())
    };

    let message = format!(
        "author({}): {}\n\nAuto-committed by product author session.",
        session_type, id_summary
    );

    // git add docs/
    let add = Command::new("git")
        .args(["add", "docs/"])
        .current_dir(root)
        .status();
    if !matches!(add, Ok(s) if s.success()) {
        eprintln!("Failed to stage changes. Commit manually.");
        return;
    }

    // git commit
    let commit = Command::new("git")
        .args(["commit", "-m", &message])
        .current_dir(root)
        .status();

    match commit {
        Ok(s) if s.success() => {
            println!("Committed: {}", message.lines().next().unwrap_or(&message));
        }
        _ => {
            eprintln!("Commit failed. Changes are staged — commit manually.");
        }
    }
}

fn default_prompt(session_type: &SessionType) -> String {
    match session_type {
        SessionType::Feature => r#"# Product Authoring Session: Feature

Before writing any content:
1. Call product_feature_list to understand what features exist
2. Call product_graph_central to identify the top-5 foundational ADRs
3. Call product_context on the most related existing feature (if any)
4. Ask the user clarifying questions based on what you found

Only after completing these steps should you scaffold any files.
After scaffolding, run product_graph_check to verify structural health.
"#.to_string(),
        SessionType::Adr => r#"# Product Authoring Session: ADR

Before writing any content:
1. Call product_graph_central — read the top-5 ADRs by centrality
2. Call product_adr_list to see what decisions already exist
3. Call product_impact on the area you're about to decide — understand blast radius
4. Check for potential contradictions with existing linked ADRs

Every ADR must include: Context, Decision, Rationale, Rejected alternatives, Test coverage.
Do not end the session without all five sections present.
"#.to_string(),
        SessionType::Review => r#"# Product Authoring Session: Review

Your goal is to improve specification coverage without adding new features.
Start by:
1. Call product_graph_check — fix structural issues first
2. Walk features by lowest test coverage — propose test criteria
3. Find orphaned ADRs — propose feature links
4. Find features with no exit-criteria TC — propose them

Do not create new features or ADRs unless fixing a specific identified gap.
"#.to_string(),
    }
}

fn schema_prompt() -> String {
    r#"# Artifact Schemas

All artifacts use YAML front-matter between `---` delimiters, followed by a markdown body.

## Feature (FT-XXX)

```yaml
---
id: FT-001
title: Feature Title
phase: 1                    # u32, default 1
status: planned             # planned | in-progress | complete | abandoned
depends-on: []              # FT-NNN references
adrs: []                    # ADR-NNN references
tests: []                   # TC-NNN references
domains: []                 # concern domain names
domains-acknowledged: {}    # domain -> reason (for intentional gaps)
---
```

Body: free-form markdown. Typically starts with `## Description`.

## ADR (ADR-XXX)

```yaml
---
id: ADR-001
title: Decision Title
status: proposed            # proposed | accepted | superseded | abandoned
features: []                # FT-NNN references
supersedes: []              # ADR-NNN references
superseded-by: []           # ADR-NNN references
domains: []                 # concern domain names
scope: feature-specific     # cross-cutting | domain | feature-specific
---
```

Body **must** contain these five sections:
- **Context:** — the problem or situation
- **Decision:** — what was decided
- **Rationale:** — why this decision
- **Rejected alternatives:** — what was considered but not chosen
- **Test coverage:** — how to verify the decision

## Test Criterion (TC-XXX)

```yaml
---
id: TC-001
title: test_name_snake_case
type: scenario              # scenario | invariant | chaos | exit-criteria
status: unimplemented       # unimplemented | implemented | passing | failing
validates:
  features: []              # FT-NNN references
  adrs: []                  # ADR-NNN references
phase: 1                    # u32, default 1
---
```

Body: prose description of what the test verifies.
"#.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn default_prompts_not_empty() {
        assert!(!default_prompt(&SessionType::Feature).is_empty());
        assert!(!default_prompt(&SessionType::Adr).is_empty());
        assert!(!default_prompt(&SessionType::Review).is_empty());
    }

    #[test]
    fn schema_prompt_covers_all_fields() {
        let doc = schema_prompt();

        // Feature fields
        let feature = FeatureFrontMatter {
            id: "FT-000".into(),
            title: "t".into(),
            phase: 1,
            status: FeatureStatus::Planned,
            depends_on: vec![],
            adrs: vec![],
            tests: vec![],
            domains: vec![],
            domains_acknowledged: Default::default(),
            bundle: None,
        };
        let yaml = serde_yaml::to_string(&feature).unwrap();
        for line in yaml.lines() {
            if let Some(key) = line.split(':').next() {
                let key = key.trim();
                if !key.is_empty() && key != "---" {
                    assert!(doc.contains(key), "schema_prompt missing feature field: {}", key);
                }
            }
        }

        // ADR fields
        let adr = AdrFrontMatter {
            id: "ADR-000".into(),
            title: "t".into(),
            status: AdrStatus::Proposed,
            features: vec![],
            supersedes: vec![],
            superseded_by: vec![],
            domains: vec![],
            scope: AdrScope::FeatureSpecific,
        };
        let yaml = serde_yaml::to_string(&adr).unwrap();
        for line in yaml.lines() {
            if let Some(key) = line.split(':').next() {
                let key = key.trim();
                if !key.is_empty() && key != "---" {
                    assert!(doc.contains(key), "schema_prompt missing ADR field: {}", key);
                }
            }
        }

        // Test criterion fields
        let tc = TestFrontMatter {
            id: "TC-000".into(),
            title: "t".into(),
            test_type: TestType::Scenario,
            status: TestStatus::Unimplemented,
            validates: ValidatesBlock { features: vec![], adrs: vec![] },
            phase: 1,
        };
        let yaml = serde_yaml::to_string(&tc).unwrap();
        for line in yaml.lines() {
            if let Some(key) = line.split(':').next() {
                let key = key.trim();
                if !key.is_empty() && key != "---" {
                    assert!(doc.contains(key), "schema_prompt missing TC field: {}", key);
                }
            }
        }
    }
}
