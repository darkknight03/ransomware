use std::io::stdout;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Receiver;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{enable_raw_mode, disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use ratatui::text::{Line, Span};
use ratatui::style::{Color, Style};


use crate::utils::logging::{Logging, LogEntry};
use crate::core::cli::{ui, keys};
use crate::core::c2::C2;
use crate::core::cli::commands;

struct TerminalGuard {
    terminal: Terminal<CrosstermBackend<std::io::Stdout>>,
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        // Called even if panic occurs
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

#[derive(Debug)]
pub enum FocusPane {
    Input,
    Output,
    Logs,
}


#[derive(Debug)]
pub struct App {
    pub input: String,
    //pub output: Vec<String>,
    pub output: Vec<Line<'static>>,
    // pub logs: Vec<(Logging, String)>, // Tuple: (log level, message)
    pub logs: Vec<LogEntry>, // Tuple: (log level, message)
    pub current_agent: u64,

    // New scroll positions
    pub log_scroll: u16,
    pub output_scroll: u16,

    // History
    pub input_history: Vec<String>,
    pub history_index: Option<usize>, // None = not navigating history

    // Channel for external log messages
    pub log_rx: Option<Receiver<LogEntry>>,

    // Which pane keybindings should work on
    pub focus: FocusPane,
}


impl App {
    pub fn new(rx: Receiver<LogEntry>) -> Self {
        Self {
            input: String::new(),
            output: vec![],
            logs: vec![],
            current_agent: 0,
            log_scroll: 0,
            output_scroll: 0,
            input_history: vec![],
            history_index: None,
            log_rx: Some(rx),
            focus: FocusPane::Input,    
        }
    }

    pub fn add_output(&mut self, line: impl Into<Line<'static>>) {
        self.output.push(line.into());
    }

    pub fn add_log(&mut self, level: Logging, message: String) {
        self.logs.push((level, message));
    }

    pub fn poll_logs(&mut self) {
        if let Some(rx) = &mut self.log_rx {
            while let Ok((level, msg)) = rx.try_recv() {
                self.logs.push((level, msg));
            }
        }
    }

    pub fn _auto_scroll(&mut self, terminal: &Terminal<CrosstermBackend<std::io::Stdout>>) {
        if let Ok(size) = terminal.size() {
            // Logs pane auto-scroll (unless focused)
            if !matches!(self.focus, FocusPane::Logs) {
                let log_height = size.height.saturating_sub(2); // adjust if header/footer exist
                let log_len = self.logs.len() as u16;
                self.log_scroll = if log_len > log_height {
                    log_len - log_height
                } else {
                    0
                };
            }
    
            // Output pane auto-scroll (unless focused)
            if !matches!(self.focus, FocusPane::Output) {
                let output_height = size.height.saturating_sub(5); // again, adjust padding if needed
                let output_len = self.output.len() as u16;
                self.output_scroll = if output_len > output_height {
                    output_len - output_height
                } else {
                    0
                };
            }
        }
    }

    pub async fn c2_cli(&mut self, c2: Arc<Mutex<C2>>, host: &str, port: u32, protocol: &str) {
        enable_raw_mode().unwrap();
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen).unwrap();
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend).unwrap();

        // This guard ensures proper cleanup even if panic occurs in the loop
        let mut terminal = TerminalGuard { terminal };

        self.input.clear(); // start with empty input each time

        // Print header ascii art
        let header = build_colored_header_output(host, port, protocol);
        self.output.extend(header);

        
        loop {
            //self.auto_scroll(&terminal);

            // Draw the TUI
            if let Err(e) = terminal.terminal.draw(|f| ui::render_ui(f, self)) {
                Logging::ERROR.log_global(&format!("UI draw error: {}", e));
                //eprintln!("UI draw error: {}", e);
                break;
            }

            // Poll for logs
            self.poll_logs();
            

            if event::poll(std::time::Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    match key.code {
                        KeyCode::Char(c) => self.input.push(c),
                        KeyCode::Backspace => { self.input.pop(); }
                        KeyCode::Enter => {
                            let trimmed = self.input.trim().to_string();
                            Logging::DEBUG.log_global(&format!("Command entered: {}", &trimmed));

                            if !trimmed.is_empty() {
                                self.add_output(format!("c2[{}]> {}", self.current_agent, trimmed));
                                self.input_history.push(trimmed.clone());
                            }

                            let parsed = match shell_words::split(&trimmed) {
                                Ok(p) => p,
                                Err(e) => {
                                    Logging::ERROR.log_global(&format!("Error parsing input: {}", e));
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
                        KeyCode::Up => { keys::up(self); }
                        KeyCode::Down => { keys::down(self); }
                        KeyCode::Tab => {
                            self.focus = match self.focus {
                                FocusPane::Input => FocusPane::Output,
                                FocusPane::Output => FocusPane::Logs,
                                FocusPane::Logs => FocusPane::Input,
                            };
                        }
                        KeyCode::Esc => break,
                        _ => {}
                    }
                }
            }
        }
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

