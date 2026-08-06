//! Error vocabulary for the LSP client stack.

use std::fmt;

/// Everything that can go wrong between a tool call and a host answer.
#[derive(Debug)]
pub enum LspError {
    /// The host process could not be spawned (missing tool, bad command).
    Spawn(String),
    /// A transport-level read/write failure (host died mid-conversation).
    Io(String),
    /// The host did not answer within the request deadline.
    Timeout(String),
    /// The host answered with a JSON-RPC error.
    Rpc { code: i64, message: String },
    /// The answer did not have the shape the protocol promises.
    Protocol(String),
    /// No adapter claims the file (unknown extension, no override).
    NoAdapter(String),
}

impl fmt::Display for LspError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn(m) => write!(f, "language server spawn failed: {m}"),
            Self::Io(m) => write!(f, "language server transport: {m}"),
            Self::Timeout(m) => write!(f, "language server timeout: {m}"),
            Self::Rpc { code, message } => write!(f, "language server error {code}: {message}"),
            Self::Protocol(m) => write!(f, "language server protocol: {m}"),
            Self::NoAdapter(m) => write!(f, "no language adapter: {m}"),
        }
    }
}

impl std::error::Error for LspError {}

impl From<std::io::Error> for LspError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, LspError>;
