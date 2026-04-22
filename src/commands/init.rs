//! Repository initialization (ADR-033).

use product_lib::{config::ProductConfig, error::ProductError, fileops};
use std::path::PathBuf;

use super::BoxResult;

pub(crate) fn handle_init(
    yes: bool,
    force: bool,
    name: Option<String>,
    cli_domains: Vec<String>,
    port: u16,
    write_tools: bool,
    path: Option<PathBuf>,
) -> BoxResult {
    let target_dir = if let Some(ref p) = path {
        std::fs::create_dir_all(p).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", p.display(), e))
        })?;
        p.canonicalize().map_err(|e| {
            ProductError::ConfigError(format!("Cannot resolve path {}: {}", p.display(), e))
        })?
    } else {
        std::env::current_dir().map_err(|e| {
            ProductError::ConfigError(format!("Cannot determine working directory: {}", e))
        })?
    };
    let toml_path = target_dir.join("product.toml");

    // Determine checklist-in-gitignore setting.
    // If --force and product.toml exists, preserve the existing setting.
    let checklist_in_gitignore = if toml_path.exists() {
        if !force {
            return Err(Box::new(ProductError::ConfigError(format!(
                "product.toml already exists\n  --> {}\n  = hint: use `product init --force` to overwrite, or edit the file directly",
                toml_path.display()
            ))));
        }
        // --force: read existing config to preserve checklist-in-gitignore
        ProductConfig::load(&toml_path)
            .map(|c| c.checklist_in_gitignore)
            .unwrap_or(true)
    } else {
        true
    };

    // Default project name from directory name
    let default_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    // Parse CLI domain flags: --domain key=value
    let mut domains: Vec<(String, String)> = Vec::new();
    for d in &cli_domains {
        if let Some((k, v)) = d.split_once('=') {
            domains.push((k.trim().to_string(), v.trim().to_string()));
        } else {
            domains.push((d.trim().to_string(), String::new()));
        }
    }

    let project_name;
    let mcp_write;
    let mcp_port;

    if yes {
        // Non-interactive mode: use defaults and CLI overrides
        project_name = name.unwrap_or(default_name);
        mcp_write = write_tools;
        mcp_port = port;
    } else {
        // Interactive mode: prompt user via stdin/stdout
        use std::io::{BufRead, Write};
        let stdin = std::io::stdin();
        let stdout = std::io::stdout();
        let mut reader = stdin.lock();
        let mut out = stdout.lock();

        // Project name
        let name_default = name.unwrap_or(default_name);
        write!(out, "Project name [{}]: ", name_default)?;
        out.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim();
        project_name = if trimmed.is_empty() {
            name_default
        } else {
            trimmed.to_string()
        };

        // Domains — prompt for common domains
        let common_domains = [
            ("security", "Authentication, authorisation, secrets, trust boundaries"),
            ("error-handling", "Error model, diagnostics, exit codes, recovery"),
            ("storage", "Persistence, durability, backup"),
            ("networking", "DNS, mTLS, service discovery, port allocation"),
            ("api", "CLI surface, MCP tools, event schema"),
            ("observability", "Metrics, tracing, logging, telemetry"),
            ("data-model", "RDF, SPARQL, ontology, event sourcing"),
        ];

        writeln!(out, "\nCommon concern domains (enter numbers separated by spaces, or blank to skip):")?;
        for (i, (name, desc)) in common_domains.iter().enumerate() {
            writeln!(out, "  [{}] {} \u{2014} {}", i + 1, name, desc)?;
        }
        write!(out, "Select domains: ")?;
        out.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            for token in trimmed.split_whitespace() {
                if let Ok(idx) = token.parse::<usize>() {
                    if idx >= 1 && idx <= common_domains.len() {
                        let (k, v) = common_domains[idx - 1];
                        // Avoid duplicates from CLI flags
                        if !domains.iter().any(|(dk, _)| dk == k) {
                            domains.push((k.to_string(), v.to_string()));
                        }
                    }
                }
            }
        }

        // Custom domains
        loop {
            write!(out, "Add custom domain? (name=description, or enter to skip): ")?;
            out.flush()?;
            let mut line = String::new();
            reader.read_line(&mut line)?;
            let trimmed = line.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some((k, v)) = trimmed.split_once('=') {
                domains.push((k.trim().to_string(), v.trim().to_string()));
            } else {
                domains.push((trimmed.to_string(), String::new()));
            }
        }

        // MCP settings
        write!(out, "\nEnable MCP write tools by default? [y/N]: ")?;
        out.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        mcp_write = matches!(line.trim().to_lowercase().as_str(), "y" | "yes");

        write!(out, "MCP HTTP port [{}]: ", port)?;
        out.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim();
        mcp_port = if trimmed.is_empty() {
            port
        } else {
            trimmed.parse::<u16>().unwrap_or(port)
        };
    }

    // Build domains TOML section
    let domains_section = if domains.is_empty() {
        String::new()
    } else {
        domains
            .iter()
            .map(|(k, v)| {
                // Escape any quotes in key or value for TOML safety
                let safe_key = k.replace('\"', "");
                let safe_val = v.replace('\"', "\\\"");
                if safe_val.is_empty() {
                    format!("\"{}\" = \"\"", safe_key)
                } else {
                    format!("\"{}\" = \"{}\"", safe_key, safe_val)
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
            + "\n"
    };

    // Generate product.toml
    let toml_content = format!(
        r#"name = "{project_name}"
schema-version = "1"
checklist-in-gitignore = {checklist_in_gitignore}

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

[phases]
1 = "Phase 1"

[domains]
{domains_section}
[mcp]
write = {mcp_write}
port = {mcp_port}

[author]
cli = "claude"
"#
    );
    fileops::write_file_atomic(&toml_path, &toml_content)?;
    println!("Created:");
    println!("  product.toml");

    // Create directory skeleton
    let dirs = ["docs/features", "docs/adrs", "docs/tests", "docs/graph"];
    for d in &dirs {
        let dir_path = target_dir.join(d);
        std::fs::create_dir_all(&dir_path).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", dir_path.display(), e))
        })?;
        println!("  {}/", d);
    }

    // Manage .gitignore
    let gitignore_path = target_dir.join(".gitignore");
    init_manage_gitignore(&gitignore_path, checklist_in_gitignore)?;

    println!("\nRun `product feature new \"My First Feature\"` to get started.");
    Ok(())
}

/// Manage .gitignore entries for generated files (ADR-033, ADR-007).
/// Always adds `docs/graph/`. Adds `docs/checklist.md` only when checklist_in_gitignore is true.
fn init_manage_gitignore(path: &std::path::Path, checklist_in_gitignore: bool) -> BoxResult {
    let mut entries_to_add: Vec<&str> = vec!["docs/graph/"];
    if checklist_in_gitignore {
        entries_to_add.push("docs/checklist.md");
    }

    let existing = if path.exists() {
        std::fs::read_to_string(path).map_err(|e| {
            ProductError::IoError(format!("failed to read {}: {}", path.display(), e))
        })?
    } else {
        String::new()
    };

    let mut lines: Vec<String> = if existing.is_empty() {
        Vec::new()
    } else {
        existing.lines().map(String::from).collect()
    };

    let has_header = lines.iter().any(|l| l.contains("Product CLI"));
    let mut added_any = false;

    for entry in &entries_to_add {
        if !lines.iter().any(|l| l.trim() == *entry) {
            if !added_any && !has_header {
                if !lines.is_empty() && lines.last().map(|l| !l.is_empty()).unwrap_or(false) {
                    lines.push(String::new());
                }
                lines.push("# Product CLI \u{2014} generated files".to_string());
            }
            lines.push(entry.to_string());
            added_any = true;
        }
    }

    let mut content = lines.join("\n");
    if !content.ends_with('\n') {
        content.push('\n');
    }
    fileops::write_file_atomic(path, &content)?;
    println!("  .gitignore");
    Ok(())
}
