use std::sync::Arc;
use tokio::sync::Mutex;


use actix_web::{HttpResponse, Responder, web};
use crate::{communication::message::{AgentMessage, ServerMessage}, core::c2::C2};

pub async fn handle_agent_message(msg: web::Json<AgentMessage>, c2: web::Data<Arc<Mutex<C2>>>) -> impl Responder {
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
