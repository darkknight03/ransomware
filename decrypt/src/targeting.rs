use std::fs;
use std::path::Path;
use walkdir::WalkDir;
use crate::logger::Logger;

/// Discovers files to be decrypted
pub fn discover_files_decrypt(logger: &Logger, target_path: &str, extension: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    logger.log("Discovering encrypted files...");

    let path: &Path = Path::new(target_path);
    logger.log_format("Searching in path:", format_args!("{}", path.display()));

    let mut encrypted_files = Vec::new();

    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                let p = entry.path();

                // Check if path is a directory
                if let Ok(metadata) = fs::metadata(p) {
                    if metadata.is_dir() {
                        continue;
                    }
                }

                // Check for target file extensions
                let ext = p.extension().unwrap_or_default();
                if ext == extension {
                    encrypted_files.push(p.to_string_lossy().into_owned());
                }
                
            }
            Err(e) => eprintln!("Error reading entry: {}", e), // Handle WalkDir errors
        }
    }

    // Write contents of files vector to new file
    logger.log_format("Targeting complete. Target files:", format_args!("{}", encrypted_files.len()));

    Ok(encrypted_files)
}