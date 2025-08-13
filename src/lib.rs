pub mod clipboard;
pub mod file_manager;
pub mod mcp;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClaudeUtilsError {
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("File operation error: {0}")]
    FileOperation(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    ImageProcessing(#[from] image::ImageError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("MCP protocol error: {0}")]
    McpProtocol(String),

    #[error("Server error: {0}")]
    Server(String),
}

pub type Result<T> = std::result::Result<T, ClaudeUtilsError>;

pub const DEFAULT_PORT: u16 = 3830;
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const STAGING_DIR_NAME: &str = "claude-utils";
pub const MAX_INLINE_SIZE: usize = 65536; // 64KB
pub const CLEANUP_INTERVAL_MINS: u64 = 15;

#[cfg(test)]
mod main_test;
