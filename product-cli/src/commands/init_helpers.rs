//! `product init` helpers — the config layout, TOML rendering, the interactive
//! prompt loop. Extracted from `commands/init.rs` to keep both files small.



/// Where `product init` writes the config. The framework graph itself lives
/// under `.product/` and its dirs are created lazily by the domain sessions.
pub(crate) struct Layout {
    pub config: &'static str,
}

pub(crate) const CANONICAL: Layout = Layout { config: ".product/config.toml" };

pub(crate) const LEGACY: Layout = Layout { config: "product.toml" };

pub(crate) struct InteractiveAnswers {
    pub project_name: String,
    pub mcp_write: bool,
    pub mcp_port: u16,
}

pub(crate) fn run_interactive_prompts(
    name: Option<String>,
    default_name: String,
    port: u16,
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

    Ok(InteractiveAnswers { project_name, mcp_write, mcp_port })
}

pub(crate) fn build_toml(
    project_name: &str,
    mcp_write: bool,
    mcp_port: u16,
    author_cli: Option<&str>,
) -> String {
    // `[author].cli` — the default agent CLI for `product session start`.
    // Active when a value is supplied; commented otherwise for discoverability.
    let author_block = match author_cli.map(str::trim).filter(|c| !c.is_empty()) {
        Some(c) => format!(
            "[author]\n# default agent CLI for `product session start`: claude | copilot\ncli = \"{c}\"\n\n"
        ),
        None => "[author]\n# default agent CLI for `product session start`: claude | copilot\n# cli = \"claude\"\n\n".to_string(),
    };

    format!(
        r#"name = "{name}"
schema-version = "1"

{author}[mcp]
write = {mcp_write}
port = {mcp_port}
"#,
        name = project_name,
        author = author_block,
        mcp_write = mcp_write,
        mcp_port = mcp_port,
    )
}

#[cfg(test)]
mod build_toml_tests {
    use super::build_toml;
    use product_core::config::ProductConfig;

    #[test]
    fn cli_some_emits_active_author_block_and_parses() {
        let toml = build_toml("app", false, 7777, Some("copilot"));
        assert!(toml.contains("[author]"));
        assert!(toml.contains("cli = \"copilot\""));
        let config: ProductConfig = toml::from_str(&toml).expect("valid config");
        assert_eq!(config.author_cli().as_deref(), Some("copilot"));
    }

    #[test]
    fn cli_none_emits_commented_author_block() {
        let toml = build_toml("app", false, 7777, None);
        assert!(toml.contains("[author]"));
        assert!(toml.contains("# cli = \"claude\""));
        // Commented out → no active default.
        let config: ProductConfig = toml::from_str(&toml).expect("valid config");
        assert_eq!(config.author_cli(), None);
    }

    #[test]
    fn generated_config_is_minimal_name_author_mcp() {
        let toml = build_toml("shop", true, 8080, None);
        assert!(!toml.contains("[product]"), "no responsibility section anymore");
        assert!(!toml.contains("[domains]"), "no domains section anymore");
        let config: ProductConfig = toml::from_str(&toml).expect("valid config");
        assert_eq!(config.product_name(), "shop");
    }
}
