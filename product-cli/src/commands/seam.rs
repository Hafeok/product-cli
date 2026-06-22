//! `product seam <step>` — the §6.3 seam verification adapter.
//!
//! Loads the captured What graph and runs `pf::seam::seam_verdict`, printing the
//! composite verdict (each sub-check with its findings) and exiting 1 if any
//! sub-check fails.

use product_core::author::domain::session_dir;
use product_core::pf::session::DomainSession;

use super::BoxResult;

pub(crate) fn handle_seam(id: String, product: Option<String>) -> BoxResult {
    let p = product
        .or_else(super::shared::default_product_name)
        .ok_or("no product — pass --product or set `name` in product.toml")?;
    let dir = session_dir(&super::shared::domain_root(), &p);
    let session = DomainSession::load(&dir).map_err(|_| {
        format!("no domain graph for {p:?} — capture one with `product author domain`")
    })?;

    let verdict = product_core::pf::seam::seam_verdict(&session.graph, &id)
        .ok_or_else(|| format!("no UI step with id {id:?} in the graph"))?;

    println!(
        "Seam verdict for {id}: {}",
        if verdict.conformant { "conformant" } else { "NOT conformant" }
    );
    for check in &verdict.checks {
        let mark = if check.passed { "✓" } else { "✗" };
        println!("  {mark} {}", check.name);
        for f in &check.findings {
            eprintln!("      - {f}");
        }
    }
    if verdict.conformant {
        Ok(())
    } else {
        let failed = verdict.checks.iter().filter(|c| !c.passed).count();
        Err(format!("seam: {id} fails {failed} sub-check(s)").into())
    }
}
