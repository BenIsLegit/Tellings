//! One error type for the whole engine. Every message is written for a person
//! who does not write code: say what went wrong and what to do about it.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Something in a setup file is wrong. The message names the setting.
    #[error("{0}")]
    Setup(String),
    /// A program this tool needs (FFmpeg, Chrome or Edge) is missing or failed.
    #[error("{0}")]
    Tool(String),
    /// ElevenLabs said no, or could not be reached.
    #[error("{0}")]
    Voice(String),
    /// Making this video would spend credits and nobody has said yes yet.
    #[error("{0}")]
    NeedsApproval(String),
    /// The person pressed Stop.
    #[error("Stopped.")]
    Cancelled,
    #[error("File problem: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not read that as a setup: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Network problem: {0}")]
    Network(#[from] reqwest::Error),
    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    pub fn setup(msg: impl Into<String>) -> Self {
        Error::Setup(msg.into())
    }
    pub fn tool(msg: impl Into<String>) -> Self {
        Error::Tool(msg.into())
    }
    pub fn voice(msg: impl Into<String>) -> Self {
        Error::Voice(msg.into())
    }
    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}

impl Serialize for Error {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
