//! Test-criterion runner configuration — runner, args, timeout, requires.

use crate::config::ProductConfig;
use crate::error::ProductError;
use crate::graph::KnowledgeGraph;
use crate::{fileops, parser};
use std::path::PathBuf;

const VALID_RUNNERS: &[&str] = &["cargo-test", "bash", "pytest", "custom"];

#[derive(Debug, Clone)]
pub struct RunnerConfigPlan {
    pub test_id: String,
    pub test_path: PathBuf,
    pub test_content: String,
    pub final_runner: Option<String>,
    pub final_args: Option<String>,
    pub final_timeout: Option<u64>,
}

#[allow(clippy::too_many_arguments)]
pub fn plan_runner_config(
    config: &ProductConfig,
    graph: &KnowledgeGraph,
    test_id: &str,
    runner: Option<&str>,
    args: Option<&str>,
    timeout: Option<&str>,
    requires: &[String],
    remove_requires: &[String],
) -> Result<RunnerConfigPlan, ProductError> {
    let t = graph
        .tests
        .get(test_id)
        .ok_or_else(|| ProductError::NotFound(format!("test {}", test_id)))?;

    let mut front = t.front.clone();

    if let Some(r) = runner {
        if !VALID_RUNNERS.contains(&r) {
            return Err(ProductError::ConfigError(format!(
                "error[E001]: unknown runner '{}'. Valid values: {}",
                r,
                VALID_RUNNERS.join(", ")
            )));
        }
        front.runner = Some(r.to_string());
    }

    if let Some(a) = args {
        front.runner_args = Some(a.to_string());
    }

    if let Some(t_str) = timeout {
        let secs = t_str
            .trim_end_matches('s')
            .parse::<u64>()
            .map_err(|_| ProductError::ConfigError(format!("invalid timeout: {}", t_str)))?;
        front.runner_timeout = Some(secs);
    }

    for req in requires {
        if !config.verify.prerequisites.contains_key(req) {
            return Err(ProductError::ConfigError(format!(
                "error[E001]: unknown prerequisite '{}'. Check [verify.prerequisites] in product.toml",
                req
            )));
        }
        if !front.requires.contains(req) {
            front.requires.push(req.clone());
        }
    }
    for req in remove_requires {
        front.requires.retain(|r| r != req);
    }

    let content = parser::render_test(&front, &t.body);
    Ok(RunnerConfigPlan {
        test_id: test_id.to_string(),
        test_path: t.path.clone(),
        test_content: content,
        final_runner: front.runner,
        final_args: front.runner_args,
        final_timeout: front.runner_timeout,
    })
}

pub fn apply_runner_config(plan: &RunnerConfigPlan) -> Result<(), ProductError> {
    fileops::write_file_atomic(&plan.test_path, &plan.test_content)?;
    Ok(())
}
