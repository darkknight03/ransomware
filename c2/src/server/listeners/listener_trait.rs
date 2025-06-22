use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;
use crate::utils::logging::LogEntry;
use crate::core::c2::C2; 


#[async_trait::async_trait]
pub trait Listener: Send + Sync {
    async fn start(&self, c2: Arc<Mutex<C2>>, log_tx: Sender<LogEntry>) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}
