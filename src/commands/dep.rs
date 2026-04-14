//! Dependency management commands (ADR-030)

use clap::Subcommand;
use product_lib::error::ProductError;
use product_lib::types::{DependencyStatus, DependencyType};

use super::{load_graph, BoxResult};

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
        DepCommands::List { r#type, status } => dep_list(r#type, status, global_fmt),
        DepCommands::Show { id } => dep_show(&id, global_fmt),
        DepCommands::Features { id } => dep_features(&id, global_fmt),
        DepCommands::Check { id, all } => dep_check(id, all),
        DepCommands::Bom { format } => dep_bom(format, global_fmt),
    }
}

fn dep_list(type_filter: Option<String>, status_filter: Option<String>, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let type_enum: Option<DependencyType> = type_filter.as_deref()
        .map(|s| s.parse::<DependencyType>()).transpose().map_err(ProductError::ConfigError)?;
    let status_enum: Option<DependencyStatus> = status_filter.as_deref()
        .map(|s| s.parse::<DependencyStatus>()).transpose().map_err(ProductError::ConfigError)?;

    let mut deps: Vec<_> = graph.dependencies.values()
        .filter(|d| {
            type_enum.is_none_or(|t| d.front.dep_type == t)
                && status_enum.is_none_or(|s| d.front.status == s)
        })
        .collect();
    deps.sort_by(|a, b| a.front.id.cmp(&b.front.id));

    if fmt == "json" {
        let arr: Vec<serde_json::Value> = deps.iter().map(|d| dep_to_json(d)).collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        dep_list_text(&deps);
    }
    Ok(())
}

fn dep_to_json(d: &product_lib::types::Dependency) -> serde_json::Value {
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

fn dep_list_text(deps: &[&product_lib::types::Dependency]) {
    println!("{:<10} {:<30} {:<10} {:<15} STATUS", "ID", "TITLE", "TYPE", "VERSION");
    println!("{}", "-".repeat(75));
    for d in deps {
        let version = d.front.version.as_deref().unwrap_or("~");
        println!("{:<10} {:<30} {:<10} {:<15} {}", d.front.id, d.front.title, d.front.dep_type, version, d.front.status);
    }
}

fn dep_show(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let dep = graph.dependencies.get(id)
        .ok_or_else(|| ProductError::NotFound(format!("dependency {}", id)))?;
    if fmt == "json" {
        let obj = serde_json::json!({
            "id": dep.front.id, "title": dep.front.title,
            "type": dep.front.dep_type.to_string(), "source": dep.front.source,
            "version": dep.front.version, "status": dep.front.status.to_string(),
            "features": dep.front.features, "adrs": dep.front.adrs,
            "availability-check": dep.front.availability_check,
            "breaking-change-risk": dep.front.breaking_change_risk,
            "interface": dep.front.interface.as_ref().map(|i| serde_json::to_value(i).unwrap_or_default()),
        });
        println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
    } else {
        dep_show_text(dep);
    }
    Ok(())
}

fn dep_show_text(dep: &product_lib::types::Dependency) {
    println!("{} \u{2014} {}", dep.front.id, dep.front.title);
    println!("  Type:    {}", dep.front.dep_type);
    println!("  Version: {}", dep.front.version.as_deref().unwrap_or("~"));
    println!("  Status:  {}", dep.front.status);
    println!("  Risk:    {}", dep.front.breaking_change_risk);
    if !dep.front.features.is_empty() {
        println!("  Features: {}", dep.front.features.join(", "));
    }
    if !dep.front.adrs.is_empty() {
        println!("  ADRs:    {}", dep.front.adrs.join(", "));
    }
    if let Some(ref check) = dep.front.availability_check {
        println!("  Check:   {}", check);
    }
}

fn dep_features(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let dep = graph.dependencies.get(id)
        .ok_or_else(|| ProductError::NotFound(format!("dependency {}", id)))?;
    if fmt == "json" {
        println!("{}", serde_json::to_string_pretty(&dep.front.features).unwrap_or_default());
    } else {
        println!("Features using {}:", dep.front.id);
        for fid in &dep.front.features {
            let title = graph.features.get(fid).map(|f| f.front.title.as_str()).unwrap_or("(unknown)");
            println!("  {} \u{2014} {}", fid, title);
        }
    }
    Ok(())
}

fn dep_check(id: Option<String>, all: bool) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let deps_to_check: Vec<&product_lib::types::Dependency> = if all {
        let mut d: Vec<_> = graph.dependencies.values().collect();
        d.sort_by(|a, b| a.front.id.cmp(&b.front.id));
        d
    } else if let Some(ref dep_id) = id {
        vec![graph.dependencies.get(dep_id)
            .ok_or_else(|| ProductError::NotFound(format!("dependency {}", dep_id)))?]
    } else {
        return Err(Box::new(ProductError::ConfigError("provide a dependency ID or use --all".into())));
    };
    let mut any_failed = false;
    for dep in &deps_to_check {
        any_failed |= run_single_check(dep);
    }
    if any_failed { std::process::exit(2); }
    Ok(())
}

fn run_single_check(dep: &product_lib::types::Dependency) -> bool {
    match &dep.front.availability_check {
        None => {
            println!("  {}  {} [no check]  \u{2713}", dep.front.id, dep.front.title);
            false
        }
        Some(check_cmd) => {
            let ok = std::process::Command::new("sh").args(["-c", check_cmd])
                .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null())
                .status().map(|s| s.success()).unwrap_or(false);
            if ok {
                println!("  {}  {} [check passed]  \u{2713}", dep.front.id, dep.front.title);
            } else {
                println!("  {}  {} [check FAILED]  \u{2717}", dep.front.id, dep.front.title);
            }
            !ok
        }
    }
}

fn dep_bom(format: Option<String>, global_fmt: &str) -> BoxResult {
    let (config, _, graph) = load_graph()?;
    let fmt = format.as_deref().unwrap_or(global_fmt);
    let mut deps: Vec<_> = graph.dependencies.values().collect();
    deps.sort_by(|a, b| a.front.id.cmp(&b.front.id));

    if fmt == "json" {
        dep_bom_json(&config, &deps);
    } else {
        dep_bom_text(&config, &deps);
    }
    Ok(())
}

fn dep_bom_json(config: &product_lib::config::ProductConfig, deps: &[&product_lib::types::Dependency]) {
    let arr: Vec<serde_json::Value> = deps.iter().map(|d| dep_to_json(d)).collect();
    let bom = serde_json::json!({ "product": config.name, "dependencies": arr, "total": deps.len() });
    println!("{}", serde_json::to_string_pretty(&bom).unwrap_or_default());
}

fn dep_bom_text(config: &product_lib::config::ProductConfig, deps: &[&product_lib::types::Dependency]) {
    println!("Dependency Bill of Materials \u{2014} {} v{}", config.name, config.version);
    println!();
    let types_and_labels = [
        (DependencyType::Library, "Libraries (build-time)"),
        (DependencyType::Service, "Services (runtime)"),
        (DependencyType::Api, "APIs (external)"),
        (DependencyType::Tool, "Tools (CLI)"),
        (DependencyType::Runtime, "Runtimes"),
        (DependencyType::Hardware, "Hardware"),
    ];
    for (dep_type, label) in &types_and_labels {
        let typed: Vec<_> = deps.iter().filter(|d| d.front.dep_type == *dep_type).collect();
        if typed.is_empty() { continue; }
        println!("{}:", label);
        for d in &typed {
            let v = d.front.version.as_deref().unwrap_or("~");
            let s = d.front.source.as_deref().unwrap_or("\u{2014}");
            println!("  {:<10} {:<25} {:<15} {:<15} {}", d.front.id, d.front.title, v, s, d.front.status);
        }
        println!();
    }
    let type_count: std::collections::HashSet<_> = deps.iter().map(|d| d.front.dep_type).collect();
    println!("Total: {} dependencies across {} types", deps.len(), type_count.len());
    let risk: Vec<_> = ["high", "medium", "low"].iter().filter_map(|r| {
        let n = deps.iter().filter(|d| d.front.breaking_change_risk == *r).count();
        if n > 0 { Some(format!("{} {}", n, r)) } else { None }
    }).collect();
    if !risk.is_empty() { println!("Breaking change risk: {}", risk.join(", ")); }
}
