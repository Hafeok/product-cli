//! Interactive terminal loop hosting one Copilot SDK session.
//!
//! The previous launcher handed the terminal to the `copilot` TUI; in
//! `--server` mode the SDK is headless, so this loop owns the interaction:
//! it streams assistant output to stdout, forwards the agent's questions to
//! stdin, and sends each line the user types as the next turn.

use std::io::{BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use github_copilot_sdk::handler::{UserInputHandler, UserInputResponse};
use github_copilot_sdk::mode::ToolSet;
use github_copilot_sdk::types::{MessageOptions, SessionConfig, SessionEvent, SessionId, Tool};
use github_copilot_sdk::session::Session;
use github_copilot_sdk::{permission, Client, ClientOptions, CliProgram};

use product_core::error::{ProductError, Result};

/// Built-in CLI tools the session may use — the same read-only set the
/// previous launcher passed via `--available-tools` / `--allow-tool`.
const READ_ONLY_BUILTINS: [&str; 5] = ["read", "glob", "grep", "list", "view"];

/// Each agent turn may run many minutes of graph authoring; cap it well
/// above the SDK's 60-second default.
const TURN_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// One hosted session: the opening prompt (parity with the previous
/// `copilot -i <prompt>` launch), the bridged tool surface, the repo the
/// agent works in.
pub struct SessionSpec {
    pub cwd: PathBuf,
    pub prompt: String,
    pub tools: Vec<Tool>,
}

/// Run the session to completion on a fresh tokio runtime (the CLI adapter
/// is synchronous).
pub fn run_blocking(spec: SessionSpec) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()
        .map_err(|e| ProductError::IoError(format!("failed to create tokio runtime: {e}")))?;
    rt.block_on(run(spec))
}

async fn run(spec: SessionSpec) -> Result<()> {
    let cli_path = super::host::resolve_cli()?;
    let mut options = ClientOptions::default();
    options.program = CliProgram::Path(cli_path);
    options.working_directory = spec.cwd.clone();
    let client = Client::start(options)
        .await
        .map_err(|e| sdk_err("start the Copilot CLI server", &e))?;

    let session = client
        .create_session(session_config(spec.tools)?)
        .await
        .map_err(|e| sdk_err("create the Copilot session", &e))?;

    // Print streamed assistant output as events arrive.
    let mut events = session.subscribe();
    let printer = tokio::spawn(async move {
        while let Ok(event) = events.recv().await {
            print_event(&event);
        }
    });

    println!("Copilot session {} started — product tools run in-process (no MCP).", session.id());
    println!("Type a message; 'exit' or Ctrl-D ends the session.");
    println!();

    let mut outcome = send_turn(&session, spec.prompt).await;
    while outcome.is_ok() {
        print!("\n› ");
        std::io::stdout().flush().ok();
        let Some(line) = read_line().await else { break };
        let line = line.trim().to_string();
        if line.is_empty() {
            continue;
        }
        if line == "exit" || line == "quit" {
            break;
        }
        outcome = send_turn(&session, line).await;
    }

    printer.abort();
    let _ = session.disconnect().await;
    let _ = client.stop().await;
    outcome
}

/// The session surface: bridged product tools, streaming output, read-only
/// built-ins, stdin for the agent's questions. Config discovery stays off —
/// parity with the previous `--no-custom-instructions` +
/// `--disable-builtin-mcps` flags (no instruction files, no MCP config).
fn session_config(tools: Vec<Tool>) -> Result<SessionConfig> {
    let mut config = SessionConfig::default()
        .with_permission_handler(permission::approve_if(|data| {
            // Only read-only built-ins get through. The bridged product
            // tools set `skip_permission` and never reach this handler.
            data.extra
                .get("tool")
                .and_then(|v| v.as_str())
                .map(|t| READ_ONLY_BUILTINS.contains(&t))
                .unwrap_or(false)
        }))
        .with_user_input_handler(Arc::new(StdinUserInput))
        .with_tools(tools);
    config.streaming = Some(true);
    config.available_tools = Some(available_tools()?);
    config.enable_config_discovery = Some(false);
    Ok(config)
}

/// The session's tool filter. `available_tools` filters **every** tool
/// source, not just built-ins — a bare built-in list would hide the bridged
/// product tools too — so it must be source-qualified: the read-only
/// built-ins plus every client-registered (`custom:`) tool.
fn available_tools() -> Result<Vec<String>> {
    ToolSet::new()
        .add_builtin_many(READ_ONLY_BUILTINS)
        .and_then(|set| set.add_custom("*"))
        .map(ToolSet::into_vec)
        .map_err(|e| sdk_err("build the session tool filter", &e))
}

/// Send one user turn; block until the session goes idle. Turn-level errors
/// (timeouts, model failures) print but keep the loop alive; only a broken
/// transport ends the session.
async fn send_turn(session: &Session, text: String) -> Result<()> {
    let message = MessageOptions::new(text).with_wait_timeout(TURN_TIMEOUT);
    match session.send_and_wait(message).await {
        Ok(_) => Ok(()),
        Err(e) if e.is_transport_failure() => Err(sdk_err("reach the Copilot CLI server", &e)),
        Err(e) => {
            eprintln!("\n[turn error] {e}");
            Ok(())
        }
    }
}

fn print_event(event: &SessionEvent) {
    match event.event_type.as_str() {
        "assistant.message_delta" => {
            if let Some(text) = event.data.get("deltaContent").and_then(|c| c.as_str()) {
                print!("{text}");
                std::io::stdout().flush().ok();
            }
        }
        // Final message — terminate the streamed line.
        "assistant.message" => println!(),
        "session.error" => {
            let msg = event.data.get("message").and_then(|m| m.as_str()).unwrap_or("unknown error");
            eprintln!("\n[session error] {msg}");
        }
        _ => {}
    }
}

/// Read one line from stdin without blocking the async runtime. `None` on
/// EOF (Ctrl-D) or a read error.
async fn read_line() -> Option<String> {
    tokio::task::spawn_blocking(|| {
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line),
        }
    })
    .await
    .ok()
    .flatten()
}

/// Forward the agent's questions (`userInput.request`) to the terminal.
struct StdinUserInput;

#[async_trait]
impl UserInputHandler for StdinUserInput {
    async fn handle(
        &self,
        _sid: SessionId,
        question: String,
        choices: Option<Vec<String>>,
        _allow_freeform: Option<bool>,
    ) -> Option<UserInputResponse> {
        println!("\n[agent asks] {question}");
        if let Some(cs) = &choices {
            for (i, c) in cs.iter().enumerate() {
                println!("  {}. {c}", i + 1);
            }
        }
        print!("> ");
        std::io::stdout().flush().ok();
        let answer = read_line().await?.trim().to_string();
        // A bare number picks the matching choice.
        let answer = match (&choices, answer.parse::<usize>()) {
            (Some(cs), Ok(n)) if (1..=cs.len()).contains(&n) => cs[n - 1].clone(),
            _ => answer,
        };
        let was_freeform = choices.as_ref().map(|cs| !cs.contains(&answer)).unwrap_or(true);
        Some(UserInputResponse { answer, was_freeform })
    }
}

fn sdk_err(action: &str, e: &github_copilot_sdk::Error) -> ProductError {
    ProductError::IoError(format!("could not {action}: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_filter_admits_custom_tools_alongside_readonly_builtins() {
        // Regression: a bare built-in list here silently hid every bridged
        // product tool — the filter applies to ALL tool sources.
        let filter = available_tools().expect("filter");
        assert!(filter.contains(&"custom:*".to_string()), "{filter:?}");
        for builtin in READ_ONLY_BUILTINS {
            assert!(filter.contains(&format!("builtin:{builtin}")), "{filter:?}");
        }
        assert!(
            !filter.iter().any(|f| f.contains("bash") || f.contains("write")),
            "no mutating built-ins: {filter:?}"
        );
    }

    #[test]
    fn session_config_carries_the_bridged_tools() {
        let config = session_config(vec![Tool::new("product_workflow_status")]).expect("config");
        assert_eq!(config.streaming, Some(true));
        assert_eq!(config.enable_config_discovery, Some(false));
        let names: Vec<String> =
            config.tools.iter().flatten().map(|t| t.name.clone()).collect();
        assert_eq!(names, ["product_workflow_status"]);
        let filter = config.available_tools.as_deref().expect("filter set");
        assert!(filter.contains(&"custom:*".to_string()), "{filter:?}");
    }
}
