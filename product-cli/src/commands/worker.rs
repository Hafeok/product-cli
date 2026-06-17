//! Worker catalog + role resolution + dispatch (capability model, Layers 1+2).
//!
//! `product worker {list,resolve,check,init}` manages the capability catalog
//! (`.product/capabilities.yaml` + `role-bindings.yaml`). `dispatch` is the
//! runner used by `build`: a `claude` subprocess, or the `litellm` proxy (any
//! provider behind a capability tag) — so workers are no longer just claude.

use clap::Subcommand;
use product_core::pf::capability::{validate_catalog, Capability, Catalog};
use product_core::pf::worker as fpw;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::BoxResult;

/// What a dispatch wrote — the paths a downstream gate (LSP, verify) can inspect.
type DispatchResult = Result<Vec<PathBuf>, Box<dyn std::error::Error>>;

/// Per-request timeout for a model call (clippy over a cold workspace is slow,
/// but a hung connection should not stall a build forever).
const HTTP_TIMEOUT: Duration = Duration::from_secs(240);
/// How many times to attempt a model call before giving up.
const HTTP_ATTEMPTS: u32 = 3;

/// POST a chat-completion body with a timeout, retrying transient failures
/// (transport errors, 429, 5xx) with linear backoff. 4xx fails fast.
fn post_json_retry(url: &str, key: &str, body: &serde_json::Value) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let mut last = String::new();
    for attempt in 1..=HTTP_ATTEMPTS {
        let req = ureq::post(url).timeout(HTTP_TIMEOUT).set("Authorization", &format!("Bearer {key}"));
        match req.send_json(body.clone()) {
            Ok(resp) => return resp.into_json().map_err(|e| format!("response not JSON: {e}").into()),
            Err(ureq::Error::Status(code, resp)) if code != 429 && code < 500 => {
                return Err(format!("HTTP {code}: {}", resp.into_string().unwrap_or_default()).into());
            }
            Err(ureq::Error::Status(code, resp)) => last = format!("HTTP {code}: {}", resp.into_string().unwrap_or_default()),
            Err(ureq::Error::Transport(t)) => last = format!("transport: {t}"),
        }
        if attempt < HTTP_ATTEMPTS {
            eprintln!("  model call attempt {attempt}/{HTTP_ATTEMPTS} failed ({last}); retrying…");
            std::thread::sleep(Duration::from_secs(2 * attempt as u64));
        }
    }
    Err(format!("model call failed after {HTTP_ATTEMPTS} attempts: {last}").into())
}

#[derive(Subcommand)]
pub enum WorkerCommands {
    /// Validate the capability catalog (bindings resolve; triggers known)
    Check {},
    /// Scaffold seed capability + role-binding catalogs
    Init {
        #[arg(long)]
        force: bool,
    },
    /// List the capabilities + role bindings
    List {},
    /// Resolve a role to its capability, applying escalation triggers
    Resolve {
        /// The role id (e.g. implementer)
        role: String,
        #[arg(long = "trigger")]
        triggers: Vec<String>,
    },
    /// Dispatch a prompt to a role's worker (first-party worker writes artifacts)
    Run {
        /// The role id to resolve
        role: String,
        /// The frozen SPMC context / prompt to hand the worker
        #[arg(long)]
        prompt: String,
        #[arg(long = "trigger")]
        triggers: Vec<String>,
    },
}

pub(crate) fn handle_worker(cmd: WorkerCommands) -> BoxResult {
    match cmd {
        WorkerCommands::Check {} => check(),
        WorkerCommands::Init { force } => init(force),
        WorkerCommands::List {} => list(),
        WorkerCommands::Resolve { role, triggers } => resolve_cmd(&role, &triggers),
        WorkerCommands::Run { role, prompt, triggers } => run_cmd(&role, &prompt, &triggers),
    }
}

fn run_cmd(role: &str, prompt: &str, triggers: &[String]) -> BoxResult {
    let cap = resolve(&load_catalog(), role, triggers);
    println!("role '{role}' → capability '{}' (endpoint {})", cap.id, cap.endpoint);
    dispatch(&cap, prompt)?;
    Ok(())
}

fn pdir() -> PathBuf {
    super::shared::domain_root().join(".product")
}

/// The built-in fallback capability so `build` works without a catalog.
fn default_claude() -> Capability {
    Capability {
        id: "claude-code".to_string(),
        endpoint: "claude".to_string(),
        model_identifier: "claude-opus-4-8".to_string(),
        tier: 2,
        status: "active".to_string(),
    }
}

/// Load the catalog, falling back to the built-in claude capability.
pub(super) fn load_catalog() -> Catalog {
    let caps = std::fs::read_to_string(pdir().join("capabilities.yaml"))
        .ok()
        .and_then(|t| Catalog::capabilities_from_yaml(&t).ok())
        .unwrap_or_default();
    let bindings = std::fs::read_to_string(pdir().join("role-bindings.yaml"))
        .ok()
        .and_then(|t| Catalog::role_bindings_from_yaml(&t).ok())
        .unwrap_or_default();
    let mut catalog = Catalog { capabilities: caps, role_bindings: bindings };
    if catalog.capabilities.is_empty() {
        catalog.capabilities.push(default_claude());
    }
    catalog
}

/// Resolve a role to a capability, falling back to the built-in claude default.
pub(super) fn resolve(catalog: &Catalog, role: &str, triggers: &[String]) -> Capability {
    catalog.resolve(role, triggers).cloned().unwrap_or_else(default_claude)
}

/// A role's capability ladder (weakest first) for the fix loop to climb. Falls
/// back to the single resolved default when the role has no ladder.
pub(super) fn ladder(catalog: &Catalog, role: &str) -> Vec<Capability> {
    let rungs = catalog.ladder(role);
    if rungs.is_empty() {
        vec![resolve(catalog, role, &[])]
    } else {
        rungs
    }
}

/// Dispatch a prompt to a capability's runner; returns the files it wrote (empty
/// for endpoints that stream to stdout rather than emit artifacts).
pub(super) fn dispatch(cap: &Capability, prompt: &str) -> DispatchResult {
    match cap.endpoint.as_str() {
        "claude" => run_claude(prompt).map(|_| Vec::new()),
        "litellm" | "anthropic" | "scaleway" => run_litellm(cap, prompt).map(|_| Vec::new()),
        "worker" => run_first_party(cap, prompt),
        other => Err(format!("unknown capability endpoint '{other}' (expected claude | litellm | worker)").into()),
    }
}

/// The first-party worker: ask the model for structured file output (or, with no
/// model configured, write a deterministic stub), then apply the files. When
/// `PRODUCT_MOCK_DIR` is set, consumes scripted responses instead of calling a
/// model — so the fix loops are testable in CI without a live model.
fn run_first_party(cap: &Capability, prompt: &str) -> DispatchResult {
    let root = super::shared::domain_root();
    let (files, edits) = worker_output(cap, prompt)?;
    let mut written = fpw::apply_files(&files, &root)?;
    written.extend(fpw::apply_edits(&edits, &root)?);
    println!("  first-party worker wrote {} file(s):", written.len());
    for w in &written {
        println!("    {}", w.display());
    }
    Ok(written)
}

/// Resolve the worker's structured output: scripted (mock) → live model → stub.
fn worker_output(cap: &Capability, prompt: &str) -> Result<(Vec<fpw::ArtifactFile>, Vec<fpw::EditOp>), Box<dyn std::error::Error>> {
    if let Some(dir) = std::env::var("PRODUCT_MOCK_DIR").ok().filter(|s| !s.is_empty()) {
        let obj = fpw::extract_json(&next_scripted_response(&dir)?)?;
        return Ok(fpw::parse_output(&obj)?);
    }
    let base = std::env::var("LITELLM_BASE_URL").ok().filter(|s| !s.is_empty());
    let key = std::env::var("LITELLM_API_KEY").ok().filter(|s| !s.is_empty());
    match (base, key) {
        (Some(base), Some(key)) => {
            let model = if cap.model_identifier.is_empty() { cap.id.as_str() } else { cap.model_identifier.as_str() };
            let url = format!("{}/chat/completions", base.trim_end_matches('/'));
            let v = post_json_retry(&url, &key, &fpw::build_request(model, prompt))?;
            let content = v["choices"][0]["message"]["content"].as_str().unwrap_or("");
            Ok(fpw::parse_output(&fpw::extract_json(content)?)?)
        }
        _ => {
            println!("  (no LITELLM_BASE_URL — first-party worker running offline; writing a stub)");
            Ok((fpw::stub_files(prompt), Vec::new()))
        }
    }
}

/// The next scripted response from `PRODUCT_MOCK_DIR` (`response-<n>.json`),
/// advancing a per-process counter; the last file repeats once scripts run out.
fn next_scripted_response(dir: &str) -> Result<String, Box<dyn std::error::Error>> {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static CALL: AtomicUsize = AtomicUsize::new(0);
    let n = CALL.fetch_add(1, Ordering::SeqCst);
    let base = Path::new(dir);
    let mut path = base.join(format!("response-{n}.json"));
    if !path.exists() {
        // Repeat the highest-numbered scripted response once the list is exhausted.
        let last = (0..n).rev().map(|i| base.join(format!("response-{i}.json"))).find(|p| p.exists());
        path = last.ok_or_else(|| format!("no scripted responses in {dir}"))?;
    }
    Ok(std::fs::read_to_string(&path)?)
}

fn run_claude(prompt: &str) -> BoxResult {
    let status = Command::new("claude")
        .arg("-p")
        .arg(prompt)
        .status()
        .map_err(|e| format!("failed to launch `claude`: {e}"))?;
    if !status.success() {
        return Err(format!("claude exited with {status}").into());
    }
    Ok(())
}

/// Route a completion through the LiteLLM proxy. `LITELLM_BASE_URL` must be the
/// *proxy* (which holds provider keys and maps the capability tag to a provider
/// model); `cap.id` is the proxy `model_name`. Scaleway/Anthropic capabilities
/// reach their providers via the proxy's model groups — there is no direct
/// provider call here.
fn run_litellm(cap: &Capability, prompt: &str) -> BoxResult {
    let base = std::env::var("LITELLM_BASE_URL").map_err(|_| "LITELLM_BASE_URL is not set (point it at the LiteLLM proxy)")?;
    let key = std::env::var("LITELLM_API_KEY").map_err(|_| "LITELLM_API_KEY is not set")?;
    let url = format!("{}/chat/completions", base.trim_end_matches('/'));
    let body = serde_json::json!({ "model": cap.id, "messages": [{ "role": "user", "content": prompt }] });
    let v = post_json_retry(&url, &key, &body)?;
    println!("{}", v["choices"][0]["message"]["content"].as_str().unwrap_or(""));
    Ok(())
}

fn list() -> BoxResult {
    let c = load_catalog();
    println!("Capabilities:");
    for cap in &c.capabilities {
        println!("  - {} [{}] {} (tier {}, {})", cap.id, cap.endpoint, cap.model_identifier, cap.tier, cap.status);
    }
    println!("Role bindings:");
    for b in &c.role_bindings {
        let esc: Vec<&str> = b.escalation_steps.iter().map(|s| s.capability.as_str()).collect();
        let ladder = if esc.is_empty() { String::new() } else { format!(" ⇡ {}", esc.join(" → ")) };
        println!("  - {} → {}{ladder}", b.role_id, b.default_capability);
    }
    Ok(())
}

fn resolve_cmd(role: &str, triggers: &[String]) -> BoxResult {
    let cap = resolve(&load_catalog(), role, triggers);
    println!("role '{role}' → capability '{}' (endpoint {}, model {}, tier {})", cap.id, cap.endpoint, cap.model_identifier, cap.tier);
    Ok(())
}

fn check() -> BoxResult {
    let problems = validate_catalog(&load_catalog());
    if problems.is_empty() {
        println!("catalog ok");
        return Ok(());
    }
    for p in &problems {
        eprintln!("  - [{}] {}: {}", p.focus, p.path, p.message);
    }
    Err(format!("{} catalog problem(s)", problems.len()).into())
}

fn init(force: bool) -> BoxResult {
    let dir = pdir();
    std::fs::create_dir_all(&dir)?;
    write_seed(&dir.join("capabilities.yaml"), CAPABILITIES_SEED, force)?;
    write_seed(&dir.join("role-bindings.yaml"), ROLE_BINDINGS_SEED, force)?;
    println!("Scaffolded worker catalog → .product/capabilities.yaml, .product/role-bindings.yaml");
    Ok(())
}

fn write_seed(path: &Path, content: &str, force: bool) -> BoxResult {
    if path.exists() && !force {
        return Err(format!("{} already exists — pass --force to overwrite", path.display()).into());
    }
    std::fs::write(path, content)?;
    Ok(())
}

const CAPABILITIES_SEED: &str = "# Worker capability catalog (the SPMC Model layer).\n# litellm/scaleway/anthropic capabilities route through the LiteLLM proxy at\n# LITELLM_BASE_URL (the proxy holds provider keys + maps the id to a provider\n# model). With LITELLM_BASE_URL pointed straight at Scaleway's OpenAI endpoint,\n# the model_identifier (worker) / id (litellm) must be a real Scaleway model.\ncapabilities:\n- id: claude-code\n  endpoint: claude\n  model_identifier: claude-opus-4-8\n  tier: 2\n- id: code-writer\n  endpoint: worker\n  model_identifier: devstral-2-123b-instruct-2512\n  tier: 2\n- id: code-writer-heavy\n  endpoint: worker\n  model_identifier: qwen3.5-397b-a17b\n  tier: 3\n- id: fast-cheap\n  endpoint: litellm\n  model_identifier: qwen3.6-35b-a3b\n  tier: 1\n- id: deep-reasoning\n  endpoint: litellm\n  model_identifier: qwen3.5-397b-a17b\n  tier: 3\n";

const ROLE_BINDINGS_SEED: &str = "# Role → capability bindings with escalation ladders.\nrole_bindings:\n- role_id: implementer\n  default_capability: claude-code\n  escalation_steps:\n  - capability: deep-reasoning\n    triggers:\n    - prior_attempts_ge_5\n    - stakes_foundational\n  active: true\n- role_id: coder\n  default_capability: code-writer\n  escalation_steps:\n  - capability: code-writer-heavy\n    triggers:\n    - prior_attempts_ge_5\n    - stakes_foundational\n  active: true\n- role_id: verifier\n  default_capability: fast-cheap\n  escalation_steps:\n  - capability: deep-reasoning\n    triggers:\n    - confidence_below_0.5\n  active: true\n";
