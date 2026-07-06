//! Repository initialization (ADR-033, ADR-048, FT-057).
//!
//! Default layout writes `.product/config.toml` plus the product's home,
//! `.product/products/<name>/` (further products get theirs via `product
//! product new`). `--legacy-layout` opts into the pre-FT-057 root-based
//! scheme (`product.toml` + `docs/...`).

use product_core::{error::ProductError, fileops};
use std::path::{Path, PathBuf};

use super::init_helpers::{build_toml, run_interactive_prompts, Layout, CANONICAL, LEGACY};
use super::BoxResult;

#[allow(clippy::too_many_arguments)]
pub(crate) fn handle_init(
    yes: bool,
    force: bool,
    name: Option<String>,
    port: u16,
    write_tools: bool,
    legacy_layout: bool,
    path: Option<PathBuf>,
    demo: bool,
    no_skills: bool,
    cli: Option<String>,
) -> BoxResult {
    // Validate the agent CLI choice up front so a typo fails before we write.
    if let Some(c) = &cli {
        product_core::author::AgentCli::parse(c)?;
    }
    let target_dir = resolve_target_dir(path.as_deref())?;
    let layout: &Layout = if legacy_layout { &LEGACY } else { &CANONICAL };
    let config_path = target_dir.join(layout.config);

    check_existing_config(&config_path, force)?;

    let default_name = target_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-project")
        .to_string();

    let project_name;
    let mcp_write;
    let mcp_port;

    if yes {
        project_name = name.unwrap_or(default_name);
        mcp_write = write_tools;
        mcp_port = port;
    } else {
        let answers = run_interactive_prompts(name, default_name, port)?;
        project_name = answers.project_name;
        mcp_write = answers.mcp_write;
        mcp_port = answers.mcp_port;
    }

    let toml_content = build_toml(&project_name, mcp_write, mcp_port, cli.as_deref());

    if let Some(parent) = config_path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", parent.display(), e))
        })?;
    }
    fileops::write_file_atomic(&config_path, &toml_content)?;
    println!("Created:");
    println!("  {}", layout.config);

    // The product's home — every artifact for it (What graph, How, delivery)
    // lives under `.product/products/<name>/`. Skipped when the project name
    // is not a valid node id (the graph commands would reject it anyway).
    if !legacy_layout && product_core::pf::ids::validate_id(&project_name).is_ok() {
        let home = product_core::pf::paths::product_home(&target_dir, &project_name);
        std::fs::create_dir_all(&home).map_err(|e| {
            ProductError::IoError(format!("failed to create {}: {}", home.display(), e))
        })?;
        println!("  .product/products/{project_name}/");
    }

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

fn check_existing_config(config_path: &Path, force: bool) -> Result<(), Box<dyn std::error::Error>> {
    if config_path.exists() && !force {
        return Err(Box::new(ProductError::ConfigError(format!(
            "{} already exists\n  --> {}\n  = hint: use `product init --force` to overwrite, or edit the file directly",
            config_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("config"),
            config_path.display()
        ))));
    }
    Ok(())
}
