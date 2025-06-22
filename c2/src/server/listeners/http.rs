use actix_web::{web, App, HttpServer};
use tokio::sync::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use std::fs::File;
use std::io::Write;
use std::io::BufReader;

use rustls::server::ServerConfig;
use rustls::pki_types;
use rcgen::generate_simple_self_signed;


use crate::core::c2::C2;
use crate::utils::logging::{Logging, LogEntry}; 
use crate::server::communication::http_session;


pub fn generate_self_signed_cert_and_key(cert_path: &str, key_path: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let subject_alt_names = vec!["localhost".to_string()];
    let rcgen::CertifiedKey { cert, key_pair } = 
            generate_simple_self_signed(subject_alt_names).unwrap();

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    File::create(cert_path)?.write_all(cert_pem.as_bytes())?;
    File::create(key_path)?.write_all(key_pem.as_bytes())?;

    Ok(())
}

/// Loads TLS certificate and private key from disk and configures a Rustls ServerConfig.
/// 
/// Returns:
/// - Ok(ServerConfig) if successful
/// - Err(Box<dyn Error>) if certificate or key loading/parsing fails
///
/// Expects certificate at `certs/cert.pem` and key at `certs/key.pem`.
pub fn configure_tls(certificate: Option<String>, private_key: Option<String>, generate_certs: bool) 
        -> Result<ServerConfig, Box<dyn std::error::Error + Send + Sync>> {
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`

    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .unwrap();

    // ────── LOGIC FLOW ──────
    //
    // Case 1: generate_certs == true
    //   1a. If certificate and private_key are provided -> use them as paths
    //   1b. If neither cert nor key is provided -> generate new self-signed cert/key in-memory (NOT IMPLEMENTED YET)
    //   1c. If only one of cert or key is missing -> return error
    //
    // Case 2: generate_certs == false
    //   2a. If certificate and private_key are provided -> use them as paths
    //   2b. Otherwise -> return error
    //

    // Determine certificate and key paths
    let (cert_path, key_path) = match (certificate, private_key, generate_certs) {
        // Case 1a and 2a: Both cert and key are provided
        (Some(cert), Some(key), _) => (cert, key),

        // Case 1b: generate_certs is true and both paths are missing — generate new certs
        (None, None, true) => {
            let cert_path = "certs/cert.pem".to_string();
            let key_path = "certs/key.pem".to_string();
            Logging::INFO.print_message("Generating certificates ...");
            generate_self_signed_cert_and_key(&cert_path, &key_path)?;
            Logging::INFO.print_message("Certificates successfully generated ...");
            (cert_path, key_path)
        }

        // Invalid cases
        (None, Some(_), _) => return Err("Certificate path is missing".into()),
        (Some(_), None, _) => return Err("Private key path is missing".into()),
        (None, None, false) => return Err("No certificate or key provided, and generate_certs is false".into()),
    };

    // let cert_path = certificate.ok_or("Missing certificate path")?;
    // let key_path = private_key.ok_or("Missing private key path")?;
        
    let cert_file = File::open(cert_path)
        .map_err(|e| format!("Failed to open cert.pem: {}", e))?;
    let key_file = File::open(key_path)
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

    Logging::INFO.log_global("TLS configuration successfully loaded");

    Ok(tls_config)

}

/// Starts the HTTP or HTTPS Actix server with the given address and C2 state.
/// 
/// # Arguments
/// * `addr` - Address to bind the server to.
/// * `c2` - Shared C2 state.
/// * `tls` - If true, starts HTTPS; otherwise, HTTP.
/// 
/// # Returns
/// * `Ok(())` if the server starts successfully.
/// * `Err` if there is a server or TLS configuration error.
pub async fn run_server(
    addr: &str, c2: Arc<Mutex<C2>>, log_tx: Sender<LogEntry>,
    tls: bool, certificate: Option<String>, 
    private_key: Option<String>, generate_certs: bool) 
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _bind_address = "0.0.0.0:8080"; // or pull from Args if needed

    if tls {
        log_tx.send((Logging::NETWORK, format!("[*] Starting HTTPS server on {}", addr))).await.ok(); // Use `.ok()` to avoid breaking on send failure

        match configure_tls(certificate, private_key, generate_certs) {
            Ok(tls_config) => {
                HttpServer::new({
                    move || {
                        App::new()
                            .app_data(web::Data::new(log_tx.clone()))
                            .app_data(web::Data::new(c2.clone()))
                            .route("/", web::get().to(http_session::health_check))
                            .route("/beacon", web::post().to(http_session::handle_beacon))
                            .route("/heartbeat", web::post().to(http_session::handle_heartbeat))
                            .route("/disconnect", web::post().to(http_session::handle_disconnect))
                            .route("/reconnect", web::post().to(http_session::handle_reconnect))
                            .default_service(web::to(http_session::handle_catch_all))
                    }})
                    .bind_rustls_0_23(addr, tls_config)? // port 8443 or 443 default
                    .workers(4)
                    .run()
                    .await?
            }
            Err(e) => {
                Logging::ERROR.log_global(&format!("Failed to configure TLS: {}", e));
                return Err(e);
            }
        }

    } else {
        // println!("[*] Starting HTTP server on {}", addr);
        log_tx.send((Logging::NETWORK, format!("[*] Starting HTTP server on {}", addr))).await.ok(); // Use `.ok()` to avoid breaking on send failure


        HttpServer::new(move || {
            App::new()
                .app_data(web::Data::new(log_tx.clone()))
                .app_data(web::Data::new(c2.clone()))
                .route("/", web::get().to(http_session::health_check))
                .route("/beacon", web::post().to(http_session::handle_beacon))
                .route("/heartbeat", web::post().to(http_session::handle_heartbeat))
                .route("/disconnect", web::post().to(http_session::handle_disconnect))
                .route("/reconnect", web::post().to(http_session::handle_reconnect))
                .default_service(web::to(http_session::handle_catch_all))
        })
        .bind(addr)? // port 8080 or 80 default
        .workers(4) // or tune as needed
        .run()
        .await?
    }

    Ok(())
    
}
