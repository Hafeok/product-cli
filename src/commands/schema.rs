//! `product schema` command — display front-matter schemas (ADR-031)

use product_lib::agent_context;

use super::BoxResult;

pub fn handle_schema(artifact_type: Option<String>, all: bool) -> BoxResult {
    if all {
        println!("{}", agent_context::generate_all_schemas());
        return Ok(());
    }

    match artifact_type {
        Some(t) => {
            let schema = agent_context::generate_schema(&t)
                .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
            println!("{}", schema);
        }
        None => {
            // No type specified and no --all flag — show help
            println!("{}", agent_context::generate_all_schemas());
        }
    }

    Ok(())
}
