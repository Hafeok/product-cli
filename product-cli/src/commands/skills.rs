//! Materialize the embedded workflow skills into a Claude Code skills dir.
//!
//! The four What→How→Build phase-guide skills are compiled into the binary
//! with `include_str!`, so a downstream user who installs `product` and runs
//! `product init` gets them without a separate download. `product skills
//! install` (re)writes them into `.claude/skills/` of the current repo, or
//! `~/.claude/skills/` with `--global`.

use clap::Subcommand;
use product_core::{config::ProductConfig, error::ProductError, fileops};
use std::path::{Path, PathBuf};

use super::BoxResult;

/// The workflow skills shipped with the binary: `(name, SKILL.md contents)`.
/// Source of truth lives in this repo's `.claude/skills/<name>/SKILL.md`; the
/// maintainer-only skills (`release`, the `*-e2e` shake-out skills) are not
/// shipped. Keep this list in sync if a new user-facing phase skill lands.
const SKILLS: &[(&str, &str)] = &[
    ("product-session", include_str!("../../../.claude/skills/product-session/SKILL.md")),
    ("product-what", include_str!("../../../.claude/skills/product-what/SKILL.md")),
    ("product-how", include_str!("../../../.claude/skills/product-how/SKILL.md")),
    ("product-build", include_str!("../../../.claude/skills/product-build/SKILL.md")),
];

#[derive(Subcommand)]
pub enum SkillsCommands {
    /// Write the bundled What→How→Build skills into a Claude Code skills dir
    Install {
        /// Install into `~/.claude/skills/` (all projects) instead of this repo
        #[arg(long)]
        global: bool,
    },
}

pub(crate) fn handle_skills(cmd: SkillsCommands) -> BoxResult {
    match cmd {
        SkillsCommands::Install { global } => install(global),
    }
}

fn install(global: bool) -> BoxResult {
    let base = skills_base(global)?;
    // An explicit install refreshes the shipped content (overwrite = true).
    let written = write_skills(&base, true)?;
    println!("Installed {} skill(s) into {}:", written.len(), base.display());
    for name in written {
        println!("  {name}");
    }
    if !global {
        println!("\nStart a new Claude Code session to pick them up, then try `/product-session`.");
    }
    Ok(())
}

/// Write each bundled skill to `<base>/<name>/SKILL.md`, returning the names
/// written. With `overwrite = false`, a skill whose `SKILL.md` already exists
/// is left untouched (so `init` never clobbers a user's edits) and omitted
/// from the returned list.
pub(crate) fn write_skills(base: &Path, overwrite: bool) -> Result<Vec<&'static str>, ProductError> {
    let mut written = Vec::new();
    for (name, contents) in SKILLS {
        let path = base.join(name).join("SKILL.md");
        if !overwrite && path.exists() {
            continue;
        }
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                ProductError::IoError(format!("failed to create {}: {}", parent.display(), e))
            })?;
        }
        fileops::write_file_atomic(&path, contents)?;
        written.push(*name);
    }
    Ok(written)
}

/// Resolve the target skills directory: `~/.claude/skills/` when `global`,
/// otherwise `.claude/skills/` under the discovered repo root.
fn skills_base(global: bool) -> Result<PathBuf, Box<dyn std::error::Error>> {
    if global {
        let home = std::env::var("HOME").map_err(|_| {
            ProductError::ConfigError("cannot resolve ~ — HOME is unset".into())
        })?;
        Ok(PathBuf::from(home).join(".claude").join("skills"))
    } else {
        let (_config, root) = ProductConfig::discover()?;
        Ok(root.join(".claude").join("skills"))
    }
}

#[cfg(test)]
#[path = "skills_tests.rs"]
mod tests;
