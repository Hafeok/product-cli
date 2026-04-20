//! `product schema` command — display front-matter schemas (ADR-031, ADR-042)

use product_lib::agent_context::schema as sch;
use product_lib::config::ProductConfig;

use super::BoxResult;

pub fn handle_schema(artifact_type: Option<String>, all: bool) -> BoxResult {
    // Pass the loaded config (if any) so the TC schema can list custom types.
    let cfg = ProductConfig::discover().ok().map(|(c, _)| c);
    let cfg_ref = cfg.as_ref();

    if all {
        println!("{}", sch::generate_all_schemas_with_config(cfg_ref));
        return Ok(());
    }

    match artifact_type {
        Some(t) => {
            let schema = sch::generate_schema_with_config(&t, cfg_ref)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            println!("{}", schema);
        }
        None => {
            println!("{}", sch::generate_all_schemas_with_config(cfg_ref));
        }
    }

    Ok(())
}
