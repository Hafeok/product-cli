//! `product design-system …` — the §11 design system as an addressable artifact.
//!
//! Vendors a design-system manifest (plus its implementation bundle) under
//! `.product/design-systems/<id>/`, parallel to blueprints and deliverables;
//! `bind` records the chosen system on the How contract (§4.5), which is what
//! `product codegen` resolves — the choice is a graph fact, not a CLI flag.

use clap::Subcommand;
use product_core::pf::ds_store;
use product_core::pf::HowContract;
use std::path::PathBuf;

use super::BoxResult;

#[derive(Subcommand)]
pub enum DesignSystemCommands {
    /// Validate a manifest's declaration half (§11.3) and vendor it (plus its
    /// implementation sources) into `.product/design-systems/<id>/`
    Add {
        /// Path to the YAML design-system manifest
        manifest: PathBuf,
        #[arg(long)]
        product: Option<String>,
    },
    /// Bind a stored design system to the How contract's screen-composition
    /// contract (§4.5) — the system `product codegen` resolves
    Bind {
        /// A stored design-system id
        id: String,
        #[arg(long)]
        product: Option<String>,
    },
    /// Coupling check (§11.2): every AIO the What references reifies in every
    /// declared context of use
    Couple {
        /// A stored design-system id (default: the How-bound one)
        id: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// List the stored design systems
    List {
        #[arg(long)]
        product: Option<String>,
    },
    /// Show a stored design system: identity, catalog, coverage, token surface
    Show {
        /// A stored design-system id (default: the How-bound one)
        id: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
    /// Validate a stored design system: declaration wholeness (§11.3) plus the
    /// bundle check (implementations per target, token values per theme)
    Validate {
        /// A stored design-system id (default: the How-bound one)
        id: Option<String>,
        #[arg(long)]
        product: Option<String>,
    },
}

pub(crate) fn handle_design_system(cmd: DesignSystemCommands) -> BoxResult {
    match cmd {
        DesignSystemCommands::Add { manifest, product } => add(manifest, product),
        DesignSystemCommands::Bind { id, product } => bind(id, product),
        DesignSystemCommands::Couple { id, product } => couple(id, product),
        DesignSystemCommands::List { product } => list(product),
        DesignSystemCommands::Show { id, product } => show(id, product),
        DesignSystemCommands::Validate { id, product } => validate(id, product),
    }
}

/// The `.product` base directory the store lives under.
fn base(product: Option<&str>) -> PathBuf {
    super::shared::artifact_dir(product, "")
}

fn add(manifest: PathBuf, product: Option<String>) -> BoxResult {
    let _lock = super::shared::acquire_write_lock()?;
    let stored = ds_store::save(&base(product.as_deref()), &manifest)?;
    let ds = &stored.manifest.design_system;
    println!(
        "added design system '{}' v{} → {} — {} component(s), {} reification rule(s), {} token(s)",
        ds.id, ds.version, stored.dir.display(),
        ds.components.len(), ds.reification.len(), ds.tokens.len(),
    );
    println!("  hash sha256:{}", stored.hash());
    println!("  bind it with `product design-system bind {}`", ds.id);
    Ok(())
}

fn bind(id: String, product: Option<String>) -> BoxResult {
    let _lock = super::shared::acquire_write_lock()?;
    let stored = ds_store::load(&base(product.as_deref()), &id)?;
    let path = base(product.as_deref()).join("how-contract.yaml");
    let mut c = HowContract::load_opt(&path)?.unwrap_or_else(|| HowContract {
        blueprint: super::shared::default_product_name().unwrap_or_else(|| "blueprint".to_string()),
        ..Default::default()
    });
    let ds = &stored.manifest.design_system;
    c.design_system = Some(product_core::pf::how::DesignSystemBinding {
        id: ds.id.clone(),
        version: (!ds.version.is_empty()).then(|| ds.version.clone()),
    });
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, c.to_yaml()?)?;
    println!("bound design system '{}' v{} to the How contract (§4.5)", ds.id, ds.version);
    Ok(())
}

fn list(product: Option<String>) -> BoxResult {
    let b = base(product.as_deref());
    let ids = ds_store::list(&b);
    if ids.is_empty() {
        println!("no design systems stored — add one with `product design-system add <manifest>`");
        return Ok(());
    }
    let bound = bound_id(product.as_deref());
    for id in ids {
        let mark = if Some(&id) == bound.as_ref() { "  (bound)" } else { "" };
        match ds_store::load(&b, &id) {
            Ok(s) => {
                let ds = &s.manifest.design_system;
                println!("{id}  v{}  {} component(s), {} rule(s), {} token(s){mark}",
                    ds.version, ds.components.len(), ds.reification.len(), ds.tokens.len());
            }
            Err(_) => println!("{id}  (unreadable manifest){mark}"),
        }
    }
    Ok(())
}

fn show(id: Option<String>, product: Option<String>) -> BoxResult {
    let stored = resolve(id, product.as_deref())?;
    let ds = &stored.manifest.design_system;
    let v = serde_json::json!({
        "id": ds.id,
        "version": ds.version,
        "hash": format!("sha256:{}", stored.hash()),
        "wcag_target": ds.wcag_target,
        "targets": ds.targets,
        "themes": ds.themes,
        "components": ds.components.iter().map(|c| c.id.clone()).collect::<Vec<_>>(),
        "reification_rules": ds.reification.len(),
        "tokens": ds.tokens.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
        "templates": ds.templates.iter().map(|t| t.id.clone()).collect::<Vec<_>>(),
        "dir": stored.dir.display().to_string(),
    });
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}

fn validate(id: Option<String>, product: Option<String>) -> BoxResult {
    let stored = resolve(id, product.as_deref())?;
    let mut findings = product_core::pf::manifest::validate_ds(&stored.manifest);
    findings.extend(product_core::pf::manifest_bundle::validate_bundle(&stored.manifest, &stored.dir));
    let ds = &stored.manifest.design_system;
    report(&format!("design system '{}'", ds.id), findings, || {
        format!("design system '{}': whole + bundled — {} component(s) over {} target(s), {} theme(s)",
            ds.id, ds.components.len(), ds.targets.len(), ds.themes.len())
    })
}

fn couple(id: Option<String>, product: Option<String>) -> BoxResult {
    let stored = resolve(id, product.as_deref())?;
    let graph = load_what(product)?;
    let findings = product_core::pf::manifest::couple_ds(&stored.manifest, &graph);
    let ds_id = stored.manifest.design_system.id.clone();
    report(&format!("design system '{ds_id}'"), findings, || {
        format!("design system '{ds_id}': coupled — every referenced AIO reifies in every declared context (§11.2)")
    })
}

fn report(label: &str, findings: Vec<String>, ok: impl Fn() -> String) -> BoxResult {
    if findings.is_empty() {
        println!("{}", ok());
        return Ok(());
    }
    eprintln!("{label}: {} finding(s):", findings.len());
    for f in &findings {
        eprintln!("  - {f}");
    }
    Err(format!("{} design-system finding(s)", findings.len()).into())
}

/// Resolve a store id: the given one, else the How contract's binding.
fn resolve(id: Option<String>, product: Option<&str>) -> Result<ds_store::StoredDs, Box<dyn std::error::Error>> {
    let id = match id.or_else(|| bound_id(product)) {
        Some(id) => id,
        None => return Err("no design system named and none bound — pass an id or `product design-system bind <id>`".into()),
    };
    Ok(ds_store::load(&base(product), &id)?)
}

/// The How-bound design system as a reify input, when one is bound (§4.5).
pub(crate) fn load_bound_ds(product: Option<&str>) -> Result<Option<product_core::pf::reify_ds::DsSpec>, Box<dyn std::error::Error>> {
    let Some(id) = bound_id(product) else { return Ok(None) };
    let stored = product_core::pf::ds_store::load(&super::shared::artifact_dir(product, ""), &id)?;
    Ok(Some(product_core::pf::reify_ds::DsSpec::from_source(stored.manifest.clone(), &stored.source)))
}

/// Design-system drift: the tree was generated against a pinned manifest hash;
/// fail when the stored manifest has moved past it (or the binding changed).
pub(crate) fn check_ds_drift(provenance: &str, product: Option<&str>) -> BoxResult {
    let Some((rec_id, rec_hash)) = product_core::pf::codegen::recorded_ds(provenance) else { return Ok(()) };
    let Some(spec) = load_bound_ds(product)? else {
        return Err(format!(
            "drift — the tree was generated with design system '{rec_id}', but the How no longer binds one"
        ).into());
    };
    if spec.manifest.design_system.id != rec_id || spec.hash != rec_hash {
        return Err(format!(
            "drift — the design system has moved past the generated code:\n  generated from '{rec_id}' sha256:{rec_hash}\n  currently bound '{}' sha256:{}\n  regenerate with `product codegen csharp` / `product codegen web`",
            spec.manifest.design_system.id, spec.hash
        ).into());
    }
    Ok(())
}

/// The How contract's bound design-system id, if any.
pub(crate) fn bound_id(product: Option<&str>) -> Option<String> {
    let path = super::shared::artifact_dir(product, "").join("how-contract.yaml");
    HowContract::load_opt(&path).ok().flatten().and_then(|c| c.design_system).map(|b| b.id)
}

fn load_what(product: Option<String>) -> Result<product_core::pf::DomainGraph, Box<dyn std::error::Error>> {
    use product_core::author::domain::session_dir;
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product — pass --product or set `name` in product.toml")?;
    let session = product_core::pf::session::DomainSession::load(&session_dir(&super::shared::domain_root(), &p))
        .map_err(|_| format!("no domain graph for {p:?} — capture one with `product author domain`"))?;
    Ok(session.graph)
}
