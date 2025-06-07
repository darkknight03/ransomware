use serde::{Serialize, Deserialize};

use crate::post::commands::AgentCommand;


#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
pub enum AgentMessage {
    Beacon {
        hostname: String,
        ip: String,
        os: String,
        time_compromised: String,
    },
    Heartbeat {
        agent_id: u64,
        session_id: String,
        result: Option<Vec<String>>
    },
    Disconnect {
        agent_id: u64,
        session_id: String,
    },
    Reconnect {
        agent_id: u64,
        session_id: String,
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum ServerMessage {
    Ack {
        agent_id: u64,
        session_id: String,
        status: String, // e.g. "registered", "ok"
    },
    Task {
        agent_id: u64,
        session_id: String,
        command: Vec<AgentCommand>,
    },
    Noop {
        agent_id: u64,
        session_id: String,
    },
    Error {
        agent_id: u64,
        session_id: String,
        message: String,
    },
    Disconnect {
        agent_id: u64,
        session_id: String,
    }
}

