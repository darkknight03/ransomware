use std::{fs::OpenOptions, io::Write, path::Path, fs::File};
use chrono::Local; 
use crossterm::{
    style::{Color, ResetColor, SetForegroundColor},
    ExecutableCommand,
};
use std::io::stdout;
use once_cell::sync::Lazy;
use std::sync::Mutex;

#[derive(PartialEq, Debug, Clone)]
pub enum Logging {
    INFO,
    DEBUG,
    SUCCESS,
    ERROR,
    NETWORK,
    RESULT
}

pub type LogEntry = (Logging, String);

pub static LOG_FILE: Lazy<Mutex<Option<File>>> = Lazy::new(|| Mutex::new(None));

impl Logging {
    pub fn _log(&self, log_file: impl AsRef<Path>, msg: &str) {
        let path = log_file.as_ref();
        let log_type: &str = match self {
            Self::INFO => "INFO",
            Self::DEBUG => "DEBUG",
            Self::SUCCESS => "SUCCESS",
            Self::ERROR => "ERROR",
            Self::NETWORK => "NETWORK",
            Self::RESULT => "RESULT"
        };

        let now = Local::now(); // Get the current date and time

        let formatted_message = format!(
                "[{}] [{}] {}\n",
                now.format("%Y-%m-%d %H:%M:%S"),
                log_type,
                msg
            );

        // Open file in append mode
        let mut file = match OpenOptions::new().create(true).append(true).open(path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Failed to open log file: {}", e);
                return;
            }
        };

        if let Err(e) = file.write_all(formatted_message.as_bytes()) {
            eprintln!("Failed to write to log file: {}", e);
        }

    }

    pub fn print_message(&self, msg: &str) {
        let mut stdout: std::io::Stdout = stdout();

        // Decide on print color
        let statement_color: Color = match self {
            Self::INFO => Color::Cyan,
            Self::DEBUG => Color::Magenta,
            Self::NETWORK => Color::DarkYellow,
            Self::SUCCESS => Color::Green,
            Self::ERROR => Color::Red,
            Self::RESULT => Color::Blue
        };

        // Set selected color
        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", msg);

        // Reset color
        stdout.execute(ResetColor).unwrap();
    }

    /// Logs using the global file set via `init_global_log_file()`
    pub fn log_global(&self, msg: &str) {
        let log_type = self.to_str();
        let now = Local::now();

        let formatted_message = format!(
            "[{}] [{}] {}\n",
            now.format("%Y-%m-%d %H:%M:%S"),
            log_type,
            msg
        );

        let mut guard = LOG_FILE.lock().unwrap();
        if let Some(file) = guard.as_mut() {
            let _ = file.write_all(formatted_message.as_bytes());
        } else {
            eprintln!("Global log file not initialized");
        }
    }

    fn to_str(&self) -> &'static str {
        match self {
            Self::INFO => "INFO",
            Self::DEBUG => "DEBUG",
            Self::SUCCESS => "SUCCESS",
            Self::ERROR => "ERROR",
            Self::NETWORK => "NETWORK",
            Self::RESULT => "RESULT",
        }
    }


}

/// Call once at startup to initialize the global log file
pub fn init_global_log_file(path: impl AsRef<Path>) {
    let file = match OpenOptions::new().create(true).append(true).open(path) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Failed to initialize global log file: {}", e);
            return;
        }
    };


    let mut guard = LOG_FILE.lock().unwrap();
    *guard = Some(file);
}