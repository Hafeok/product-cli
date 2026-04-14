//! `product agent-init` command — generate AGENT.md from repo state (ADR-031)

use product_lib::{agent_context, config::ProductConfig, fileops, graph::KnowledgeGraph, parser};

use super::BoxResult;

pub fn handle_agent_init(watch: bool) -> BoxResult {
    let (config, root) = ProductConfig::discover()?;

    if watch {
        agent_context::watch_and_regenerate(&config, &root)?;
        return Ok(());
    }

    // Load the full graph
    let features_dir = config.resolve_path(&root, &config.paths.features);
    let adrs_dir = config.resolve_path(&root, &config.paths.adrs);
    let tests_dir = config.resolve_path(&root, &config.paths.tests);
    let deps_dir = config.resolve_path(&root, &config.paths.dependencies);

    let loaded = parser::load_all_with_deps(&features_dir, &adrs_dir, &tests_dir, Some(&deps_dir))?;

    // Print parse errors to stderr
    for e in &loaded.parse_errors {
        eprintln!("{}", e);
    }

    let graph = KnowledgeGraph::build_with_deps(
        loaded.features,
        loaded.adrs,
        loaded.tests,
        loaded.dependencies,
    );

    // Generate AGENT.md content
    let content = agent_context::generate_agent_md(&config, &graph, &root);

    // Write to configured output path
    let output_path = root.join(&config.agent_context.output_file);
    fileops::write_file_atomic(&output_path, &content)?;

    println!("Generated: {}", output_path.display());
    Ok(())
}
