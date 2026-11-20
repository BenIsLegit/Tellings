use serde::Serialize;

/// Errors cross to the window as plain sentences.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Engine(#[from] pvc_engine::Error),
    #[error("File problem: {0}")]
    Io(#[from] std::io::Error),
    #[error("Could not read that: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

/// Run engine work off the window's thread, so the app never freezes.
pub async fn off_thread<T: Send + 'static>(work: impl FnOnce() -> AppResult<T> + Send + 'static) -> AppResult<T> {
    tauri::async_runtime::spawn_blocking(work).await.map_err(|e| AppError::Other(format!("The work stopped unexpectedly: {e}")))?
}
