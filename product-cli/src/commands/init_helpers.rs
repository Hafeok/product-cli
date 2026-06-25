//! `product init` helpers — the config layout, TOML rendering, the interactive
//! prompt loop. Extracted from `commands/init.rs` to keep both files small.



/// Where `product init` writes the config. The framework graph itself lives
/// under `.product/` and its dirs are created lazily by the domain sessions.
pub(crate) struct Layout {
    pub config: &'static str,
}

pub(crate) const CANONICAL: Layout = Layout { config: ".product/config.toml" };

pub(crate) const LEGACY: Layout = Layout { config: "product.toml" };

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

pub(crate) fn build_toml(
    project_name: &str,
    responsibility: Option<&str>,
    domains: &[(String, String)],
    mcp_write: bool,
    mcp_port: u16,
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
        _ => "[product]\n# responsibility \u{2014} a single statement of what the product is and is not\nresponsibility = \"\"\n\n".to_string(),
    };

    let domains_block = if domains_section.is_empty() {
        String::new()
    } else {
        format!("[domains]\n{domains_section}\n")
    };

    format!(
        r#"name = "{name}"
schema-version = "1"

{product}{domains}[mcp]
write = {mcp_write}
port = {mcp_port}
"#,
        name = project_name,
        product = product_section,
        domains = domains_block,
        mcp_write = mcp_write,
        mcp_port = mcp_port,
    )
}
