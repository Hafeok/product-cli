//! Repository initialization (ADR-033, ADR-048, FT-057).
//!
//! Default layout writes `.product/config.toml` plus
//! `.product/{features,adrs,tests,graph}/`. `--legacy-layout` opts into the
//! pre-FT-057 root-based scheme (`product.toml` + `docs/...`).

use product_core::{config::ProductConfig, error::ProductError, fileops};
use std::path::{Path, PathBuf};

use super::init_helpers::{
    build_toml, parse_cli_domains, run_interactive_prompts, Layout, CANONICAL, LEGACY,
};
use super::BoxResult;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_init(
    yes: bool,
    force: bool,
    name: Option<String>,
    description: Option<String>,
    cli_domains: Vec<String>,
    port: u16,
    write_tools: bool,
    legacy_layout: bool,
    path: Option<PathBuf>,
    demo: bool,
    no_skills: bool,
) -> BoxResult {
    let target_dir = resolve_target_dir(path.as_deref())?;
    let layout: &Layout = if legacy_layout { &LEGACY } else { &CANONICAL };
    let config_path = target_dir.join(layout.config);

    let preserved_responsibility = check_existing_config(&config_path, force)?;

    let default_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let mut domains = parse_cli_domains(&cli_domains);

    let project_name;
    let responsibility;
    let mcp_write;
    let mcp_port;

    if yes {
        project_name = name.unwrap_or(default_name);
        responsibility = description.or(preserved_responsibility);
        mcp_write = write_tools;
        mcp_port = port;
    } else {
        let answers = run_interactive_prompts(
            name,
            description,
            preserved_responsibility,
            default_name,
            port,
            &mut domains,
        )?;
        project_name = answers.project_name;
        responsibility = answers.responsibility;
        mcp_write = answers.mcp_write;
        mcp_port = answers.mcp_port;
    }

    let toml_content = build_toml(
        &project_name,
        responsibility.as_deref(),
        &domains,
        mcp_write,
        mcp_port,
    );

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", parent.display(), e))
        })?;
    }
    fileops::write_file_atomic(&config_path, &toml_content)?;
    println!("Created:");
    println!("  {}", layout.config);

    if !no_skills {
        let base = target_dir.join(".claude").join("skills");
        // Don't clobber a user's existing skill edits on re-init (overwrite = false).
        let written = super::skills::write_skills(&base, false)?;
        for name in &written {
            println!("  .claude/skills/{name}/SKILL.md");
        }
    }

    if demo {
        let n = product_core::demo::seed_bookstore(&target_dir, &project_name)?;
        println!("\nSeeded the bookstore demo — {n} What nodes.");
        print_next_steps(true);
    } else {
        print_next_steps(false);
    }
    Ok(())
}

/// Signpost the framework graph (What → How → Delivery) after init.
/// `product guide` is the through-line.
fn print_next_steps(demo: bool) {
    println!("\nNext steps:");
    if demo {
        println!("  product guide               # your journey checklist + the next step");
        println!("  product domain list         # the seeded What nodes, by kind");
        println!("  product domain show Order   # inspect a node and its links");
        println!("  product domain validate --strict   # check graph completeness");
    } else {
        println!("  product guide               # model your product (What → How → Delivery)");
        println!("  product author domain <name>       # facilitated What-capture session");
        println!("  product domain new system <id> …   # or capture the What directly");
    }
}

fn resolve_target_dir(path: Option<&Path>) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(p) = path {
        std::fs::create_dir_all(p).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", p.display(), e))
        })?;
        let canonical = p.canonicalize().map_err(|e| {
            ProductError::ConfigError(format!("Cannot resolve path {}: {}", p.display(), e))
        })?;
        Ok(canonical)
    } else {
        std::env::current_dir().map_err(|e| {
            Box::new(ProductError::ConfigError(format!(
                "Cannot determine working directory: {}",
                e
            ))) as Box<dyn std::error::Error>
        })
    }
}

fn check_existing_config(
    config_path: &Path,
    force: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !config_path.exists() {
        return Ok(None);
    }
    if !force {
        return Err(Box::new(ProductError::ConfigError(format!(
            "{} already exists\n  --> {}\n  = hint: use `product init --force` to overwrite, or edit the file directly",
            config_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("config"),
            config_path.display()
        ))));
    }
    match ProductConfig::load(config_path) {
        Ok(c) => Ok(c.responsibility().map(|s| s.to_string())),
        Err(_) => Ok(None),
    }
}
