use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Mutex;
use chrono::Local; 

pub struct Logger {
    log_file: Mutex<Option<std::fs::File>>,
}

impl Logger {
    pub fn new() -> Self {
        Logger {
            log_file: Mutex::new(None),
        }
    }

    pub fn init_file_logging(&self, file_path: &str) -> std::io::Result<()> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)?;
        *self.log_file.lock().unwrap() = Some(file);
        Ok(())
    }

    pub fn log_to_stdout(&self, msg_type: &str, fmt: std::fmt::Arguments) {
        println!("[{}] {}", msg_type, fmt);
    }


    pub fn log_to_file(&self, msg_type: &str, message: &str) -> std::io::Result<()> {
        if let Some(ref mut file) = *self.log_file.lock().unwrap() {
            let now = Local::now(); // Get the current date and time
            
            let formatted_message = format!(
                "[{}] [{}] {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                msg_type,
                message
            );
            writeln!(file, "{}", formatted_message)?;
        }
        Ok(())
    }

    pub fn log_to_file_format(&self, msg_type: &str, message: &str, args: std::fmt::Arguments) -> std::io::Result<()> {
        if let Some(ref mut file) = *self.log_file.lock().unwrap() {
            let now = Local::now(); // Get the current date and time

            let formatted_message = format!(
                "[{}] [{}] {}",
                now.format("%Y-%m-%d %H:%M:%S"),
                msg_type,
                message
            );
            writeln!(file, "{} {}", formatted_message, args)?;
        }
        Ok(())
    }

    // Logs information message to both stdout and file
    pub fn log(&self, message: &str) {
        self.log_to_stdout("INFO", format_args!("{}", message));
        if let Err(e) = self.log_to_file("INFO", message) {
            eprint!("Failed to log to file: {}", e);
        }
    }

    pub fn log_format(&self, message: &str, args: std::fmt::Arguments) {
        self.log_to_stdout("INFO", format_args!("{} {}", message, args));
        if let Err(e) = self.log_to_file_format("INFO", message, args) {
            eprint!("Failed to log to file: {}", e);
        }
    }
}