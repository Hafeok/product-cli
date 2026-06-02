//! Template resolution — repo → user → built-in. First match wins (ADR-049).

use super::loader::{parse_template, Template};
use super::validate::validate_template;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where a resolved template was read from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateSource {
    /// `.product/templates/<name>.toml` inside the repo.
    Repo(PathBuf),
    /// `~/.product/templates/<name>.toml`.
    User(PathBuf),
    /// Embedded built-in (no on-disk path).
    Builtin,
}

impl TemplateSource {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Repo(_) => "repo",
            Self::User(_) => "user",
            Self::Builtin => "built-in",
        }
    }

    pub fn path(&self) -> Option<&Path> {
        match self {
            Self::Repo(p) | Self::User(p) => Some(p.as_path()),
            Self::Builtin => None,
        }
    }
}

/// A template that has been parsed, validated, and tagged with its origin.
#[derive(Debug, Clone)]
pub struct ResolvedTemplate {
    pub name: String,
    pub source: TemplateSource,
    pub raw_toml: String,
    pub template: Template,
}

// ---------------------------------------------------------------------------
// Built-in templates — embedded via include_str! so the binary is
// self-contained. Files live in src/context/template/builtin/*.toml.
// ---------------------------------------------------------------------------

const BUILTIN_TEMPLATES: &[(&str, &str)] = &[
    ("claude-opus",     include_str!("builtin/claude-opus.toml")),
    ("claude-haiku",    include_str!("builtin/claude-haiku.toml")),
    ("gpt-4-markdown",  include_str!("builtin/gpt-4-markdown.toml")),
    ("gpt-mini-json",   include_str!("builtin/gpt-mini-json.toml")),
    ("gemini-yaml",     include_str!("builtin/gemini-yaml.toml")),
    ("human",           include_str!("builtin/human.toml")),
];

/// Names of the six built-in templates (in stable iteration order).
pub fn builtin_names() -> Vec<&'static str> {
    BUILTIN_TEMPLATES.iter().map(|(n, _)| *n).collect()
}

/// Returns the embedded TOML for a built-in name, or `None` if not built-in.
pub fn builtin_toml(name: &str) -> Option<&'static str> {
    BUILTIN_TEMPLATES
        .iter()
        .find(|(n, _)| *n == name)
        .map(|(_, t)| *t)
}

/// Returned by `resolve_all` — splits successes from validation failures so
/// the CLI can list valid templates while warning about invalid ones.
pub struct ResolveOutcome {
    pub resolved: HashMap<String, ResolvedTemplate>,
    /// `(name, path-or-none, reason)` — invalid templates are excluded from
    /// `resolved` (TC-746) but reported via this list.
    pub warnings: Vec<(String, Option<PathBuf>, String)>,
}

/// Build the merged template map. `repo_root` is the canonical project root
/// (the directory containing `.product/`). Resolution order is:
/// `<repo_root>/.product/templates` → `~/.product/templates` → built-ins.
pub fn resolve_all(repo_root: &Path) -> ResolveOutcome {
    let mut resolved: HashMap<String, ResolvedTemplate> = HashMap::new();
    let mut warnings: Vec<(String, Option<PathBuf>, String)> = Vec::new();

    // Pass 1: repo-local templates.
    let repo_dir = repo_root.join(".product").join("templates");
    load_dir_into(&repo_dir, &mut resolved, &mut warnings, |p| {
        TemplateSource::Repo(p)
    });

    // Pass 2: user templates (only where the name is not already taken).
    if let Some(user_dir) = user_templates_dir() {
        load_dir_into(&user_dir, &mut resolved, &mut warnings, |p| {
            TemplateSource::User(p)
        });
    }

    // Pass 3: built-ins (fill any remaining names).
    for (name, toml_text) in BUILTIN_TEMPLATES {
        if resolved.contains_key(*name) {
            continue;
        }
        match parse_template(toml_text) {
            Ok(parsed) => {
                if let Err(e) = validate_template(&parsed) {
                    warnings.push(((*name).to_string(), None, e));
                    continue;
                }
                resolved.insert(
                    (*name).to_string(),
                    ResolvedTemplate {
                        name: (*name).to_string(),
                        source: TemplateSource::Builtin,
                        raw_toml: (*toml_text).to_string(),
                        template: parsed,
                    },
                );
            }
            Err(e) => {
                warnings.push(((*name).to_string(), None, e.to_string()));
            }
        }
    }

    ResolveOutcome { resolved, warnings }
}

/// Resolve a single named template. Convenience wrapper used by the CLI
/// flag handler and the MCP read tool. Returns `None` when the name does
/// not match any resolved template.
pub fn resolve_one(repo_root: &Path, name: &str) -> Option<ResolvedTemplate> {
    let outcome = resolve_all(repo_root);
    outcome.resolved.get(name).cloned()
}

/// User templates directory — `$HOME/.product/templates`. Returns `None` if
/// the home directory cannot be determined.
pub fn user_templates_dir() -> Option<PathBuf> {
    home_dir().map(|h| h.join(".product").join("templates"))
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn load_dir_into<F>(
    dir: &Path,
    resolved: &mut HashMap<String, ResolvedTemplate>,
    warnings: &mut Vec<(String, Option<PathBuf>, String)>,
    src_for: F,
)
where
    F: Fn(PathBuf) -> TemplateSource,
{
    let entries = match std::fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return, // Missing directory is fine — skip.
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|s| s != "toml").unwrap_or(true) {
            continue;
        }
        let stem = match path.file_stem().and_then(|s| s.to_str()) {
            Some(s) => s.to_string(),
            None => continue,
        };
        if resolved.contains_key(&stem) {
            continue; // First-match-wins.
        }
        let raw = match std::fs::read_to_string(&path) {
            Ok(s) => s,
            Err(e) => {
                warnings.push((stem, Some(path.clone()), format!("read error: {}", e)));
                continue;
            }
        };
        match parse_template(&raw) {
            Ok(parsed) => {
                if parsed.template.name != stem {
                    // Allow either matching filename or matching template.name;
                    // we store under the parsed template.name so users that
                    // rename a file keep the canonical key.
                }
                if let Err(e) = validate_template(&parsed) {
                    warnings.push((parsed.template.name.clone(), Some(path.clone()), e));
                    continue;
                }
                let key = parsed.template.name.clone();
                if resolved.contains_key(&key) {
                    continue;
                }
                resolved.insert(
                    key.clone(),
                    ResolvedTemplate {
                        name: key,
                        source: src_for(path.clone()),
                        raw_toml: raw,
                        template: parsed,
                    },
                );
            }
            Err(e) => {
                warnings.push((stem, Some(path.clone()), e.to_string()));
            }
        }
    }
}
