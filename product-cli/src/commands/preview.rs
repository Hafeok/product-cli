//! `product preview …` — the §11/§12 PREVIEW profile validators (FT-141/142).
//!
//! Reads an external provider manifest (a design system's component catalog or a
//! content store's copy), validates its internal wholeness, and — with
//! `--couple` — confirms it couples to the captured What graph. Read-only; exits
//! 1 on any finding.

use clap::Subcommand;
use product_core::author::domain::session_dir;
use product_core::pf::session::DomainSession;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum PreviewCommands {
    /// Content-store manifest (§12.2) — validate wholeness; `--couple` checks it
    /// resolves every (content key, locale) the What's UI steps reference
    ContentStore {
        /// Path to the TOML content-store manifest
        manifest: PathBuf,
        /// Also run the coupling check against the captured What graph
        #[arg(long)]
        couple: bool,
        #[arg(long)]
        product: Option<String>,
    },
    /// Design-system manifest (§11.3) — validate wholeness; `--couple` checks
    /// reification coverage over the core AIOs × the What's contexts of use
    DesignSystem {
        /// Path to the YAML design-system manifest
        manifest: PathBuf,
        /// Also run the coupling check against the captured What graph
        #[arg(long)]
        couple: bool,
        #[arg(long)]
        product: Option<String>,
    },
    /// Render contract — project a flow's page graph + Abstract UI as the JSON a
    /// renderer consumes (the §11 coupling's other half)
    RenderContract {
        /// The flow id to project
        flow: String,
        /// A declared context of use to stamp on the contract
        #[arg(long)]
        context: Option<String>,
        /// Resolve content keys for this locale
        #[arg(long)]
        locale: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_preview(cmd: PreviewCommands) -> BoxResult {
    match cmd {
        PreviewCommands::DesignSystem { manifest, couple, product } => {
            design_system(manifest, couple, product)
        }
        PreviewCommands::ContentStore { manifest, couple, product } => {
            content_store(manifest, couple, product)
        }
        PreviewCommands::RenderContract { flow, context, locale, product } => {
            render_contract(flow, context, locale, product)
        }
    }
}

fn render_contract(flow: String, context: Option<String>, locale: Option<String>, product: Option<String>) -> BoxResult {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product — pass --product or set `name` in product.toml")?;
    let graph = load_what(Some(p.clone()))?;
    let contract = product_core::pf::render_contract::build(&graph, &flow, &p, context, locale)?;
    println!("{}", serde_json::to_string_pretty(&contract)?);
    Ok(())
}

fn content_store(manifest: PathBuf, couple: bool, product: Option<String>) -> BoxResult {
    let src = std::fs::read_to_string(&manifest)
        .map_err(|e| format!("cannot read manifest {}: {e}", manifest.display()))?;
    let m = product_core::pf::manifest_content::parse_content(&src)?;

    let mut findings = product_core::pf::manifest_content::validate_content(&m);
    if couple {
        let graph = load_what(product)?;
        findings.extend(product_core::pf::manifest_content::couple_content(&m, &graph));
    }

    let cs = &m.content_store;
    let summary = format!("content store '{}': {{scope}} — {} entries, {} locales",
        cs.id, cs.entries.len(), cs.locales_supported.len());
    report(&format!("content store '{}'", cs.id), couple, &summary, findings)
}

/// Print the scope line (whole / whole + coupled) on success, or each finding on
/// stderr and a non-zero exit on failure.
fn report(label: &str, couple: bool, summary_template: &str, findings: Vec<String>) -> BoxResult {
    let scope = if couple { "whole + coupled" } else { "whole" };
    if findings.is_empty() {
        println!("{}", summary_template.replace("{scope}", scope));
        return Ok(());
    }
    eprintln!("{label}: {} finding(s):", findings.len());
    for f in &findings {
        eprintln!("  - {f}");
    }
    Err(format!("{} manifest finding(s)", findings.len()).into())
}

fn design_system(manifest: PathBuf, couple: bool, product: Option<String>) -> BoxResult {
    let src = std::fs::read_to_string(&manifest)
        .map_err(|e| format!("cannot read manifest {}: {e}", manifest.display()))?;
    let m = product_core::pf::manifest::parse_ds(&src)?;

    let mut findings = product_core::pf::manifest::validate_ds(&m);
    if couple {
        let graph = load_what(product)?;
        findings.extend(product_core::pf::manifest::couple_ds(&m, &graph));
    }

    let ds = &m.design_system;
    let summary = format!("design system '{}': {{scope}} — {} components, {} reification rules",
        ds.id, ds.components.len(), ds.reification.len());
    report(&format!("design system '{}'", ds.id), couple, &summary, findings)
}

/// Load the captured What graph for the coupling check.
fn load_what(product: Option<String>) -> Result<product_core::pf::DomainGraph, Box<dyn std::error::Error>> {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product — pass --product or set `name` in product.toml")?;
    let dir = session_dir(&super::shared::domain_root(), &p);
    let session = DomainSession::load(&dir)
        .map_err(|_| format!("no domain graph for {p:?} — capture one with `product author domain`"))?;
    Ok(session.graph)
}
