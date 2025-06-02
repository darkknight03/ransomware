use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};
use pkcs1::DecodeRsaPrivateKey;
use openssl::symm::{self};
use crate::logger::Logger;


// Decrypts files
pub fn decrypt(targets: Vec<String>, logger: &Logger, key_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = get_files(&targets)?;

    // Use private key to decrypt encrypted AES key
    let private_key = load_private_key()?;
    
    let mut reader = File::open(key_path)?;
    let mut encrypted_key = Vec::new();
    reader.read_to_end(&mut encrypted_key)?;


    let aes_key = decrypt_key(encrypted_key, &private_key)?;
    logger.log("Key decrypted");
    

    for file in files {
        logger.log_format("Decrypting", format_args!("{}", file.display().to_string()));
        decrypt_file(file, &aes_key)?;
    }

    logger.log("All files decrypted");
    // Delete the encrypted key file
    // fs::remove_file("files/encrypted_key.bin")?;

    Ok(())
}

/// Get target in correct format
fn get_files(targets: &Vec<String>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for file in targets {
        let file_path = Path::new(file);
        if file_path.exists() {
            files.push(file_path.to_path_buf());
        } else {
            eprintln!("File {} does not exist", file);
        }
    }

    Ok(files)
}

/// Decrypts a file using AES
fn decrypt_file(path: PathBuf, key: &Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    // Open the input file for reading in buffered mode
    let input_file = File::open(&path)?;
    let mut reader = io::BufReader::new(input_file);

    // Read the first 16 bytes, the IV
    let mut iv = [0u8; 16];
    reader.read_exact(&mut iv)?;

    // Create a new file path without the `.enc` extension
    let decrypted_path = path.with_extension("");

    // Open the output file for writing in buffered mode
    let output_file = File::create(&decrypted_path)?;
    let mut writer = io::BufWriter::new(output_file);

    // Initialize the AES-256-CTR Crypter
    let cipher = symm::Cipher::aes_256_ctr();
    let mut crypter = symm::Crypter::new(cipher, symm::Mode::Decrypt, &key, Some(&iv))?;
    let mut buffer = [0u8; 4096]; // 4KB buffer
    let mut decrypted_buffer = vec![0u8; 4096 + cipher.block_size()]; // Extra space for decryption overhead

    // Decrypt each chunk and write to output_file
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        let count = crypter.update(&buffer[..bytes_read], &mut decrypted_buffer)?;

        writer.write_all(&decrypted_buffer[..count])?;
    }

    // Finalize decryption (not strictly necessary for CTR, but good practice)
    let count = crypter.finalize(&mut decrypted_buffer)?;
    writer.write_all(&decrypted_buffer[..count])?;

    // Delete old file
    fs::remove_file(&path)?;

    Ok(())
}

/// Decrypts AES key using private key
fn decrypt_key(encrypted_aes_key: Vec<u8>, private_key: &RsaPrivateKey) -> Result<Vec<u8>,Box<dyn std::error::Error>> {

    let decrypted_key = private_key.decrypt(Pkcs1v15Encrypt, &encrypted_aes_key)
    .map_err(|e| format!("Key Decryption Failed {}", e))?;

    Ok(decrypted_key)
}

/// Load private key from file
fn load_private_key() -> Result<RsaPrivateKey, Box<dyn std::error::Error>> {
    let path = "keys/private_key.pem";
    let mut reader = fs::File::open(path)?;
    let mut key = String::new();
    reader.read_to_string(&mut key)?;

    let rsa_key = RsaPrivateKey::from_pkcs1_pem(&key)?;

    Ok(rsa_key)
}

