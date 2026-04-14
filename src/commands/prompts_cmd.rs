//! Prompts management — init, list, get.

use clap::Subcommand;
use product_lib::{author, config::ProductConfig};

use super::BoxResult;

#[derive(Subcommand)]
pub enum PromptsCommands {
    /// Initialize default prompt files in benchmarks/prompts/
    Init,
    /// List available prompts with version numbers
    List,
    /// Print a prompt to stdout (for piping to agents)
    Get {
        /// Prompt name (e.g. author-feature, author-adr, author-review, implement)
        name: String,
    },
}

pub(crate) fn handle_prompts(cmd: PromptsCommands) -> BoxResult {
    let (_config, root) = ProductConfig::discover()?;
    match cmd {
        PromptsCommands::Init => prompts_init(&root),
        PromptsCommands::List => prompts_list(&root),
        PromptsCommands::Get { name } => prompts_get(&root, &name),
    }
}

fn prompts_init(root: &std::path::Path) -> BoxResult {
    let created = author::prompts_init(root)?;
    if created.is_empty() {
        println!("All prompt files already exist.");
    } else {
        for f in &created {
            println!("  created: benchmarks/prompts/{}", f);
        }
        println!("{} prompt file(s) created.", created.len());
    }
    Ok(())
}

fn prompts_list(root: &std::path::Path) -> BoxResult {
    let prompts = author::prompts_list(root);
    println!("{:<20} {:<8} FILE", "NAME", "VERSION");
    println!("{}", "-".repeat(60));
    for p in &prompts {
        println!("{:<20} v{:<7} {}", p.name, p.version, p.filename);
    }
    Ok(())
}

fn prompts_get(root: &std::path::Path, name: &str) -> BoxResult {
    let content = author::prompts_get(root, name)?;
    print!("{}", content);
    Ok(())
}
