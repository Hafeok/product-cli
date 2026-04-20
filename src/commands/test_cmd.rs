//! Test criterion navigation, creation, status management.

use clap::Subcommand;
use product_lib::{error::ProductError, tc, types};

use super::{acquire_write_lock_typed, load_graph_typed, BoxResult, CmdResult, Output};

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
    /// Configure test runner (runner, args, timeout, requires)
    Runner {
        /// Test ID
        id: String,
        /// Runner name: cargo-test, bash, pytest, custom
        #[arg(long)]
        runner: Option<String>,
        /// Runner arguments (e.g. test function name)
        #[arg(long)]
        args: Option<String>,
        /// Runner timeout (e.g. "60s")
        #[arg(long)]
        timeout: Option<String>,
        /// Add prerequisite (repeatable)
        #[arg(long)]
        requires: Vec<String>,
        /// Remove prerequisite (repeatable)
        #[arg(long)]
        remove_requires: Vec<String>,
    },
}

pub(crate) fn handle_test(cmd: TestCommands, fmt: &str) -> BoxResult {
    match cmd {
        TestCommands::List {
            phase,
            test_type,
            status,
            failing,
        } => super::render(test_list(phase, test_type, status, failing), fmt),
        TestCommands::Show { id } => super::render(test_show(&id), fmt),
        TestCommands::Untested => super::render(test_untested(), fmt),
        TestCommands::New { title, test_type } => super::render(test_new(&title, &test_type), fmt),
        TestCommands::Status { id, new_status } => super::render(test_status(&id, &new_status), fmt),
        TestCommands::Runner {
            id,
            runner,
            args,
            timeout,
            requires,
            remove_requires,
        } => super::render(test_runner(&id, runner, args, timeout, requires, remove_requires), fmt),
    }
}

fn test_list(
    phase: Option<u32>,
    test_type: Option<String>,
    status: Option<String>,
    failing: bool,
) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let mut tests: Vec<&types::TestCriterion> = graph.tests.values().collect();
    tests.sort_by_key(|t| &t.front.id);
    if let Some(p) = phase {
        tests.retain(|t| t.front.phase == p);
    }
    if let Some(ref tt) = test_type {
        let target: types::TestType = tt.parse().map_err(ProductError::ConfigError)?;
        tests.retain(|t| t.front.test_type == target);
    }
    if failing {
        tests.retain(|t| t.front.status == types::TestStatus::Failing);
    } else if let Some(ref s) = status {
        let target: types::TestStatus = s.parse().map_err(ProductError::ConfigError)?;
        tests.retain(|t| t.front.status == target);
    }
    let json = serde_json::Value::Array(
        tests
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
            .collect(),
    );
    let mut text = format!(
        "{:<10} {:<8} {:<15} {:<15} TITLE\n",
        "ID", "PHASE", "TYPE", "STATUS"
    );
    text.push_str(&"-".repeat(70));
    text.push('\n');
    for t in &tests {
        text.push_str(&format!(
            "{:<10} {:<8} {:<15} {:<15} {}\n",
            t.front.id, t.front.phase, t.front.test_type, t.front.status, t.front.title
        ));
    }
    Ok(Output::both(text, json))
}

fn test_show(id: &str) -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let t = graph
        .tests
        .get(id)
        .ok_or_else(|| ProductError::NotFound(format!("test {}", id)))?;
    let json = serde_json::json!({
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
    let text = render_test_show_text(t);
    Ok(Output::both(text, json))
}

fn render_test_show_text(t: &types::TestCriterion) -> String {
    let mut out = format!("# {} — {}\n\n", t.front.id, t.front.title);
    out.push_str(&format!("Type:     {}\n", t.front.test_type));
    out.push_str(&format!("Status:   {}\n", t.front.status));
    out.push_str(&format!("Phase:    {}\n", t.front.phase));
    out.push_str(&format!(
        "Features: {}\n",
        if t.front.validates.features.is_empty() {
            "(none)".to_string()
        } else {
            t.front.validates.features.join(", ")
        }
    ));
    out.push_str(&format!(
        "ADRs:     {}\n",
        if t.front.validates.adrs.is_empty() {
            "(none)".to_string()
        } else {
            t.front.validates.adrs.join(", ")
        }
    ));
    out.push_str(&format!("\n{}", t.body));
    out
}

fn test_untested() -> CmdResult {
    let (_, _, graph) = load_graph_typed()?;
    let list = product_lib::status::build_untested_list(&graph);
    let text = product_lib::status::render_feature_list_text(
        "Features with no linked test criteria:",
        &list,
    );
    let json = serde_json::to_value(&list.items).unwrap_or(serde_json::Value::Null);
    Ok(Output::both(text, json))
}

fn test_new(title: &str, test_type: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, root, graph) = load_graph_typed()?;
    let tt: types::TestType = test_type.parse().map_err(ProductError::ConfigError)?;
    let existing: Vec<String> = graph.tests.keys().cloned().collect();
    let plan = tc::plan_create(title, tt, &existing, &config.prefixes.test)?;
    let target_dir = config.resolve_path(&root, &config.paths.tests);
    let path = tc::apply_create(&plan, &target_dir)?;
    Ok(Output::text(format!("Created: {} at {}", plan.id, path.display())))
}

fn test_status(id: &str, new_status: &str) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (_, _, graph) = load_graph_typed()?;
    let status: types::TestStatus = new_status.parse().map_err(ProductError::ConfigError)?;
    let plan = tc::plan_status_change(&graph, id, status)?;
    tc::apply_status_change(&plan)?;
    Ok(Output::text(format!("{} status -> {}", id, plan.new_status)))
}

fn test_runner(
    id: &str,
    runner: Option<String>,
    args: Option<String>,
    timeout: Option<String>,
    requires: Vec<String>,
    remove_requires: Vec<String>,
) -> CmdResult {
    let _lock = acquire_write_lock_typed()?;
    let (config, _, graph) = load_graph_typed()?;
    let plan = tc::plan_runner_config(
        &config,
        &graph,
        id,
        runner.as_deref(),
        args.as_deref(),
        timeout.as_deref(),
        &requires,
        &remove_requires,
    )?;
    tc::apply_runner_config(&plan)?;
    Ok(Output::text(format!(
        "{} runner: {} args: {} timeout: {}",
        id,
        plan.final_runner.as_deref().unwrap_or("(none)"),
        plan.final_args.as_deref().unwrap_or("(none)"),
        plan.final_timeout
            .map_or("(none)".to_string(), |t| format!("{}s", t)),
    )))
}

