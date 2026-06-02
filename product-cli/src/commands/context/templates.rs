//! `product context templates [--show NAME | --where | --reset NAME]` —
//! template lifecycle commands. Defaults to listing all resolved templates.

use product_lib::context::template::{self, TemplateSource};
use product_lib::error::ProductError;
use std::path::PathBuf;
use std::process;

use super::super::{load_graph, BoxResult};

pub(crate) fn dispatch(
    show: Option<String>,
    where_flag: bool,
    reset: Option<String>,
) -> BoxResult {
    let (config, root, _graph) = load_graph()?;
    if let Some(name) = show {
        return show_template(&root, &name);
    }
    if let Some(name) = reset {
        return reset_template(&root, &name);
    }
    if where_flag {
        return list_where(&root);
    }
    list_templates(&root, &config)
}

fn list_templates(root: &std::path::Path, config: &product_lib::config::ProductConfig) -> BoxResult {
    let outcome = template::resolve_all(root);
    let mut names: Vec<String> = outcome.resolved.keys().cloned().collect();
    names.sort();
    println!("{:<24} {:<10} DESCRIPTION", "NAME", "SOURCE");
    for name in &names {
        let t = match outcome.resolved.get(name) {
            Some(v) => v,
            None => continue,
        };
        println!(
            "{:<24} {:<10} {}",
            t.name,
            format!("({})", t.source.label()),
            t.template.template.description,
        );
    }
    let default = config
        .context
        .default_target
        .clone()
        .unwrap_or_else(|| "human (fallback)".to_string());
    println!("\nDefault target: {}", default);
    if !outcome.warnings.is_empty() {
        eprintln!();
        for (n, p, reason) in &outcome.warnings {
            let path_label = p
                .as_ref()
                .map(|pp| pp.display().to_string())
                .unwrap_or_else(|| "(built-in)".to_string());
            eprintln!(
                "warning[E030]: invalid template {:?} at {}\n   = {}",
                n, path_label, reason
            );
        }
    }
    Ok(())
}

fn list_where(root: &std::path::Path) -> BoxResult {
    let outcome = template::resolve_all(root);
    let mut names: Vec<String> = outcome.resolved.keys().cloned().collect();
    names.sort();
    for name in &names {
        let t = match outcome.resolved.get(name) {
            Some(v) => v,
            None => continue,
        };
        let where_str = match &t.source {
            TemplateSource::Repo(p) | TemplateSource::User(p) => p.display().to_string(),
            TemplateSource::Builtin => "(built-in)".to_string(),
        };
        println!("{:<24} {}", t.name, where_str);
    }
    Ok(())
}

fn show_template(root: &std::path::Path, name: &str) -> BoxResult {
    let outcome = template::resolve_all(root);
    if let Some(t) = outcome.resolved.get(name) {
        print!("{}", t.raw_toml);
        return Ok(());
    }
    // Fallback: show the embedded built-in even if it failed to resolve.
    if let Some(builtin) = template::builtin_toml(name) {
        print!("{}", builtin);
        return Ok(());
    }
    let mut available: Vec<String> = outcome.resolved.keys().cloned().collect();
    available.sort();
    eprintln!(
        "{}",
        ProductError::UnknownTarget {
            name: name.to_string(),
            available,
        }
    );
    process::exit(1);
}

fn reset_template(_root: &std::path::Path, name: &str) -> BoxResult {
    // Resolve only against the user dir — reset never touches repo or built-in.
    let user_dir = match template::resolve::user_templates_dir() {
        Some(d) => d,
        None => {
            eprintln!("error: $HOME is not set; cannot locate user templates dir");
            process::exit(1);
        }
    };
    let candidates: Vec<PathBuf> = vec![
        user_dir.join(format!("{}.toml", name)),
    ];
    let user_path = candidates.into_iter().find(|p| p.exists());
    match user_path {
        Some(p) => {
            std::fs::remove_file(&p)?;
            println!("removed user override: {}", p.display());
            Ok(())
        }
        None => {
            // Confirm the name resolves only to a built-in.
            if template::builtin_toml(name).is_some() {
                eprintln!("{}", ProductError::CannotResetBuiltin { name: name.to_string() });
                process::exit(1);
            }
            // Otherwise the name is unknown.
            eprintln!(
                "{}",
                ProductError::UnknownTarget {
                    name: name.to_string(),
                    available: template::builtin_names()
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect(),
                }
            );
            process::exit(1);
        }
    }
}
