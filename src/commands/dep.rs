//! Dependency management commands (ADR-030) — thin read-only adapters.

use clap::Subcommand;
use product_lib::config::ProductConfig;
use product_lib::error::ProductError;
use product_lib::types::{Dependency, DependencyStatus, DependencyType};

use super::{load_graph_typed, BoxResult, CmdResult, Output};

#[derive(Subcommand)]
pub enum DepCommands {
    /// List all dependencies
    List {
        /// Filter by dependency type (library, service, api, tool, hardware, runtime)
        #[arg(long, rename_all = "kebab-case")]
        r#type: Option<String>,
        /// Filter by status (active, evaluating, deprecated, migrating)
        #[arg(long)]
        status: Option<String>,
    },
    /// Show full detail for a dependency
    Show {
        /// Dependency ID (e.g. DEP-001)
        id: String,
    },
    /// Show which features use a dependency
    Features {
        /// Dependency ID
        id: String,
    },
    /// Run availability check for a dependency
    Check {
        /// Dependency ID (omit with --all to check all)
        id: Option<String>,
        /// Check all dependencies
        #[arg(long)]
        all: bool,
    },
    /// Produce a dependency bill of materials
    Bom {
        /// Output format: text or json
        #[arg(long)]
        format: Option<String>,
    },
}

pub(crate) fn handle_dep(cmd: DepCommands, global_fmt: &str) -> BoxResult {
    match cmd {
        DepCommands::List { r#type, status } => {
            super::render(dep_list(r#type, status), global_fmt)
        }
        DepCommands::Show { id } => super::render(dep_show(&id), global_fmt),
        DepCommands::Features { id } => super::render(dep_features(&id), global_fmt),
        DepCommands::Check { id, all } => dep_check(id, all),
        DepCommands::Bom { format } => {
            let fmt = format.as_deref().unwrap_or(global_fmt);
            super::render(dep_bom(), fmt)
        }
    }
}

fn dep_list(type_filter: Option<String>, status_filter: Option<String>) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let type_enum: Option<DependencyType> = type_filter
        .as_deref()
        .map(|s| s.parse::<DependencyType>())
        .transpose()
        .map_err(ProductError::ConfigError)?;
    let status_enum: Option<DependencyStatus> = status_filter
        .as_deref()
        .map(|s| s.parse::<DependencyStatus>())
        .transpose()
        .map_err(ProductError::ConfigError)?;

    let mut deps: Vec<&Dependency> = graph
        .dependencies
        .values()
        .filter(|d| {
            type_enum.is_none_or(|t| d.front.dep_type == t)
                && status_enum.is_none_or(|s| d.front.status == s)
        })
        .collect();
    deps.sort_by(|a, b| a.front.id.cmp(&b.front.id));

    let json = serde_json::Value::Array(deps.iter().map(|d| dep_to_json(d)).collect());
    let text = render_dep_list_text(&deps);
    Ok(Output::both(text, json))
}

fn dep_show(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let dep = graph
        .dependencies
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("dependency {}", id)))?;
    let json = serde_json::json!({
        "id": dep.front.id, "title": dep.front.title,
        "type": dep.front.dep_type.to_string(), "source": dep.front.source,
        "version": dep.front.version, "status": dep.front.status.to_string(),
        "features": dep.front.features, "adrs": dep.front.adrs,
        "availability-check": dep.front.availability_check,
        "breaking-change-risk": dep.front.breaking_change_risk,
        "interface": dep.front.interface.as_ref().map(|i| serde_json::to_value(i).unwrap_or_default()),
    });
    let text = render_dep_show_text(dep);
    Ok(Output::both(text, json))
}

fn dep_features(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let dep = graph
        .dependencies
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("dependency {}", id)))?;
    let mut text = format!("Features using {}:\n", dep.front.id);
    for fid in &dep.front.features {
        let title = graph
            .features
            .get(fid)
            .map(|f| f.front.title.as_str())
            .unwrap_or("(unknown)");
        text.push_str(&format!("  {} \u{2014} {}\n", fid, title));
    }
    let json = serde_json::to_value(&dep.front.features).unwrap_or(serde_json::Value::Null);
    Ok(Output::both(text, json))
}

/// Left as `BoxResult` because availability-check failure has exit-code-2
/// semantics that the `CmdResult`/`ProductError` surface cannot express.
/// Prints progress to stdout directly.
fn dep_check(id: Option<String>, all: bool) -> BoxResult {
    let (_, _, graph) = load_graph_typed()?;
    let deps_to_check: Vec<&Dependency> = if all {
        let mut d: Vec<_> = graph.dependencies.values().collect();
        d.sort_by(|a, b| a.front.id.cmp(&b.front.id));
        d
    } else if let Some(ref dep_id) = id {
        vec![graph
            .dependencies
            .get(dep_id)
            .ok_or_else(|| ProductError::NotFound(format!("dependency {}", dep_id)))?]
    } else {
        return Err(Box::new(ProductError::ConfigError(
            "provide a dependency ID or use --all".to_string(),
        )));
    };

    let mut any_failed = false;
    for dep in &deps_to_check {
        let (line, failed) = run_single_check(dep);
        println!("{}", line);
        any_failed |= failed;
    }
    if any_failed {
        std::process::exit(2);
    }
    Ok(())
}

fn dep_bom() -> CmdResult {
    let (config, _, graph) = load_graph_typed()?;
    let mut deps: Vec<&Dependency> = graph.dependencies.values().collect();
    deps.sort_by(|a, b| a.front.id.cmp(&b.front.id));
    let arr: Vec<serde_json::Value> = deps.iter().map(|d| dep_to_json(d)).collect();
    let json = serde_json::json!({
        "product": config.name,
        "dependencies": arr,
        "total": deps.len(),
    });
    let text = render_dep_bom_text(&config, &deps);
    Ok(Output::both(text, json))
}

// ---- pure renderers ------------------------------------------------------

fn dep_to_json(d: &Dependency) -> serde_json::Value {
    serde_json::json!({
        "id": d.front.id,
        "title": d.front.title,
        "type": d.front.dep_type.to_string(),
        "version": d.front.version,
        "status": d.front.status.to_string(),
        "features": d.front.features,
        "breaking-change-risk": d.front.breaking_change_risk,
    })
}

fn render_dep_list_text(deps: &[&Dependency]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<10} {:<30} {:<10} {:<15} STATUS\n",
        "ID", "TITLE", "TYPE", "VERSION"
    ));
    out.push_str(&"-".repeat(75));
    out.push('\n');
    for d in deps {
        let version = d.front.version.as_deref().unwrap_or("~");
        out.push_str(&format!(
            "{:<10} {:<30} {:<10} {:<15} {}\n",
            d.front.id, d.front.title, d.front.dep_type, version, d.front.status
        ));
    }
    out
}

fn render_dep_show_text(dep: &Dependency) -> String {
    let mut out = format!("{} \u{2014} {}\n", dep.front.id, dep.front.title);
    out.push_str(&format!("  Type:    {}\n", dep.front.dep_type));
    out.push_str(&format!(
        "  Version: {}\n",
        dep.front.version.as_deref().unwrap_or("~")
    ));
    out.push_str(&format!("  Status:  {}\n", dep.front.status));
    out.push_str(&format!("  Risk:    {}\n", dep.front.breaking_change_risk));
    if !dep.front.features.is_empty() {
        out.push_str(&format!("  Features: {}\n", dep.front.features.join(", ")));
    }
    if !dep.front.adrs.is_empty() {
        out.push_str(&format!("  ADRs:    {}\n", dep.front.adrs.join(", ")));
    }
    if let Some(ref check) = dep.front.availability_check {
        out.push_str(&format!("  Check:   {}\n", check));
    }
    out
}

fn render_dep_bom_text(config: &ProductConfig, deps: &[&Dependency]) -> String {
    let mut out = format!(
        "Dependency Bill of Materials \u{2014} {} v{}\n\n",
        config.name, config.version
    );
    for (dep_type, label) in &BOM_TYPES {
        append_bom_section(&mut out, deps, *dep_type, label);
    }
    append_bom_summary(&mut out, deps);
    out
}

const BOM_TYPES: [(DependencyType, &str); 6] = [
    (DependencyType::Library, "Libraries (build-time)"),
    (DependencyType::Service, "Services (runtime)"),
    (DependencyType::Api, "APIs (external)"),
    (DependencyType::Tool, "Tools (CLI)"),
    (DependencyType::Runtime, "Runtimes"),
    (DependencyType::Hardware, "Hardware"),
];

fn append_bom_section(out: &mut String, deps: &[&Dependency], dep_type: DependencyType, label: &str) {
    let typed: Vec<&&Dependency> = deps.iter().filter(|d| d.front.dep_type == dep_type).collect();
    if typed.is_empty() {
        return;
    }
    out.push_str(&format!("{}:\n", label));
    for d in &typed {
        let v = d.front.version.as_deref().unwrap_or("~");
        let s = d.front.source.as_deref().unwrap_or("\u{2014}");
        out.push_str(&format!(
            "  {:<10} {:<25} {:<15} {:<15} {}\n",
            d.front.id, d.front.title, v, s, d.front.status
        ));
    }
    out.push('\n');
}

fn append_bom_summary(out: &mut String, deps: &[&Dependency]) {
    let type_count: std::collections::HashSet<_> = deps.iter().map(|d| d.front.dep_type).collect();
    out.push_str(&format!(
        "Total: {} dependencies across {} types\n",
        deps.len(),
        type_count.len()
    ));
    let risk: Vec<_> = ["high", "medium", "low"]
        .iter()
        .filter_map(|r| {
            let n = deps.iter().filter(|d| d.front.breaking_change_risk == *r).count();
            if n > 0 {
                Some(format!("{} {}", n, r))
            } else {
                None
            }
        })
        .collect();
    if !risk.is_empty() {
        out.push_str(&format!("Breaking change risk: {}\n", risk.join(", ")));
    }
}

fn run_single_check(dep: &Dependency) -> (String, bool) {
    match &dep.front.availability_check {
        None => (
            format!(
                "  {}  {} [no check]  \u{2713}",
                dep.front.id, dep.front.title
            ),
            false,
        ),
        Some(check_cmd) => {
            let ok = std::process::Command::new("sh")
                .args(["-c", check_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if ok {
                (
                    format!(
                        "  {}  {} [check passed]  \u{2713}",
                        dep.front.id, dep.front.title
                    ),
                    false,
                )
            } else {
                (
                    format!(
                        "  {}  {} [check FAILED]  \u{2717}",
                        dep.front.id, dep.front.title
                    ),
                    true,
                )
            }
        }
    }
}
