use std::io::{self, Write, stdout};
use std::sync::Arc;
use tokio::sync::Mutex;
use shell_words;
use crossterm::{
    event::{self, Event, KeyCode},
    style::{Color, PrintStyledContent, Stylize},
    ExecutableCommand,
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;
use std::time::Duration;

use crate::core::c2::C2;
use crate::core::cli::commands;
use crate::core::cli::app::App;
use crate::utils::logging::Logging;
use crate::core::cli::ui::render_ui;

pub struct C2Cli {
    pub current_agent: u64,
}

impl C2Cli {
    pub async fn run(&mut self, c2: Arc<Mutex<C2>>, host: &str, port: u32, protocol: &str) {

        //println!("=== Welcome to the C2 Command Interface ===\n\n");
        print_header(host, port, protocol).await;

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

    pub async fn start_c2_ui(&mut self, c2: Arc<Mutex<C2>>, host: &str, port: u32, protocol: &str) {
        // Print the grim reaper header as in original CLI
        print_header(host, port, protocol).await;
    
        // Setup terminal
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();
    
        // App state
        let mut app = App::new();
        let mut input = String::new();
        let mut current_agent: u64 = 0;
    
        loop {
            // Draw the TUI
            terminal.draw(|f| render_ui(f, &app)).unwrap();
    
            // Handle input
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key_event) = event::read().unwrap() {
                    match key_event.code {
                        KeyCode::Char(c) => {
                            input.push(c);
                        }
                        KeyCode::Backspace => {
                            input.pop();
                        }
                        KeyCode::Enter => {
                            let trimmed = input.trim();
                            if !trimmed.is_empty() {
                                app.add_output(format!("c2[{}]> {}", current_agent, trimmed));
                            }
    
                            let parsed = match shell_words::split(trimmed) {
                                Ok(p) => p,
                                Err(e) => {
                                    app.add_log(Logging::ERROR, format!("Error parsing input: {}", e));
                                    input.clear();
                                    continue;
                                }
                            };
    
                            let mut parts = parsed.into_iter();
                            let cmd = parts.next();
    
                            match cmd.as_deref() {
                                Some("list") => {
                                    commands::handle_list_command(&c2).await;
                                }
                                Some("exit") | Some("quit") => {
                                    app.add_log(Logging::INFO, "Exiting C2 CLI.".into());
                                    break;
                                }
                                Some("help") => {
                                    commands::handle_help_command().await;
                                }
                                Some("show") => {
                                    commands::handle_show_command(&c2, parts).await;
                                }
                                Some("remove") => {
                                    commands::handle_remove_command(&C2Cli { current_agent }, &c2, parts).await;
                                }
                                Some("home") => {
                                    commands::handle_home_command(&mut C2Cli { current_agent }).await;
                                }
                                Some("use") => {
                                    commands::handle_use_command(&mut C2Cli { current_agent }, &c2, parts).await;
                                }
                                Some("send") => {
                                    commands::handle_send_command(&C2Cli { current_agent }, &c2, parts).await;
                                }
                                Some("result") => {
                                    commands::handle_result_command(&C2Cli { current_agent }, &c2, parts).await;
                                }
                                Some(other) => {
                                    app.add_log(Logging::ERROR, format!("Unknown command: '{}'", other));
                                }
                                None => {}
                            }
    
                            input.clear();
                        }
                        KeyCode::Esc => {
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }
    
        // Restore terminal
        disable_raw_mode().unwrap();
        crossterm::execute!(
            terminal.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen
        )
        .unwrap();
        terminal.show_cursor().unwrap();
    }
}

pub async fn print_header(host: &str, port: u32, protocol: &str) {
    let mut stdout = stdout();

    let reaper_art = r#"
        ...
      ;::::;
    ;::::; :;
  ;:::::'   :;        ______   ______   ______
 ;:::::;     ;      /_____/\ /_____/\ /_____/\ 
,:::::'       ;     \::::_\/_\::::_\/_\:::_ \ \
::::::;       ;      \:\/___/\\:\/___/\\:(_) ) )_
;::::::;     ;        \_::._\:\\_::._\:\\: __ `\ \
:::::::::.. .           /____\:\\_____\/ \ \ `\ \ \
"#;

    let cyber_font_title = "C2 COMMAND INTERFACE";

    // Print the grim reaper art in red
    for line in reaper_art.lines() {
        stdout
            .execute(PrintStyledContent(line.red())) // Red reaper
            .unwrap();
        println!();
    }

    // Print the cyber font title in bold purple
    println!("{}", cyber_font_title.with(Color::Magenta).bold());
    println!("{}", "=====================================================".with(Color::DarkMagenta));

    // Listener info in styled format
    println!(
        "{} {}",
        "Listener:".with(Color::Red).bold(),
        format!("{}:{}", host, port).with(Color::White)
    );

    println!(
        "{} {}",
        "Protocol:".with(Color::Red).bold(),
        protocol.to_uppercase().with(Color::White)
    );

    // Footer hint
    println!("\n{}", "Type 'help' to get started.".with(Color::DarkGrey).italic());
    println!();
}