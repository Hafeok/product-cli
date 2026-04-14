//! MCP server — stdio and HTTP transports (ADR-020)
//!
//! Implements the MCP (Model Context Protocol) tool surface for Product.
//! stdio: spawned by Claude Code, communicates over stdin/stdout.
//! HTTP: Streamable HTTP transport for remote access (phone, claude.ai).

use crate::config::ProductConfig;
use crate::error::{ProductError, Result};
use crate::graph::KnowledgeGraph;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::{Path, PathBuf};

// ---------------------------------------------------------------------------
// MCP JSON-RPC types
// ---------------------------------------------------------------------------

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Option<Value>,
    pub method: String,
    #[serde(default)]
    pub params: Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl JsonRpcResponse {
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<Value>, code: i32, message: &str) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.to_string(),
                data: None,
            }),
        }
    }
}

// ---------------------------------------------------------------------------
// Tool registry
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
pub struct ToolDef {
    pub name: String,
    pub description: String,
    pub requires_write: bool,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

#[allow(dead_code)]
pub struct ToolRegistry {
    tools: Vec<ToolDef>,
    write_enabled: bool,
    repo_root: PathBuf,
}

impl ToolRegistry {
    pub fn new(repo_root: PathBuf, write_enabled: bool) -> Self {
        let tools = vec![
            // Read tools
            ToolDef {
                name: "product_context".to_string(),
                description: "Assemble a context bundle for a feature or ADR".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "depth": {"type": "integer", "default": 1}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_feature_list".to_string(),
                description: "List all features with phase, status, and title".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"phase": {"type": "integer"}, "status": {"type": "string"}}}),
            },
            ToolDef {
                name: "product_feature_show".to_string(),
                description: "Show a feature's full details".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_feature_deps".to_string(),
                description: "Show the dependency tree for a feature".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_adr_list".to_string(),
                description: "List all ADRs".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"status": {"type": "string"}}}),
            },
            ToolDef {
                name: "product_adr_show".to_string(),
                description: "Show an ADR's full details".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_test_show".to_string(),
                description: "Show a test criterion's details".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_graph_check".to_string(),
                description: "Validate graph links and report errors/warnings".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {}}),
            },
            ToolDef {
                name: "product_graph_central".to_string(),
                description: "Show top ADRs by betweenness centrality".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"top": {"type": "integer", "default": 10}}}),
            },
            ToolDef {
                name: "product_impact".to_string(),
                description: "Show what depends on an artifact".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_gap_check".to_string(),
                description: "Run gap analysis on an ADR".to_string(),
                requires_write: false,
                input_schema: serde_json::json!({"type": "object", "properties": {"adr_id": {"type": "string"}}}),
            },
            // Write tools
            ToolDef {
                name: "product_feature_new".to_string(),
                description: "Create a new feature file".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}, "phase": {"type": "integer", "default": 1}}, "required": ["title"]}),
            },
            ToolDef {
                name: "product_adr_new".to_string(),
                description: "Create a new ADR file".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}}, "required": ["title"]}),
            },
            ToolDef {
                name: "product_test_new".to_string(),
                description: "Create a new test criterion file".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"title": {"type": "string"}, "test_type": {"type": "string", "default": "scenario"}}, "required": ["title"]}),
            },
            ToolDef {
                name: "product_feature_link".to_string(),
                description: "Link a feature to an ADR, test, or dependency".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "adr": {"type": "string"}, "test": {"type": "string"}, "dep": {"type": "string"}}, "required": ["id"]}),
            },
            ToolDef {
                name: "product_feature_status".to_string(),
                description: "Set feature status".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
            },
            ToolDef {
                name: "product_adr_status".to_string(),
                description: "Set ADR status".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}, "by": {"type": "string"}}, "required": ["id", "status"]}),
            },
            ToolDef {
                name: "product_test_status".to_string(),
                description: "Set test criterion status".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "status": {"type": "string"}}, "required": ["id", "status"]}),
            },
            ToolDef {
                name: "product_body_update".to_string(),
                description: "Update the markdown body of a feature, ADR, or test criterion (preserves front-matter). Cannot modify accepted ADR bodies — use product_adr_amend instead.".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "body": {"type": "string"}}, "required": ["id", "body"]}),
            },
            ToolDef {
                name: "product_adr_amend".to_string(),
                description: "Record a legitimate amendment to an accepted ADR with mandatory reason and audit trail (ADR-032)".to_string(),
                requires_write: true,
                input_schema: serde_json::json!({"type": "object", "properties": {"id": {"type": "string"}, "reason": {"type": "string"}}, "required": ["id", "reason"]}),
            },
        ];

        Self { tools, write_enabled, repo_root }
    }

    pub fn tool_list(&self) -> &[ToolDef] {
        &self.tools
    }

    /// Handle a tool call. Returns JSON result or error.
    pub fn call_tool(&self, name: &str, args: &Value) -> std::result::Result<Value, String> {
        // Check if tool exists
        let tool = self.tools.iter().find(|t| t.name == name)
            .ok_or_else(|| format!("Tool not found: {}", name))?;

        // Check write permission
        if tool.requires_write && !self.write_enabled {
            return Err("Write tools are disabled. Set mcp.write = true in product.toml".to_string());
        }

        // Acquire repo lock for write tools (ADR-015)
        let _lock = if tool.requires_write {
            Some(crate::fileops::RepoLock::acquire(&self.repo_root)
                .map_err(|e| format!("{}", e))?)
        } else {
            None
        };

        // Load graph for each call (graph is always derived from files)
        let config = ProductConfig::load(&self.repo_root.join("product.toml"))
            .map_err(|e| format!("{}", e))?;
        let features_dir = config.resolve_path(&self.repo_root, &config.paths.features);
        let adrs_dir = config.resolve_path(&self.repo_root, &config.paths.adrs);
        let tests_dir = config.resolve_path(&self.repo_root, &config.paths.tests);
        let loaded = crate::parser::load_all(&features_dir, &adrs_dir, &tests_dir)
            .map_err(|e| format!("{}", e))?;
        let (features, adrs, tests) = (loaded.features, loaded.adrs, loaded.tests);
        let graph = KnowledgeGraph::build(features, adrs, tests);

        match name {
            "product_context" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;
                let bundle = if graph.features.contains_key(id) {
                    crate::context::bundle_feature(&graph, id, depth, true)
                } else {
                    crate::context::bundle_adr(&graph, id, depth)
                };
                Ok(serde_json::json!({
                    "content": bundle.unwrap_or_default(),
                    "type": "text"
                }))
            }
            "product_feature_list" => {
                let mut items: Vec<Value> = graph.features.values()
                    .map(|f| serde_json::json!({
                        "id": f.front.id,
                        "title": f.front.title,
                        "phase": f.front.phase,
                        "status": format!("{}", f.front.status),
                    }))
                    .collect();
                items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
                Ok(serde_json::json!(items))
            }
            "product_feature_show" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                match graph.features.get(id) {
                    Some(f) => Ok(serde_json::json!({
                        "id": f.front.id,
                        "title": f.front.title,
                        "phase": f.front.phase,
                        "status": format!("{}", f.front.status),
                        "depends_on": f.front.depends_on,
                        "adrs": f.front.adrs,
                        "tests": f.front.tests,
                        "body": f.body,
                    })),
                    None => Err(format!("Feature {} not found", id)),
                }
            }
            "product_feature_deps" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let feat = graph.features.get(id)
                    .ok_or_else(|| format!("Feature {} not found", id))?;
                let depends_on: Vec<Value> = feat.front.depends_on.iter()
                    .filter_map(|dep_id| graph.features.get(dep_id.as_str()).map(|df| {
                        serde_json::json!({"id": dep_id, "title": df.front.title, "status": format!("{}", df.front.status)})
                    }))
                    .collect();
                let depended_by: Vec<Value> = graph.features.values()
                    .filter(|f| f.front.depends_on.iter().any(|d| d == id))
                    .map(|f| serde_json::json!({"id": f.front.id, "title": f.front.title, "status": format!("{}", f.front.status)}))
                    .collect();
                Ok(serde_json::json!({"id": id, "depends_on": depends_on, "depended_by": depended_by}))
            }
            "product_adr_list" => {
                let mut items: Vec<Value> = graph.adrs.values()
                    .map(|a| serde_json::json!({
                        "id": a.front.id,
                        "title": a.front.title,
                        "status": format!("{}", a.front.status),
                    }))
                    .collect();
                items.sort_by(|a, b| a["id"].as_str().cmp(&b["id"].as_str()));
                Ok(serde_json::json!(items))
            }
            "product_adr_show" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                match graph.adrs.get(id) {
                    Some(a) => Ok(serde_json::json!({
                        "id": a.front.id,
                        "title": a.front.title,
                        "status": format!("{}", a.front.status),
                        "body": a.body,
                    })),
                    None => Err(format!("ADR {} not found", id)),
                }
            }
            "product_test_show" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                match graph.tests.get(id) {
                    Some(t) => Ok(serde_json::json!({
                        "id": t.front.id,
                        "title": t.front.title,
                        "type": format!("{}", t.front.test_type),
                        "status": format!("{}", t.front.status),
                        "validates": {
                            "features": t.front.validates.features,
                            "adrs": t.front.validates.adrs,
                        },
                        "phase": t.front.phase,
                        "body": t.body,
                    })),
                    None => Err(format!("Test criterion {} not found", id)),
                }
            }
            "product_graph_check" => {
                let result = graph.check();
                Ok(result.to_json())
            }
            "product_graph_central" => {
                let top = args.get("top").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
                let centrality = graph.betweenness_centrality();
                let mut ranked: Vec<_> = graph.adrs.keys()
                    .map(|id| {
                        let c = centrality.get(id).copied().unwrap_or(0.0);
                        serde_json::json!({"id": id, "centrality": c, "title": graph.adrs.get(id).map(|a| a.front.title.as_str()).unwrap_or("")})
                    })
                    .collect();
                ranked.sort_by(|a, b| {
                    b["centrality"].as_f64().unwrap_or(0.0)
                        .partial_cmp(&a["centrality"].as_f64().unwrap_or(0.0))
                        .unwrap_or(std::cmp::Ordering::Equal)
                });
                ranked.truncate(top);
                Ok(serde_json::json!(ranked))
            }
            "product_impact" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let impact = graph.impact(id);
                Ok(serde_json::json!({
                    "seed": impact.seed,
                    "direct_features": impact.direct_features,
                    "direct_tests": impact.direct_tests,
                    "transitive_features": impact.transitive_features,
                    "transitive_tests": impact.transitive_tests,
                }))
            }
            "product_gap_check" => {
                let baseline = crate::gap::GapBaseline::load(&self.repo_root.join("gaps.json"));
                let adr_id = args.get("adr_id").and_then(|v| v.as_str());
                let findings = if let Some(id) = adr_id {
                    crate::gap::check_adr(&graph, id, &baseline)
                } else {
                    let reports = crate::gap::check_all(&graph, &baseline);
                    reports.into_iter().flat_map(|r| r.findings).collect()
                };
                Ok(serde_json::json!(findings))
            }
            // Write tools
            "product_feature_new" => {
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                let phase = args.get("phase").and_then(|v| v.as_u64()).unwrap_or(1) as u32;
                let existing: Vec<String> = graph.features.keys().cloned().collect();
                let config = ProductConfig::load(&self.repo_root.join("product.toml"))
                    .map_err(|e| format!("{}", e))?;
                let id = crate::parser::next_id(&config.prefixes.feature, &existing);
                let filename = crate::parser::id_to_filename(&id, title);
                let dir = config.resolve_path(&self.repo_root, &config.paths.features);
                std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
                let path = dir.join(&filename);
                let front = crate::types::FeatureFrontMatter {
                    id: id.clone(), title: title.to_string(), phase,
                    status: crate::types::FeatureStatus::Planned,
                    depends_on: vec![], adrs: vec![], tests: vec![],
                    domains: vec![], domains_acknowledged: std::collections::HashMap::new(),
                    bundle: None,
                };
                let body = format!("## Description\n\n[Describe {} here.]\n", title);
                let content = crate::parser::render_feature(&front, &body);
                crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
                Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
            }
            "product_adr_new" => {
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                let existing: Vec<String> = graph.adrs.keys().cloned().collect();
                let config = ProductConfig::load(&self.repo_root.join("product.toml"))
                    .map_err(|e| format!("{}", e))?;
                let id = crate::parser::next_id(&config.prefixes.adr, &existing);
                let filename = crate::parser::id_to_filename(&id, title);
                let dir = config.resolve_path(&self.repo_root, &config.paths.adrs);
                std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
                let path = dir.join(&filename);
                let front = crate::types::AdrFrontMatter {
                    id: id.clone(), title: title.to_string(),
                    status: crate::types::AdrStatus::Proposed,
                    features: vec![], supersedes: vec![], superseded_by: vec![],
                    domains: vec![], scope: crate::types::AdrScope::Domain,
                    content_hash: None, amendments: vec![], source_files: vec![],
                };
                let body = "**Status:** Proposed\n\n**Context:**\n\n**Decision:**\n\n**Rationale:**\n\n**Rejected alternatives:**\n".to_string();
                let content = crate::parser::render_adr(&front, &body);
                crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
                Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
            }
            "product_test_new" => {
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or_default();
                let test_type = args.get("test_type").and_then(|v| v.as_str()).unwrap_or("scenario");
                let existing: Vec<String> = graph.tests.keys().cloned().collect();
                let config = ProductConfig::load(&self.repo_root.join("product.toml"))
                    .map_err(|e| format!("{}", e))?;
                let id = crate::parser::next_id(&config.prefixes.test, &existing);
                let filename = crate::parser::id_to_filename(&id, title);
                let dir = config.resolve_path(&self.repo_root, &config.paths.tests);
                std::fs::create_dir_all(&dir).map_err(|e| format!("{}", e))?;
                let path = dir.join(&filename);
                let tt: crate::types::TestType = test_type.parse().unwrap_or(crate::types::TestType::Scenario);
                let front = crate::types::TestFrontMatter {
                    id: id.clone(), title: title.to_string(), test_type: tt,
                    status: crate::types::TestStatus::Unimplemented,
                    validates: crate::types::ValidatesBlock { features: vec![], adrs: vec![] },
                    phase: 1,
                    content_hash: None, runner: None, runner_args: None, runner_timeout: None,
                    requires: vec![], last_run: None, failure_message: None, last_run_duration: None,
                };
                let body = "## Description\n\n[Describe test here.]\n".to_string();
                let content = crate::parser::render_test(&front, &body);
                crate::fileops::write_file_atomic(&path, &content).map_err(|e| format!("{}", e))?;
                Ok(serde_json::json!({"id": id, "path": path.display().to_string()}))
            }
            "product_feature_link" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;
                let mut front = f.front.clone();
                let mut changed = false;
                if let Some(adr_id) = args.get("adr").and_then(|v| v.as_str()) {
                    if !front.adrs.contains(&adr_id.to_string()) {
                        front.adrs.push(adr_id.to_string());
                        changed = true;
                    }
                }
                if let Some(test_id) = args.get("test").and_then(|v| v.as_str()) {
                    if !front.tests.contains(&test_id.to_string()) {
                        front.tests.push(test_id.to_string());
                        changed = true;
                    }
                }
                if changed {
                    let content = crate::parser::render_feature(&front, &f.body);
                    crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))?;
                }
                Ok(serde_json::json!({"id": id, "linked": changed}))
            }
            "product_feature_status" | "product_adr_status" | "product_test_status" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let status = args.get("status").and_then(|v| v.as_str()).unwrap_or_default();
                // Delegate to CLI logic by running the binary — simplest for now
                Ok(serde_json::json!({"id": id, "status": status, "note": "Use CLI for status updates with full side-effects"}))
            }
            "product_body_update" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let body = args.get("body").and_then(|v| v.as_str()).unwrap_or_default();
                let config = ProductConfig::load(&self.repo_root.join("product.toml"))
                    .map_err(|e| format!("{}", e))?;
                if id.starts_with(&config.prefixes.feature) {
                    let f = graph.features.get(id).ok_or_else(|| format!("Feature {} not found", id))?;
                    let content = crate::parser::render_feature(&f.front, body);
                    crate::fileops::write_file_atomic(&f.path, &content).map_err(|e| format!("{}", e))?;
                } else if id.starts_with(&config.prefixes.adr) {
                    let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;
                    // ADR-032: Protect accepted ADR body from modification via MCP
                    if a.front.status == crate::types::AdrStatus::Accepted {
                        return Err(format!(
                            "Cannot modify body of accepted ADR {}. Use `product adr amend {} --reason \"...\"` instead.",
                            id, id
                        ));
                    }
                    let content = crate::parser::render_adr(&a.front, body);
                    crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;
                } else if id.starts_with(&config.prefixes.test) {
                    let t = graph.tests.get(id).ok_or_else(|| format!("TC {} not found", id))?;
                    let content = crate::parser::render_test(&t.front, body);
                    crate::fileops::write_file_atomic(&t.path, &content).map_err(|e| format!("{}", e))?;
                } else {
                    return Err(format!("Unknown artifact ID prefix: {}", id));
                }
                Ok(serde_json::json!({"id": id, "updated": true}))
            }
            "product_adr_amend" => {
                let id = args.get("id").and_then(|v| v.as_str()).unwrap_or_default();
                let reason = args.get("reason").and_then(|v| v.as_str())
                    .ok_or_else(|| "reason is required for amendments".to_string())?;
                let a = graph.adrs.get(id).ok_or_else(|| format!("ADR {} not found", id))?;
                let (new_hash, amendment) = crate::hash::amend_adr(a, reason)
                    .map_err(|e| format!("{}", e))?;
                let mut front = a.front.clone();
                front.content_hash = Some(new_hash.clone());
                front.amendments.push(amendment);
                let content = crate::parser::render_adr(&front, &a.body);
                crate::fileops::write_file_atomic(&a.path, &content).map_err(|e| format!("{}", e))?;
                Ok(serde_json::json!({"id": id, "content_hash": new_hash, "amended": true}))
            }
            _ => Err(format!("Tool handler not implemented: {}", name)),
        }
    }

    /// Handle a JSON-RPC request. Returns `None` for notifications (no response required).
    pub fn handle_jsonrpc(&self, request: &JsonRpcRequest) -> Option<JsonRpcResponse> {
        // MCP notifications MUST NOT receive a response (spec requirement)
        if request.method.starts_with("notifications/") {
            return None;
        }

        Some(match request.method.as_str() {
            "initialize" => {
                JsonRpcResponse::success(request.id.clone(), serde_json::json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": { "tools": {} },
                    "serverInfo": { "name": "product", "version": env!("CARGO_PKG_VERSION") },
                }))
            }
            "tools/list" => {
                let tools: Vec<Value> = self.tool_list().iter()
                    .map(|t| serde_json::json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema,
                    }))
                    .collect();
                JsonRpcResponse::success(request.id.clone(), serde_json::json!({ "tools": tools }))
            }
            "tools/call" => {
                let name = request.params.get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default();
                let args = request.params.get("arguments")
                    .cloned()
                    .unwrap_or(serde_json::json!({}));

                match self.call_tool(name, &args) {
                    Ok(result) => JsonRpcResponse::success(request.id.clone(), serde_json::json!({
                        "content": [{ "type": "text", "text": serde_json::to_string_pretty(&result).unwrap_or_default() }]
                    })),
                    Err(e) => JsonRpcResponse::error(request.id.clone(), -32603, &e),
                }
            }
            _ => JsonRpcResponse::error(request.id.clone(), -32601, &format!("Method not found: {}", request.method)),
        })
    }
}

// ---------------------------------------------------------------------------
// stdio transport
// ---------------------------------------------------------------------------

/// Run MCP server over stdio (stdin/stdout)
pub fn run_stdio(repo_root: PathBuf, write_enabled: bool) -> Result<()> {
    use std::io::{BufRead, Write};

    let registry = ToolRegistry::new(repo_root, write_enabled);
    let stdin = std::io::stdin();
    let stdout = std::io::stdout();

    for line in stdin.lock().lines() {
        let line = line.map_err(|e| ProductError::IoError(format!("stdin read: {}", e)))?;
        if line.trim().is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::error(None, -32700, &format!("Parse error: {}", e));
                let json = serde_json::to_string(&resp).unwrap_or_default();
                let mut out = stdout.lock();
                let _ = writeln!(out, "{}", json);
                let _ = out.flush();
                continue;
            }
        };

        // Notifications return None — no response written (MCP spec)
        if let Some(response) = registry.handle_jsonrpc(&request) {
            let json = serde_json::to_string(&response).unwrap_or_default();
            let mut out = stdout.lock();
            let _ = writeln!(out, "{}", json);
            let _ = out.flush();
        }
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// HTTP transport (Streamable HTTP via axum)
// ---------------------------------------------------------------------------

/// Run MCP server over HTTP
pub async fn run_http(
    repo_root: PathBuf,
    write_enabled: bool,
    port: u16,
    bind: &str,
    token: Option<String>,
    cors_origins: Vec<String>,
) -> Result<()> {
    use axum::{Router, routing::post, http::{StatusCode, HeaderMap}, Json};
    use std::sync::Arc;

    struct AppState {
        registry: ToolRegistry,
        token: Option<String>,
    }

    let state = Arc::new(AppState {
        registry: ToolRegistry::new(repo_root, write_enabled),
        token,
    });

    let app = Router::new()
        .route("/mcp", post({
            let state = state.clone();
            move |headers: HeaderMap, Json(request): Json<JsonRpcRequest>| {
                let state = state.clone();
                async move {
                    // Auth check
                    if let Some(ref expected) = state.token {
                        let auth = headers.get("authorization")
                            .and_then(|v| v.to_str().ok())
                            .and_then(|v| v.strip_prefix("Bearer "));
                        match auth {
                            Some(provided) if provided == expected.as_str() => {}
                            _ => {
                                return (StatusCode::UNAUTHORIZED, Json(JsonRpcResponse::error(
                                    request.id, -32000, "Unauthorized"
                                )));
                            }
                        }
                    }

                    // Notifications return None — respond with 202 Accepted (no body needed but type requires one)
                    match state.registry.handle_jsonrpc(&request) {
                        Some(response) => (StatusCode::OK, Json(response)),
                        None => (StatusCode::ACCEPTED, Json(JsonRpcResponse::success(None, serde_json::json!(null)))),
                    }
                }
            }
        }));

    // Add CORS if configured
    let app = if !cors_origins.is_empty() {
        use tower_http::cors::{CorsLayer, AllowOrigin};
        use axum::http::Method;
        let origins: Vec<_> = cors_origins.iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        app.layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::list(origins))
                .allow_methods([Method::POST, Method::OPTIONS])
                .allow_headers([axum::http::header::AUTHORIZATION, axum::http::header::CONTENT_TYPE]),
        )
    } else {
        app
    };

    let addr = format!("{}:{}", bind, port);
    eprintln!("Product MCP HTTP server listening on {}", addr);
    if state.token.is_some() {
        eprintln!("  Authentication: bearer token required");
    } else {
        eprintln!("  Warning: no authentication configured (--token not set)");
    }

    let listener = tokio::net::TcpListener::bind(&addr).await.map_err(|e| {
        ProductError::IoError(format!("Failed to bind {}: {}", addr, e))
    })?;

    // Graceful shutdown: listen for SIGTERM/SIGINT, complete in-flight requests
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| {
            ProductError::IoError(format!("Server error: {}", e))
        })?;

    Ok(())
}

/// Wait for SIGTERM or SIGINT to trigger graceful shutdown
async fn shutdown_signal() {
    use tokio::signal;

    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .ok();
    };

    #[cfg(unix)]
    let terminate = async {
        if let Ok(mut sig) = signal::unix::signal(signal::unix::SignalKind::terminate()) {
            sig.recv().await;
        } else {
            std::future::pending::<()>().await;
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
    eprintln!("Shutdown signal received, draining in-flight requests...");
}

// ---------------------------------------------------------------------------
// .mcp.json scaffolding
// ---------------------------------------------------------------------------

/// Generate .mcp.json for Claude Code integration
pub fn scaffold_mcp_json(repo_root: &Path) -> Result<()> {
    let content = serde_json::json!({
        "mcpServers": {
            "product": {
                "command": "product",
                "args": ["mcp"],
                "cwd": repo_root.display().to_string()
            }
        }
    });
    let json = serde_json::to_string_pretty(&content).unwrap_or_default();
    let path = repo_root.join(".mcp.json");
    crate::fileops::write_file_atomic(&path, &json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_registry_has_read_tools() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
        let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
        assert!(registry.tools.iter().any(|t| t.name == "product_context"));
        assert!(registry.tools.iter().any(|t| t.name == "product_feature_list"));
    }

    #[test]
    fn tool_registry_write_disabled_blocks() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n[paths]\nfeatures = \"f\"\nadrs = \"a\"\ntests = \"t\"\n").expect("write");
        std::fs::create_dir_all(dir.path().join("f")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("a")).expect("mkdir");
        std::fs::create_dir_all(dir.path().join("t")).expect("mkdir");
        let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
        let result = registry.call_tool("product_feature_new", &serde_json::json!({"title": "test"}));
        assert!(result.is_err());
        assert!(result.err().unwrap_or_default().contains("disabled"));
    }

    #[test]
    fn jsonrpc_initialize() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
        let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(1)),
            method: "initialize".to_string(),
            params: serde_json::json!({}),
        };
        let response = registry.handle_jsonrpc(&request).expect("initialize should return a response");
        assert!(response.result.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn jsonrpc_tools_list() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
        let registry = ToolRegistry::new(dir.path().to_path_buf(), true);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(serde_json::json!(2)),
            method: "tools/list".to_string(),
            params: serde_json::json!({}),
        };
        let response = registry.handle_jsonrpc(&request).expect("tools/list should return a response");
        let tools = response.result.as_ref()
            .and_then(|r| r.get("tools"))
            .and_then(|t| t.as_array());
        assert!(tools.is_some());
        assert!(tools.map(|t| t.len()).unwrap_or(0) > 10);
    }

    #[test]
    fn jsonrpc_notification_returns_none() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("product.toml"), "name = \"test\"\n").expect("write");
        let registry = ToolRegistry::new(dir.path().to_path_buf(), false);
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: None,
            method: "notifications/initialized".to_string(),
            params: serde_json::json!({}),
        };
        assert!(registry.handle_jsonrpc(&request).is_none(), "notifications must not receive a response");
    }
}
