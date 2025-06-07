use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;


pub type ResultQueue = Arc<Mutex<Option<Vec<String>>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommand {
    // System Interactions
    RunShell(String),
    ChangeDirectory(String),
    ListDirectory(String),
    TakeScreenshot,
    GetSystemInfo,
    GetProcessList,

    // File operations
    DownloadFile { path: String },
    UploadFile { path: String, data: Vec<u8> },
    DeleteFile(String),
    SearchFiles { pattern: String },

    // Behavior
    SelfDestruct,
    InvalidTask
}