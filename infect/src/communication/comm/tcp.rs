
// use std::sync::Arc;

// use crate::communication::beacon_tcp; 
// use crate::communication::comm::channel::CommChannel;
// use crate::utils::logger::Logger;




// pub struct TcpCommChannel {
//     pub address: String,
//     pub logger: Arc<Logger>,
//     // optionally include config, retry settings, etc.
// }

// #[async_trait::async_trait]
// impl CommChannel for TcpCommChannel {
//     async fn initial_beacon(&self, encrypted_key: &[u8]) -> Result<(u32, String), String> {
//         beacon_tcp::initial_beacon(&self.address, encrypted_key, &self.logger).await
//     }

//     async fn send_heartbeat(&self, agent_id: u32, session_id: &str, results: Option<String>) -> Result<Option<AgentCommand>, String> {
//         beacon_tcp::send_heartbeat(&self.address, agent_id, session_id, results, &self.logger).await
//     }
// }
