//! `product init` helpers — layout descriptors, TOML rendering, gitignore
//! management, and the interactive prompt loop. Extracted from
//! `commands/init.rs` to keep both files under the 400-line fitness limit.

use product_lib::{error::ProductError, fileops};

use super::BoxResult;

/// Filesystem layout produced by `product init`.
pub(crate) struct Layout {
    pub config: &'static str,
    pub features: &'static str,
    pub adrs: &'static str,
    pub tests: &'static str,
    pub graph: &'static str,
    pub checklist: &'static str,
    pub extra_paths: &'static [(&'static str, &'static str)],
    pub gitignore_graph: &'static str,
    pub gitignore_checklist: &'static str,
    pub gitignore_extra: Option<&'static str>,
}

pub(crate) const CANONICAL: Layout = Layout {
    config: ".product/config.toml",
    features: ".product/features",
    adrs: ".product/adrs",
    tests: ".product/tests",
    graph: ".product/graph",
    checklist: ".product/checklist.md",
    extra_paths: &[
        ("dependencies", ".product/dependencies"),
        ("requests", ".product/requests.jsonl"),
        ("prompts", ".product/prompts"),
        ("gaps", ".product/gaps.json"),
    ],
    gitignore_graph: ".product/graph/",
    gitignore_checklist: ".product/checklist.md",
    gitignore_extra: Some(".product/sessions/"),
};

pub(crate) const LEGACY: Layout = Layout {
    config: "product.toml",
    features: "docs/features",
    adrs: "docs/adrs",
    tests: "docs/tests",
    graph: "docs/graph",
    checklist: "docs/checklist.md",
    extra_paths: &[],
    gitignore_graph: "docs/graph/",
    gitignore_checklist: "docs/checklist.md",
    gitignore_extra: None,
};

pub(crate) fn parse_cli_domains(cli_domains: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for d in cli_domains {
        if let Some((k, v)) = d.split_once('=') {
            out.push((k.trim().to_string(), v.trim().to_string()));
        } else {
            out.push((d.trim().to_string(), String::new()));
        }
    }
    out
}

pub(crate) struct InteractiveAnswers {
    pub project_name: String,
    pub responsibility: Option<String>,
    pub mcp_write: bool,
    pub mcp_port: u16,
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn run_interactive_prompts(
    name: Option<String>,
    description: Option<String>,
    preserved_responsibility: Option<String>,
    default_name: String,
    port: u16,
    domains: &mut Vec<(String, String)>,
) -> Result<InteractiveAnswers, Box<dyn std::error::Error>> {
    use std::io::{BufRead, Write};
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();
    let mut reader = stdin.lock();
    let mut out = stdout.lock();

    let name_default = name.unwrap_or(default_name);
    write!(out, "Project name [{}]: ", name_default)?;
    out.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();
    let project_name = if trimmed.is_empty() {
        name_default
    } else {
        trimmed.to_string()
    };

    let resp_default = description.clone().or_else(|| preserved_responsibility.clone());
    if let Some(ref d) = resp_default {
        write!(out, "Product description [{}]: ", d)?;
    } else {
        writeln!(out, "\nProduct description \u{2014} a single statement of what the product is and is not (FT-039).")?;
        write!(out, "Description (blank to skip): ")?;
    }
    out.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();
    let responsibility = if trimmed.is_empty() {
        resp_default
    } else {
        Some(trimmed.to_string())
    };

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
    for (i, (n, desc)) in common_domains.iter().enumerate() {
        writeln!(out, "  [{}] {} \u{2014} {}", i + 1, n, desc)?;
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
                    if !domains.iter().any(|(dk, _)| dk == k) {
                        domains.push((k.to_string(), v.to_string()));
                    }
                }
            }
        }
    }

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

    write!(out, "\nEnable MCP write tools by default? [y/N]: ")?;
    out.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let mcp_write = matches!(line.trim().to_lowercase().as_str(), "y" | "yes");

    write!(out, "MCP HTTP port [{}]: ", port)?;
    out.flush()?;
    let mut line = String::new();
    reader.read_line(&mut line)?;
    let trimmed = line.trim();
    let mcp_port = if trimmed.is_empty() {
        port
    } else {
        trimmed.parse::<u16>().unwrap_or(port)
    };

    Ok(InteractiveAnswers { project_name, responsibility, mcp_write, mcp_port })
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_toml(
    project_name: &str,
    checklist_in_gitignore: bool,
    responsibility: Option<&str>,
    domains: &[(String, String)],
    mcp_write: bool,
    mcp_port: u16,
    layout: &Layout,
) -> String {
    let domains_section = if domains.is_empty() {
        String::new()
    } else {
        domains
            .iter()
            .map(|(k, v)| {
                let safe_key = k.replace('"', "");
                let safe_val = v.replace('"', "\\\"");
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

    let product_section = match responsibility {
        Some(r) if !r.trim().is_empty() => {
            let escaped = r.replace('\\', "\\\\").replace('"', "\\\"");
            format!("[product]\nresponsibility = \"{}\"\n\n", escaped)
        }
        _ => "[product]\n# responsibility \u{2014} single statement of what the product is and is not (FT-039)\nresponsibility = \"\"\n\n".to_string(),
    };

    let mut paths_block = format!(
        "[paths]\nfeatures = \"{}\"\nadrs = \"{}\"\ntests = \"{}\"\ngraph = \"{}\"\nchecklist = \"{}\"\n",
        layout.features, layout.adrs, layout.tests, layout.graph, layout.checklist
    );
    for (k, v) in layout.extra_paths {
        paths_block.push_str(&format!("{} = \"{}\"\n", k, v));
    }

    format!(
        r#"name = "{name}"
schema-version = "1"
checklist-in-gitignore = {clg}

{product}{paths}
[prefixes]
feature = "FT"
adr = "ADR"
test = "TC"

[phases]
1 = "Phase 1"

[domains]
{domains}
[mcp]
write = {mcp_write}
port = {mcp_port}

[author]
cli = "claude"

# FT-055 / ADR-047 — feature body completeness check (W030).
# Defaults are listed explicitly to future-proof against changes upstream.
[features]
required-sections = ["Description", "Functional Specification", "Out of scope"]
functional-spec-subsections = ["Inputs", "Outputs", "State", "Behaviour", "Invariants", "Error handling", "Boundaries"]
required-from-phase = 1
completeness-severity = "warning"
"#,
        name = project_name,
        clg = checklist_in_gitignore,
        product = product_section,
        paths = paths_block,
        domains = domains_section,
        mcp_write = mcp_write,
        mcp_port = mcp_port,
    )
}

pub(crate) fn manage_gitignore(
    path: &std::path::Path,
    checklist_in_gitignore: bool,
    layout: &Layout,
) -> BoxResult {
    let mut entries_to_add: Vec<&str> = vec![layout.gitignore_graph];
    if let Some(extra) = layout.gitignore_extra {
        entries_to_add.push(extra);
    }
    if checklist_in_gitignore {
        entries_to_add.push(layout.gitignore_checklist);
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
