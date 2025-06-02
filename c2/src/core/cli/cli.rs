use std::io::{self, Write};
use std::sync::Arc;
use tokio::sync::Mutex;
use shell_words;

use crate::core::c2::C2;
use crate::core::cli::commands;

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
                    commands::handle_list_command(&c2).await;
                    // let c2 = c2.lock().await;
                    // c2.list_agents().await;
                    // TODO: fix formatting to be in column format
                }
                Some("exit") | Some("quit") => {
                    println!("Exiting C2 CLI.");
                    break;
                }
                Some("help") => {
                    commands::handle_help_command().await;
                    println!("Available: list, exit, show, remove, use, home, send");
                }
                Some("show") => {
                    commands::handle_show_command(&c2, parts).await;
                    // match parts.next() {
                    //     Some(id_str) => match id_str.parse::<u64>() {
                    //         Ok(id) => {
                    //             let c2 = c2.lock().await;
                    //             if c2.agent_exists(id).await {
                    //                 c2.list_agent(id).await;
                    //             } else {
                    //                 println!("Invalid agent ID: {}", id);
                    //             }
                    //         }
                    //         Err(_) => println!("Invalid agent ID: {}", id_str),
                    //     },
                    //     None => println!("Usage: show <agent_id>"),
                    // }
                }
                Some("remove") => {
                    commands::handle_remove_command(&self, &c2, parts).await;
                    // match parts.next() {
                    //     Some(id_str) => match id_str.parse::<u64>() {
                    //         Ok(id) => {
                    //             if self.current_agent != id {
                    //                 let mut c2 = c2.lock().await;
                    //                 if c2.agent_exists(id).await {
                    //                     c2.remove_agent(id).await;
                    //                 } else {println!("Invalid agent ID: {}", id);}
                    //             } else {println!("You cannot remove the currently selected agent.");}
                    //         }
                    //         Err(_) => println!("Invalid agent ID: {}", id_str)
                    //     }
                    //     None => println!("Usage: remove <agent_id>")
                    // }
                }
                Some("home") => {
                    commands::handle_home_command(self).await;
                    // self.current_agent = 0;
                }
                Some("use") => {
                    commands::handle_use_command(self, &c2, parts).await;
                    // match parts.next() {
                    //     Some(id_str) => match id_str.parse::<u64>() {
                    //         Ok(id) => {
                    //             let c2 = c2.lock().await;
                    //             if c2.agent_exists(id).await {
                    //                 self.current_agent = id;
                    //             } else {println!("Invalid agent ID: {}", id);}
                    //         }
                    //         Err(_) => println!("Invalid agent ID: {}", id_str),
                    //     },
                    //     None => println!("Usage: use <agent_id>"),
                    // }
                }
                Some("send") => {
                    commands::handle_send_command(&self, &c2, parts).await;
                    // if self.current_agent == 0 {
                    //     println!("Select an Agent to use");
                    //     continue;
                    // }
                    // match parts.next().as_deref() {
                    //     Some("command") => {
                    //         if parts.clone().count() == 0 {
                    //             println!("Usage: send command <shell command>");
                    //             continue;
                    //         }
                    //         let task = parts.collect::<Vec<String>>().join(" ");
                    //         dbg!(&task);
                    //         let c2 = c2.lock().await;
                    //         c2.create_task(self.current_agent, 
                    //             Task {
                    //                 command: AgentCommand::RunShell(task),
                    //                 dispatched: false
                    //             }
                    //         ).await;
                            
                    //     }

                    //     Some(other) => {
                    //         println!("Unknown send subcommand: '{}'", other);
                    //     }
                    //     None => {
                    //         println!("Usage: send command <command>");
                    //     }
                    // }
                }
                
                Some("result") => {
                    commands::handle_result_command(&self, &c2, parts).await;
                }

                Some(other) => {println!("Unknown command: '{}'", other);}
                None => {}
            }
        }
    }
}