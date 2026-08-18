//! Planning a generation: rendering the template, then holding it at the gate.
//!
//! `plan_generation` is pure — it returns the whole tree in memory, already
//! verified. Nothing reaches disk until [`super::apply`] is called with the
//! plan, so a gate failure leaves no directory behind at all.

use crate::error::Result;

use super::params::RegistryParams;
use super::substitute::{render, Site, Token};
use super::template::{BASE_IRI_TOKEN, TEMPLATE, TEMPLATE_VERSION};
use super::verify::{gate, GateReport, Gated};

/// One file of a planned instance.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlannedFile {
    /// Path relative to the instance root.
    pub path: String,
    /// The rendered contents.
    pub contents: String,
    /// Whether the file is written executable.
    pub executable: bool,
    /// Spans the renderer wrote parameter values into.
    pub sites: Vec<Site>,
}

/// A verified instance, held in memory.
#[derive(Debug, serde::Serialize)]
pub struct GenerationPlan {
    /// The parameters it was minted from.
    pub params: RegistryParams,
    /// The template version the tree came from.
    pub template_version: String,
    /// The generator version that rendered it.
    pub generator_version: String,
    /// Every file, in template order.
    pub files: Vec<PlannedFile>,
    /// What the gate saw.
    pub gate: GateReport,
}

/// Render the template with `params` and run the gate. Returns the verified
/// tree, or the first fault — in which case nothing has been written.
pub fn plan_generation(params: &RegistryParams, generator_version: &str) -> Result<GenerationPlan> {
    params.validate()?;
    let tokens = token_table(params, generator_version);
    let files = render_files(&tokens);
    let report = gate(&gated(&files), &tokens)?;
    Ok(GenerationPlan {
        params: params.clone(),
        template_version: TEMPLATE_VERSION.to_string(),
        generator_version: generator_version.to_string(),
        files,
        gate: report,
    })
}

/// Render every template file, leaving the verbatim ones untouched.
fn render_files(tokens: &[Token]) -> Vec<PlannedFile> {
    TEMPLATE
        .iter()
        .map(|f| {
            let rendered = if f.substituted {
                render(f.text, tokens)
            } else {
                super::substitute::Rendered { text: f.text.to_string(), sites: Vec::new() }
            };
            PlannedFile {
                path: f.path.to_string(),
                contents: rendered.text,
                executable: f.executable,
                sites: rendered.sites,
            }
        })
        .collect()
}

/// Pair each rendered file with the template text it came from, for the gate.
fn gated<'a>(files: &'a [PlannedFile]) -> Vec<Gated<'a>> {
    TEMPLATE
        .iter()
        .zip(files.iter())
        .map(|(t, f)| Gated {
            path: &f.path,
            source: t.text,
            rendered: &f.contents,
            sites: &f.sites,
            substituted: t.substituted,
        })
        .collect()
}

/// The closed substitution table. Every placeholder the template carries has an
/// entry here; a placeholder without one fails the gate rather than shipping.
pub(crate) fn token_table<'a>(params: &'a RegistryParams, generator_version: &'a str) -> Vec<Token<'a>> {
    vec![
        Token { label: "--owner", token: "{{OWNER_ORG}}", value: &params.owner },
        Token { label: "--repo", token: "{{REPO_NAME}}", value: &params.repo },
        Token { label: "--ratifier", token: "{{RATIFIER}}", value: &params.ratifier },
        Token { label: "--display-name", token: "{{DISPLAY_NAME}}", value: &params.display_name },
        Token { label: "--date", token: "{{MINT_DATE}}", value: &params.mint_date },
        Token { label: "--generated-by", token: "{{GENERATED_BY}}", value: &params.generated_by },
        Token { label: "template version", token: "{{TEMPLATE_VERSION}}", value: TEMPLATE_VERSION },
        Token { label: "generator version", token: "{{GENERATOR_VERSION}}", value: generator_version },
        Token { label: "--base-iri", token: BASE_IRI_TOKEN, value: &params.base_iri },
    ]
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
