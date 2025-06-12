use crate::{post::commands::AgentCommand, utils::logger::Logger};




#[async_trait::async_trait]
pub trait CommChannel: Send + Sync {
    //async fn initial_beacon2(&self, encrypted_key: &[u8]) -> Result<(u32, String), String>;
    
    async fn initial_beacon(
        url: &str,
        retries: u64,
        timeout: u64,
        logger: &Logger,
        key: &Vec<u8>,
    ) -> (u64, String);

    async fn heartbeat(
        url: &str, 
        agent_id: u64, 
        session_id: &str, 
        timeout: u64, 
        result: Option<Vec<String>>,
        logger: &Logger
    ) -> Option<Vec<AgentCommand>>;

    async fn reconnect(
        url: &str,
        agent_id: u64,
        session_id: &str,
        timeout: u64,
        logger: &Logger,
    ) -> (u64, String);
}