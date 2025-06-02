use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::tasking::agent_command::AgentCommand;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub command: AgentCommand,
    pub dispatched: bool,
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    // Agent ID -> List of pending tasks
    queues: Arc<Mutex<HashMap<u64, Vec<Task>>>>,
}

impl TaskManager {
    pub fn new() -> Self {
        TaskManager { queues: Arc::new(Mutex::new(HashMap::new())) }
    }

    pub async fn add_task(&self, agent_id: u64, task: Task) {
        let mut queue = self.queues.lock().await;

        queue.entry(agent_id)
         .or_insert_with(Vec::new)
         .push(task);
    }

    pub async fn get_next_task(&self, agent_id: u64) -> Option<Task> {
        let mut queues = self.queues.lock().await;

        if let Some(tasks) = queues.get_mut(&agent_id) {
            for task in tasks.iter_mut() {
                if !task.dispatched {
                    task.dispatched = true;
                    return Some(task.clone());
                }
            }
        }
        None

        // let tasks = queue[&agent_id];
        // for task in tasks {
        //     if !task.dispatched {
        //         return Some(task)
        //     }
        // }
        // None
    }

    /// Gets all incomplete tasks from queue and returns as vector
    pub async fn get_tasks(&self, agent_id: u64) -> Option<Vec<AgentCommand>> {
        let mut queues = self.queues.lock().await;
        let mut commands: Vec<AgentCommand> = Vec::new();

        if let Some(tasks) = queues.get_mut(&agent_id) {
            for task in tasks.iter_mut() {
                if !task.dispatched {
                    commands.push(task.command.clone());
                    task.dispatched = true;
                }
            }
            if commands.is_empty() {
                return None
            }
        } else {
            return None
        }

        Some(commands)

    }

    pub async fn _has_pending_tasks(&self, agent_id: u64) -> bool {
        let queue = self.queues.lock().await;

        if let Some(tasks) = queue.get(&agent_id) {
            tasks.iter().any(|task| !task.dispatched)
        } else {
            false
        }

    }
}