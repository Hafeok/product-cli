//! Prompts management — init, list, get.

use clap::Subcommand;
use product_lib::{author, config::ProductConfig};

use super::BoxResult;

#[derive(Subcommand)]
pub enum PromptsCommands {
    /// Print a prompt to stdout (for piping to agents)
    Get {
        /// Prompt name (e.g. author-feature, author-adr, author-review, implement)
        name: String,
    },
    /// Initialize default prompt files in benchmarks/prompts/
    Init,
    /// List available prompts with version numbers
    List,
}

pub(crate) fn handle_prompts(cmd: PromptsCommands) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;
    let prompts_path = config.paths.prompts_resolved().to_string();
    match cmd {
        PromptsCommands::Get { name } => prompts_get(&root, &prompts_path, &name),
        PromptsCommands::Init => prompts_init(&root, &prompts_path),
        PromptsCommands::List => prompts_list(&root, &prompts_path),
    }
}

fn prompts_init(root: &std::path::Path, prompts_path: &str) -> BoxResult {
    let created = author::prompts_init(root, prompts_path)?;
    if created.is_empty() {
        println!("All prompt files already exist.");
    } else {
        for f in &created {
            println!("  created: {}/{}", prompts_path, f);
        }
        println!("{} prompt file(s) created.", created.len());
    }
    Ok(())
}

fn prompts_list(root: &std::path::Path, prompts_path: &str) -> BoxResult {
    let prompts = author::prompts_list(root, prompts_path);
    println!("{:<20} {:<8} FILE", "NAME", "VERSION");
    println!("{}", "-".repeat(60));
    for p in &prompts {
        println!("{:<20} v{:<7} {}", p.name, p.version, p.filename);
    }
    Ok(())
}

fn prompts_get(root: &std::path::Path, prompts_path: &str, name: &str) -> BoxResult {
    let content = author::prompts_get(root, prompts_path, name)?;
    print!("{}", content);
    Ok(())
}
