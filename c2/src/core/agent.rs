use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::utils::logging::Logging;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Agent {
    pub id: u64,
    pub status: AgentState,
    pub ip: String,
    pub last_seen: DateTime<Local>,
    pub time_compromised: DateTime<Local>,
    pub hostname: String,
    pub os: String,
    pub session_id: String,
    pub results: Vec<AgentResult>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum AgentState {
    Alive,
    Dead,
    Error
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentResult {
    CommandOutput {
        command: String,
        output: Option<String>,
        read: bool,
        timestamp: Option<DateTime<Utc>>
    },
}


impl Agent {
    pub fn show(&self) {
        let agent_str = format!(
            "ID: {}\nIP: {}\nStatus: {:?}\nLast Seen: {}\nTime Compromised: {}\nSession ID: {}\n",
            self.id, self.ip, self.status, self.last_seen, self.time_compromised, self.session_id);

            Logging::INFO.print_message(&agent_str);    
    }

    pub fn update_time(&mut self) {
        self.last_seen = Local::now();
    }

    pub fn update_status(&mut self, status: AgentState) {
        self.status = status
    }

    pub fn update_results(&mut self, res: Vec<AgentResult>) {
        self.results = res;
    }

    pub fn unread_results(&self) -> Vec<&AgentResult> {
        self.results.iter().filter(|r| match r {
            AgentResult::CommandOutput { read, .. } => !read,
        }).collect()
    }

    pub fn get_all_results(&self) -> Vec<&AgentResult> {
        self.results.iter().collect()
    }

    pub fn clear_results(&mut self) {
        self.results.clear();
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tests_agents() {
        let mut agent_1 = Agent {
            id: 1,
            status: AgentState::Alive,
            ip: "192.168.1.1".to_string(),
            last_seen: Local::now(),
            time_compromised: Local::now(),
            hostname: "PC5".to_string(),
            os: "Windows 11".to_string(),
            session_id: "abcd".to_string(),
            results: vec![]
        };
        agent_1.show();
        agent_1.update_status(AgentState::Dead);
        agent_1.update_time();
        agent_1.show();

    }
}