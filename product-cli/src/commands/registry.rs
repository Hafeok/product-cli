//! `product registry …` — minting a ground-registry instance from the template.
//!
//! `generate` renders the versioned registry template with typed parameters,
//! holds the result at the verification gate, writes the tree, records the
//! birth provenance in one local commit. It configures no remote: **publishing
//! is a separate act**, printed for a human to run.
//!
//! `check` reads a generated instance against its own rules — the file rule
//! plus the SHACL shapes it ships — reporting an unreadable shape distinctly
//! from data that violates one.

use std::path::{Path, PathBuf};

use clap::Subcommand;
use product_core::error::ProductError;
use product_core::registry::{
    apply_generation, check_instance, plan_generation, CheckReport, FindingKind, GenerationReport,
    RegistryParams,
};
use serde_json::json;

use super::output::{CmdResult, Output};

/// This generator's own version, recorded in every instance's birth provenance
/// beside the template's — so an instance can re-pin when either one moves.
const GENERATOR_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Subcommand)]
pub enum RegistryCommands {
    /// Read a generated instance against its own rules: the file rule, the shapes it ships
    Check {
        /// The instance's root directory
        dir: PathBuf,
    },
    /// Generate a registry instance from the versioned template into a fresh directory
    Generate {
        /// The owning organisation
        #[arg(long)]
        owner: String,
        /// The repository name
        #[arg(long)]
        repo: String,
        /// The named person whose merge is ratification
        #[arg(long)]
        ratifier: String,
        /// The registry's display name
        #[arg(long = "display-name")]
        display_name: String,
        /// The base IRI identifiers are minted under (ends in `#` or `/`)
        #[arg(long = "base-iri")]
        base_iri: String,
        /// Mint date, YYYY-MM-DD — when the authority was demonstrably controlled
        #[arg(long)]
        date: String,
        /// Who ran the generation, recorded in the birth provenance
        #[arg(long = "generated-by")]
        generated_by: String,
        /// Target directory — must not exist, or must be empty
        #[arg(long)]
        out: PathBuf,
    },
}

pub(crate) fn handle_registry(command: RegistryCommands) -> CmdResult {
    match command {
        RegistryCommands::Check { dir } => handle_check(&dir),
        RegistryCommands::Generate {
            owner,
            repo,
            ratifier,
            display_name,
            base_iri,
            date,
            generated_by,
            out,
        } => {
            let params = RegistryParams {
                owner,
                repo,
                ratifier,
                display_name,
                base_iri,
                mint_date: date,
                generated_by,
            };
            handle_generate(&params, &out)
        }
    }
}

fn handle_generate(params: &RegistryParams, out: &Path) -> CmdResult {
    let plan = plan_generation(params, GENERATOR_VERSION)?;
    let report = apply_generation(&plan, out)?;
    let sites: usize = plan.gate.sites_per_label.values().sum();
    Ok(Output::both(
        render_generation(params, &plan.template_version, sites, &report),
        json!({
            "out": report.out,
            "params": params,
            "template_version": plan.template_version,
            "generator_version": plan.generator_version,
            "files": report.files,
            "commit": report.commit,
            "parameter_sites_verified": sites,
            "publish": report.publish,
        }),
    ))
}

fn render_generation(
    params: &RegistryParams,
    template_version: &str,
    sites: usize,
    report: &GenerationReport,
) -> String {
    let mut s = format!(
        "generated '{}' at {}\n  template {} · generator {} · minted {}\n  \
         {} files · gate clean ({} parameter sites verified byte-identical)\n  birth commit {}\n",
        params.display_name,
        report.out.display(),
        template_version,
        GENERATOR_VERSION,
        params.mint_date,
        report.files.len(),
        sites,
        report.commit,
    );
    s.push_str("\nnothing has been published — generation is local, publishing is a separate act:\n");
    for line in &report.publish {
        s.push_str(&format!("  {line}\n"));
    }
    s.push_str(
        "\nthe founding decision is the instance's first ratified content, filed by its ratifier.\n",
    );
    s
}

fn handle_check(dir: &Path) -> CmdResult {
    let report = check_instance(dir)?;
    let text = render_check(dir, &report);
    if report.conforms() {
        return Ok(Output::both(text, check_json(&report)));
    }
    Err(ProductError::ConfigError(text))
}

fn render_check(dir: &Path, report: &CheckReport) -> String {
    let unevaluable = report.of_kind(FindingKind::Unevaluable);
    let violations = report.of_kind(FindingKind::ShapeViolation);
    let file_rule = report.of_kind(FindingKind::FileRule);
    let mut s = format!(
        "registry check {} — {} data file(s), {} constraint(s) evaluated\n",
        dir.display(),
        report.files_checked,
        report.constraints_evaluated
    );
    if report.conforms() {
        s.push_str("  conformant: file rule clean, every shape read, no violation\n");
        return s;
    }
    s.push_str(&format!(
        "  FAILED — {} file-rule finding(s), {} shape violation(s), {} unreadable shape(s)\n",
        file_rule.len(),
        violations.len(),
        unevaluable.len()
    ));
    if !unevaluable.is_empty() {
        s.push_str("\nunreadable shapes (the check fails closed — no data was judged against these):\n");
        for f in &unevaluable {
            s.push_str(&format!("  {} — {}\n", f.focus, f.message));
        }
    }
    push_findings(&mut s, "file rule", &file_rule);
    push_findings(&mut s, "shape violations", &violations);
    s
}

fn push_findings(s: &mut String, heading: &str, findings: &[&product_core::registry::Finding]) {
    if findings.is_empty() {
        return;
    }
    s.push_str(&format!("\n{heading}:\n"));
    for f in findings {
        let where_ = f.file.clone().unwrap_or_else(|| f.focus.clone());
        s.push_str(&format!("  {where_} — {}\n", f.message));
    }
}

fn check_json(report: &CheckReport) -> serde_json::Value {
    json!({
        "conforms": report.conforms(),
        "files_checked": report.files_checked,
        "constraints_evaluated": report.constraints_evaluated,
        "findings": report.findings,
    })
}
