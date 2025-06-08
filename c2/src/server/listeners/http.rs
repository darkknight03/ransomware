use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use tokio::sync::Mutex;
use std::sync::Arc;
use std::fs::File;
use std::io::BufReader;

use rustls::server::ServerConfig;
use rustls::pki_types;


use crate::core::c2::C2;
use crate::utils::logging::Logging; 
use crate::communication::handler;


// GET route: health check
async fn health_check() -> impl Responder {
    Logging::INFO.print_message("Health check endpoint hit");
    HttpResponse::Ok().body("C2 Server is running")
}

pub fn configure_tls() -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // Open certificate and private key files
    let cert_file = File::open("certs/cert.pem")
        .map_err(|e| format!("Failed to open cert.pem: {}", e))?;
    let key_file = File::open("certs/key.pem")
        .map_err(|e| format!("Failed to open key.pem: {}", e))?;

    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);


    // Parse cert chain
    let certs: Vec<pki_types::CertificateDer<'static>> = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Failed to parse cert.pem: {}", e))?;

    if certs.is_empty() {
        return Err("No certificates found in cert.pem".into());
    }

    // Parse private key
    let keys: Vec<pki_types::PrivatePkcs8KeyDer<'static>> = rustls_pemfile::pkcs8_private_keys(&mut key_reader)
        .collect::<Result<_, _>>()
        .map_err(|e| format!("Failed to parse key.pem: {}", e))?;

    let key = keys
        .into_iter()
        .next()
        .ok_or_else(|| "No private key found in key.pem")?;
    
    let key = pki_types::PrivateKeyDer::Pkcs8(key);

    // set up TLS config options
    let tls_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| format!("Failed to create ServerConfig: {}", e))?;

    Ok(tls_config)

}

pub async fn run_server(addr: &str, c2: Arc<Mutex<C2>>, tls: bool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _bind_address = "0.0.0.0:8080"; // or pull from Args if needed

    if tls {
        println!("[*] Starting HTTPS server on {}", addr);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::from(c2.clone()))
                .route("/", web::get().to(health_check))
            })
            .bind_rustls_0_23(addr, configure_tls()?)?// port 8443 or 443 default
            .workers(4)
            .run()
            .await?
    } else {
        println!("[*] Starting HTTP server on {}", addr);

        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::from(c2.clone()))
                .route("/", web::get().to(health_check))
                .route("/agent", web::post().to(handler::handle_agent_message))
                // You can add more routes here as needed
        })
        .bind(addr)? // port 8080 or 80 default
        .workers(4) // or tune as needed
        .run()
        .await?
    }

    Ok(())
    
}
