use serde::{Deserialize, Serialize};



#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentCommand {
    // System Interactions
    RunShell(String),
    // Add more later
}