//! Test criterion navigation, creation, status management.

use clap::Subcommand;
use product_lib::{error::ProductError, fileops, parser, types};

use super::{acquire_write_lock, load_graph, BoxResult};

#[derive(Subcommand)]
pub enum TestCommands {
    /// List all test criteria
    List {
        #[arg(long)]
        phase: Option<u32>,
        #[arg(long = "type")]
        test_type: Option<String>,
        #[arg(long)]
        status: Option<String>,
        /// Show only failing tests
        #[arg(long)]
        failing: bool,
    },
    /// Show a test criterion's details
    Show { id: String },
    /// List features with no linked test criteria
    Untested,
    /// Create a new test criterion file
    New {
        /// Test title
        title: String,
        /// Test type: scenario, invariant, chaos, exit-criteria
        #[arg(long = "type", default_value = "scenario")]
        test_type: String,
    },
    /// Set test criterion status
    Status {
        /// Test ID
        id: String,
        /// New status: unimplemented, implemented, passing, failing
        new_status: String,
    },
}

pub(crate) fn handle_test(cmd: TestCommands, fmt: &str) -> BoxResult {
    match cmd {
        TestCommands::List {
            phase,
            test_type,
            status,
            failing,
        } => test_list(phase, test_type, status, failing, fmt),
        TestCommands::Show { id } => test_show(&id, fmt),
        TestCommands::Untested => test_untested(),
        TestCommands::New { title, test_type } => test_new(&title, &test_type),
        TestCommands::Status { id, new_status } => test_status(&id, &new_status),
    }
}

fn test_list(
    phase: Option<u32>,
    test_type: Option<String>,
    status: Option<String>,
    failing: bool,
    fmt: &str,
) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let mut tests: Vec<&types::TestCriterion> = graph.tests.values().collect();
    tests.sort_by_key(|t| &t.front.id);

    if let Some(p) = phase {
        tests.retain(|t| t.front.phase == p);
    }
    if let Some(ref tt) = test_type {
        let target: types::TestType = tt.parse().map_err(|e: String| ProductError::ConfigError(e))?;
        tests.retain(|t| t.front.test_type == target);
    }
    if failing {
        tests.retain(|t| t.front.status == types::TestStatus::Failing);
    } else if let Some(ref s) = status {
        let target: types::TestStatus = s.parse().map_err(|e: String| ProductError::ConfigError(e))?;
        tests.retain(|t| t.front.status == target);
    }

    if fmt == "json" {
        let arr: Vec<serde_json::Value> = tests
            .iter()
            .map(|t| {
                serde_json::json!({
                    "id": t.front.id,
                    "phase": t.front.phase,
                    "type": t.front.test_type.to_string(),
                    "status": t.front.status.to_string(),
                    "title": t.front.title,
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&arr).unwrap_or_default());
    } else {
        println!(
            "{:<10} {:<8} {:<15} {:<15} TITLE",
            "ID", "PHASE", "TYPE", "STATUS"
        );
        println!("{}", "-".repeat(70));
        for t in &tests {
            println!(
                "{:<10} {:<8} {:<15} {:<15} {}",
                t.front.id, t.front.phase, t.front.test_type, t.front.status, t.front.title
            );
        }
    }
    Ok(())
}

fn test_show(id: &str, fmt: &str) -> BoxResult {
    let (_, _, graph) = load_graph()?;
    let t = graph
        .tests
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("test {}", id)))?;
    if fmt == "json" {
        print_test_json(t);
    } else {
        print_test_text(t);
    }
    Ok(())
}

fn print_test_json(t: &types::TestCriterion) {
    let obj = serde_json::json!({
        "id": t.front.id,
        "title": t.front.title,
        "type": t.front.test_type.to_string(),
        "status": t.front.status.to_string(),
        "phase": t.front.phase,
        "validates": {
            "features": t.front.validates.features,
            "adrs": t.front.validates.adrs,
        },
        "body": t.body,
    });
    println!("{}", serde_json::to_string_pretty(&obj).unwrap_or_default());
}

fn print_test_text(t: &types::TestCriterion) {
    println!("# {} — {}\n", t.front.id, t.front.title);
    println!("Type:     {}", t.front.test_type);
    println!("Status:   {}", t.front.status);
    println!("Phase:    {}", t.front.phase);
    println!(
        "Features: {}",
        if t.front.validates.features.is_empty() {
            "(none)".to_string()
        } else {
            t.front.validates.features.join(", ")
        }
    );
    println!(
        "ADRs:     {}",
        if t.front.validates.adrs.is_empty() {
            "(none)".to_string()
        } else {
            t.front.validates.adrs.join(", ")
        }
    );
    println!("\n{}", t.body);
}

fn test_untested() -> BoxResult {
    let (_, _, graph) = load_graph()?;
    println!("Features with no linked test criteria:");
    let mut found = false;
    for f in graph.features.values() {
        if f.front.status != types::FeatureStatus::Abandoned && f.front.tests.is_empty() {
            println!("  {} — {} (phase {})", f.front.id, f.front.title, f.front.phase);
            found = true;
        }
    }
    if !found {
        println!("  (none — all features have linked tests)");
    }
    Ok(())
}

fn test_new(title: &str, test_type: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (config, root, graph) = load_graph()?;
    let existing: Vec<String> = graph.tests.keys().cloned().collect();
    let id = parser::next_id(&config.prefixes.test, &existing);
    let filename = parser::id_to_filename(&id, title);
    let dir = config.resolve_path(&root, &config.paths.tests);
    std::fs::create_dir_all(&dir)?;
    let path = dir.join(&filename);

    let tt: types::TestType = test_type
        .parse()
        .map_err(|e: String| ProductError::ConfigError(e))?;

    let front = types::TestFrontMatter {
        id: id.clone(),
        title: title.to_string(),
        test_type: tt,
        status: types::TestStatus::Unimplemented,
        validates: types::ValidatesBlock {
            features: vec![],
            adrs: vec![],
        },
        phase: 1,
        content_hash: None,
        runner: None,
        runner_args: None,
        runner_timeout: None,
        requires: vec![],
        last_run: None,
        failure_message: None,
        last_run_duration: None,
    };

    let body = "## Description\n\n[Describe the test criterion here.]\n".to_string();
    let content = parser::render_test(&front, &body);
    fileops::write_file_atomic(&path, &content)?;
    println!("Created: {} at {}", id, path.display());
    Ok(())
}

fn test_status(id: &str, new_status: &str) -> BoxResult {
    let _lock = acquire_write_lock()?;
    let (_, _, graph) = load_graph()?;
    let t = graph
        .tests
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("test {}", id)))?;

    let status: types::TestStatus = new_status
        .parse()
        .map_err(|e: String| ProductError::ConfigError(e))?;

    let mut front = t.front.clone();
    front.status = status;
    let content = parser::render_test(&front, &t.body);
    fileops::write_file_atomic(&t.path, &content)?;
    println!("{} status -> {}", id, status);
    Ok(())
}

