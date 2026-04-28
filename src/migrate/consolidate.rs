//! Plan phase for `product migrate consolidate` (FT-057, ADR-048).
//!
//! Pure functions only — `plan_consolidate` inspects the filesystem and the
//! loaded config and returns a `ConsolidationPlan`. The apply phase lives in
//! `consolidate_apply` so the planner stays I/O-free at the slice boundary.

use crate::config::ProductConfig;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedMove {
    pub from: String,
    pub to: String,
    /// `true` if the source is a directory (recursive move).
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathRewrite {
    pub key: String,
    pub from: Option<String>,
    pub to: String,
}

#[derive(Debug, Clone, Default)]
pub struct ConsolidationPlan {
    pub moves: Vec<PlannedMove>,
    pub path_rewrites: Vec<PathRewrite>,
    pub gitignore_lines: Vec<String>,
    pub current_config_path: String,
    pub canonical_config_path: String,
}

impl ConsolidationPlan {
    pub fn is_noop(&self) -> bool {
        self.moves.is_empty()
            && self.path_rewrites.is_empty()
            && self.gitignore_lines.is_empty()
            && self.current_config_path == self.canonical_config_path
    }

    /// Render the plan as a human-readable text block for `--dry-run`.
    pub fn render(&self) -> String {
        let mut out = String::new();
        if self.is_noop() {
            out.push_str("Already canonical — nothing to do.\n");
            return out;
        }
        out.push_str("Planned consolidation (FT-057):\n\n");
        if self.current_config_path != self.canonical_config_path {
            out.push_str(&format!(
                "  config: {} → {}\n",
                self.current_config_path, self.canonical_config_path
            ));
        }
        if !self.moves.is_empty() {
            out.push_str("\n  moves:\n");
            for m in &self.moves {
                let kind = if m.is_dir { "dir " } else { "file" };
                out.push_str(&format!("    {} {} → {}\n", kind, m.from, m.to));
            }
        }
        if !self.path_rewrites.is_empty() {
            out.push_str("\n  [paths] rewrites:\n");
            for r in &self.path_rewrites {
                let from = r.from.as_deref().unwrap_or("<unset>");
                out.push_str(&format!("    {} = \"{}\"  (was \"{}\")\n", r.key, r.to, from));
            }
        }
        if !self.gitignore_lines.is_empty() {
            out.push_str("\n  .gitignore additions:\n");
            for l in &self.gitignore_lines {
                out.push_str(&format!("    {}\n", l));
            }
        }
        out
    }
}

pub const CANONICAL_FEATURES: &str = ".product/features";
pub const CANONICAL_ADRS: &str = ".product/adrs";
pub const CANONICAL_TESTS: &str = ".product/tests";
pub const CANONICAL_DEPS: &str = ".product/dependencies";
pub const CANONICAL_GRAPH: &str = ".product/graph";
pub const CANONICAL_CHECKLIST: &str = ".product/checklist.md";
pub const CANONICAL_REQUESTS: &str = ".product/requests.jsonl";
pub const CANONICAL_PROMPTS: &str = ".product/prompts";
pub const CANONICAL_GAPS: &str = ".product/gaps.json";
pub const CANONICAL_CONFIG: &str = ".product/config.toml";

pub(crate) fn canonical_paths_block_static() -> &'static str {
    "[paths]\n\
     features = \".product/features\"\n\
     adrs = \".product/adrs\"\n\
     tests = \".product/tests\"\n\
     dependencies = \".product/dependencies\"\n\
     graph = \".product/graph\"\n\
     checklist = \".product/checklist.md\"\n\
     requests = \".product/requests.jsonl\"\n\
     prompts = \".product/prompts\"\n\
     gaps = \".product/gaps.json\"\n\n"
}

fn current_config_path_for(root: &Path) -> String {
    let candidates = [".product/config.toml", ".product/product.toml", "product.toml"];
    for c in candidates {
        if root.join(c).exists() {
            return c.to_string();
        }
    }
    "product.toml".to_string()
}

/// Build a consolidation plan for `root` given the current `config`.
/// Idempotent: when every artifact is already at its canonical path and the
/// config is already at `.product/config.toml`, the plan is a no-op.
pub fn plan_consolidate(root: &Path, config: &ProductConfig) -> ConsolidationPlan {
    let mut plan = ConsolidationPlan {
        current_config_path: current_config_path_for(root),
        canonical_config_path: CANONICAL_CONFIG.to_string(),
        ..Default::default()
    };
    plan_directory_moves(&mut plan, root, config);
    plan_file_moves(&mut plan, root, config);
    plan_path_rewrites(&mut plan, config);
    plan_gitignore_additions(&mut plan, root);
    plan
}

fn plan_directory_moves(plan: &mut ConsolidationPlan, root: &Path, config: &ProductConfig) {
    let pairs: [(&str, &str); 6] = [
        (config.paths.features.as_str(), CANONICAL_FEATURES),
        (config.paths.adrs.as_str(), CANONICAL_ADRS),
        (config.paths.tests.as_str(), CANONICAL_TESTS),
        (config.paths.dependencies.as_str(), CANONICAL_DEPS),
        (config.paths.graph.as_str(), CANONICAL_GRAPH),
        (config.paths.prompts_resolved(), CANONICAL_PROMPTS),
    ];
    for (from, to) in pairs {
        plan_move_if_needed(plan, root, from, to, true);
    }
}

fn plan_file_moves(plan: &mut ConsolidationPlan, root: &Path, config: &ProductConfig) {
    let current_config = plan.current_config_path.clone();
    let pairs: [(&str, &str); 4] = [
        (config.paths.checklist.as_str(), CANONICAL_CHECKLIST),
        (config.paths.requests.as_str(), CANONICAL_REQUESTS),
        (config.paths.gaps_resolved(), CANONICAL_GAPS),
        (current_config.as_str(), CANONICAL_CONFIG),
    ];
    for (from, to) in pairs {
        plan_move_if_needed(plan, root, from, to, false);
    }
}

fn plan_move_if_needed(
    plan: &mut ConsolidationPlan,
    root: &Path,
    from: &str,
    to: &str,
    is_dir: bool,
) {
    if from == to {
        return;
    }
    let abs = root.join(from);
    if !abs.exists() {
        return;
    }
    plan.moves.push(PlannedMove {
        from: from.to_string(),
        to: to.to_string(),
        is_dir,
    });
}

fn plan_path_rewrites(plan: &mut ConsolidationPlan, config: &ProductConfig) {
    let pairs: [(&str, &str, &str); 9] = [
        ("features", config.paths.features.as_str(), CANONICAL_FEATURES),
        ("adrs", config.paths.adrs.as_str(), CANONICAL_ADRS),
        ("tests", config.paths.tests.as_str(), CANONICAL_TESTS),
        ("dependencies", config.paths.dependencies.as_str(), CANONICAL_DEPS),
        ("graph", config.paths.graph.as_str(), CANONICAL_GRAPH),
        ("checklist", config.paths.checklist.as_str(), CANONICAL_CHECKLIST),
        ("requests", config.paths.requests.as_str(), CANONICAL_REQUESTS),
        ("prompts", config.paths.prompts_resolved(), CANONICAL_PROMPTS),
        ("gaps", config.paths.gaps_resolved(), CANONICAL_GAPS),
    ];
    for (key, current, canonical) in pairs {
        if current != canonical {
            plan.path_rewrites.push(PathRewrite {
                key: key.to_string(),
                from: Some(current.to_string()),
                to: canonical.to_string(),
            });
        }
    }
}

fn plan_gitignore_additions(plan: &mut ConsolidationPlan, root: &Path) {
    let gitignore = root.join(".gitignore");
    let existing = std::fs::read_to_string(&gitignore).unwrap_or_default();
    for line in [".product/graph/", ".product/sessions/"] {
        if !existing.lines().any(|l| l.trim() == line) {
            plan.gitignore_lines.push(line.to_string());
        }
    }
}
