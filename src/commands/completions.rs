//! Shell completion generation for bash, zsh, fish.

use std::process;

use super::BoxResult;

pub(crate) fn handle_completions(shell: &str, cmd: &mut clap::Command) -> BoxResult {
    use clap_complete::{generate, Shell};

    let shell = match shell.to_lowercase().as_str() {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        other => {
            eprintln!("Unknown shell: {}. Use: bash, zsh, fish", other);
            process::exit(1);
        }
    };

    generate(shell, cmd, "product", &mut std::io::stdout());
    Ok(())
}
