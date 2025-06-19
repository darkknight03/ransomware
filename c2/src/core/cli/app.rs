use std::io::{self, Write, stdout};
use std::sync::Arc;
use tokio::sync::Mutex;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    ExecutableCommand,
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};



use crate::utils::logging::Logging;
use crate::core::cli::ui::render_ui;
use crate::core::c2::C2;
use crate::core::cli::commands;




#[derive(PartialEq, Debug, Clone)]
pub struct App {
    pub input: String,
    //pub output: Vec<String>,
    pub output: Vec<Line<'static>>,
    pub logs: Vec<(Logging, String)>, // Tuple: (log level, message)
    pub current_agent: u64,

    // New scroll positions
    pub log_scroll: u16,
    pub output_scroll: u16,

    // History
    pub input_history: Vec<String>,
    pub history_index: Option<usize>, // None = not navigating history
}

// TODO: add file logging mechanism to struct and add_output and add_log and input data

impl App {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            output: vec![],
            logs: vec![],
            current_agent: 0,
            log_scroll: 0,
            output_scroll: 0,
            input_history: vec![],
            history_index: None,
        }
    }

    pub fn add_output(&mut self, line: impl Into<Line<'static>>) {
        //self.output.push(line);
        self.output.push(line.into());
    }

    pub fn add_log(&mut self, level: Logging, message: String) {
        self.logs.push((level, message));
    }

    pub fn change_agent(&mut self, agent: u64) {
        self.current_agent = agent;
    }

    pub async fn c2_cli(&mut self, c2: Arc<Mutex<C2>>, host: &str, port: u32, protocol: &str) {
        // TODO: Need to print header/welcome art

        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend).unwrap();

        self.input.clear(); // start with empty input each time

        let header = build_colored_header_output(host, port, protocol);
        self.output.extend(header);


        loop {
            // Draw the TUI
            terminal.draw(|f| render_ui(f, self)).unwrap();

            if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char(c) => self.input.push(c),
                        KeyCode::Backspace => { self.input.pop(); }
                        KeyCode::Enter => {
                            let trimmed = self.input.trim().to_string();

                            if !trimmed.is_empty() {
                                self.add_output(format!("c2[{}]> {}", self.current_agent, trimmed));
                                self.input_history.push(trimmed.clone());
                            }

                            let parsed = match shell_words::split(&trimmed) {
                                Ok(p) => p,
                                Err(e) => {
                                    self.add_log(Logging::ERROR, format!("Error parsing input: {}", e));
                                    self.input.clear();
                                    continue;
                                }
                            };

                            let mut parts = parsed.into_iter();
                            let cmd = parts.next();

                            match cmd.as_deref() {
                                Some("list") => commands::handle_list_command(self, &c2).await,
                                Some("exit") | Some("quit") => { break; }
                                Some("help") => commands::handle_help_command(self).await,
                                // Some("show") => commands::handle_show_command(&c2, parts).await,
                                Some("remove") => commands::handle_remove_command(self, &c2, parts).await,
                                Some("home") => commands::handle_home_command(self).await,
                                Some("use") => commands::handle_use_command(self, &c2, parts).await,
                                Some("send") => commands::handle_send_command(self, &c2, parts).await,
                                Some("result") => commands::handle_result_command(self, &c2, parts).await,
                                Some(other) => {
                                    self.add_output(format!("Unknown command: '{}'", other));
                                    //self.add_log(Logging::ERROR, format!("Unknown command: '{}'", other));
                                }
                                None => {}
                            }

                            self.history_index = None;
                            self.input.clear();
                        }
                        KeyCode::Up => {
                            if self.input_history.is_empty() {
                                continue;
                            }
                        
                            let max = self.input_history.len() - 1;
                            self.history_index = Some(match self.history_index {
                                Some(0) | None => max,
                                Some(i) => i.saturating_sub(1),
                            });
                        
                            if let Some(i) = self.history_index {
                                self.input = self.input_history[i].clone();
                            }
                        },
                        KeyCode::Down => {
                            if self.input_history.is_empty() {
                                continue;
                            }
                        
                            self.history_index = match self.history_index {
                                None => None,
                                Some(i) if i >= self.input_history.len() - 1 => None,
                                Some(i) => Some(i + 1),
                            };
                        
                            self.input = match self.history_index {
                                Some(i) => self.input_history[i].clone(),
                                None => String::new(),
                            };
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }

        disable_raw_mode().unwrap();
        execute!(terminal.backend_mut(), LeaveAlternateScreen).unwrap();
        terminal.show_cursor().unwrap();
    }
}




pub fn build_colored_header_output(host: &str, port: u32, protocol: &str) -> Vec<Line<'static>> {
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

    let mut lines = Vec::new();

    // Red Grim Reaper ASCII art
    for line in reaper_art.lines() {
        lines.push(Line::styled(line.to_string(), Style::default().fg(Color::Red)));
    }

    // Purple bold title
    lines.push(Line::styled(
        "C2 COMMAND INTERFACE",
        Style::default().fg(Color::Magenta).add_modifier(ratatui::style::Modifier::BOLD),
    ));

    // Dark purple separator
    lines.push(Line::styled(
        "=====================================================",
        Style::default().fg(Color::Magenta),
    ));

    // Listener
    lines.push(Line::from(vec![
        Span::styled("Listener: ", Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::styled(format!("{}:{}", host, port), Style::default().fg(Color::White)),
    ]));

    // Protocol
    lines.push(Line::from(vec![
        Span::styled("Protocol: ", Style::default().fg(Color::Red).add_modifier(ratatui::style::Modifier::BOLD)),
        Span::styled(protocol.to_uppercase(), Style::default().fg(Color::White)),
    ]));

    // Footer
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Type 'help' to get started.",
        Style::default().fg(Color::DarkGray).add_modifier(ratatui::style::Modifier::ITALIC),
    ));

    lines
}

