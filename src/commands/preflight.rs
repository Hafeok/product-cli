//! Pre-flight analysis: domain coverage, cross-cutting checks, dependency availability (ADR-030).

use product_lib::domains;
use product_lib::types::DependencyStatus;
use std::process;

use super::{load_graph, BoxResult};

pub(crate) fn handle_preflight(id: &str) -> BoxResult {
    let (config, _root, graph) = load_graph()?;
    let result = domains::preflight(&graph, id, &config.domains)?;
    print!("{}", domains::render_preflight(&result));

    // Dependency availability checks (ADR-030)
    let mut dep_warnings = false;
    let feature_deps: Vec<_> = graph.dependencies.values()
        .filter(|d| d.front.features.contains(&id.to_string()))
        .collect();
    if !feature_deps.is_empty() {
        println!();
        println!("\u{2501}\u{2501}\u{2501} Dependency Availability \u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}\u{2501}");
        println!();
        for dep in &feature_deps {
            match &dep.front.availability_check {
                None => {
                    println!("  {}  {:<25} [{} \u{2014} no check]    \u{2713}", dep.front.id, dep.front.title, dep.front.dep_type);
                }
                Some(check_cmd) => {
                    let check_result = std::process::Command::new("sh")
                        .args(["-c", check_cmd])
                        .stdout(std::process::Stdio::null())
                        .stderr(std::process::Stdio::null())
                        .status();
                    match check_result {
                        Ok(status) if status.success() => {
                            println!("  {}  {:<25} [{}]         \u{2713}", dep.front.id, dep.front.title, dep.front.dep_type);
                        }
                        _ => {
                            println!("  {}  {:<25} [{}]         \u{2717} not running", dep.front.id, dep.front.title, dep.front.dep_type);
                            dep_warnings = true;
                        }
                    }
                }
            }
            // Also warn if deprecated
            if dep.front.status == DependencyStatus::Deprecated || dep.front.status == DependencyStatus::Migrating {
                println!("    \u{26A0}  status: {} \u{2014} consider migration", dep.front.status);
                dep_warnings = true;
            }
        }
        println!();
    }

    if !result.is_clean {
        process::exit(1);
    }
    if dep_warnings {
        process::exit(2);
    }
    Ok(())
}
