//! Checklist generation from feature files.

use clap::Subcommand;
use product_lib::{checklist, fileops};

use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum ChecklistCommands {
    /// Regenerate checklist.md from feature files
    Generate,
}

pub(crate) fn handle_checklist(cmd: ChecklistCommands) -> BoxResult {
    match cmd {
        ChecklistCommands::Generate => {
            let _lock = acquire_write_lock()?;
            let (config, root, graph) = load_graph()?;
            // Git-aware warning: check for uncommitted artifact files
            fileops::warn_uncommitted_changes(&root);
            let content = checklist::generate(&graph);
            let path = config.resolve_path(&root, &config.paths.checklist);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            fileops::write_file_atomic(&path, &content)?;
            println!("Generated: {}", path.display());
        }
    }
    Ok(())
}
