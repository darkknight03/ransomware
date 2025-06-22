mod core;
mod server;
mod utils;
mod tasking;

use std::sync::Arc;
use server::listeners::listener_trait::Listener;
use tokio::sync::Mutex;
use tokio::sync::mpsc;
use clap::Parser;
use tokio::task::LocalSet;

use crate::utils::logging::{Logging, LogEntry};
use crate::core::c2::C2; 
use crate::server::listeners::tcp::TCPCommListener;
use crate::server::listeners::http;
use crate::core::cli::cli::C2Cli;
use crate::core::cli::app::App;

/// Command and Control Server Configuration
#[derive(Parser, Debug)]
#[command(version, about = "C2 server")]
struct Args {
    /// Host to bind the server to
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to run the server on
    #[arg(short = 'P', long, default_value_t = 6969)]
    port: u32,

    /// Log file path
    #[arg(short = 'f', long, default_value = "server.log")]
    log_file: String,

    /// Type of listener to use: tcp, http, https, dns, multi
    #[arg(short, long, default_value = "tcp")]
    protocol: String,

    /// How often (sec) C2 should check if agents alive, default is 10 min
    #[arg(short, long, default_value_t = 600)]
    sweep: u64,

    /// Timeout (sec) for agent, default is 10 min
    #[arg(short, long, default_value_t = 600)]
    timeout: u64,

    /// Path to SSL certificate (only used if protocol is https)
    #[arg(short, long)]
    cert: Option<String>,

    /// Path to SSL private key (only used if protocol is https)
    #[arg(short, long)]
    key: Option<String>,

    /// Flag to generate new certificate/private key
    #[arg(long)]
    generate_certs: bool
}

#[tokio::main]
async fn main() {
    let args = handle_arguments().await;

    Logging::INFO.print_message("Starting up C2 server...");

    match args.protocol.as_str() {
        "tcp" => {tcp_server(args).await}
        "http" | "https" => {
            let local = LocalSet::new();
            local.run_until(async move {
                if args.protocol == "http" {
                    http_server(args).await;
                } else {
                    https_server(args).await;
                }
            }).await;
        }
        "dns" => {dns_server().await}
        "multi" => {multi_server().await}
        _ => {
            Logging::ERROR.print_message("Unsupported protocol specified.");
            return;
        }
    }

}

async fn handle_arguments() -> Args{
    Args::parse()
}

async fn tcp_server(args: Args) {

    let c2 = Arc::new(
        Mutex::new(
            C2::create(args.log_file, None).unwrap()));

    let address = format!("{}:{}", args.host, args.port);


    let listener = TCPCommListener {
            bind_addr: address.parse().unwrap()
        };

    let c2_clone = Arc::clone(&c2);
    tokio::spawn(async move {
        if let Err(e) = listener.start(c2_clone).await {
            Logging::ERROR.print_message(&format!("Listener error: {}", e));
        }
    });

    let c2_sweep = Arc::clone(&c2);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(args.sweep)).await; // sweep every X min
            let mut c2 = c2_sweep.lock().await;
            Logging::DEBUG.print_message("Sweeping for dead agents");
            c2.sweep_dead_agents(120).await;  // FIX: timeout duration
        }
    });

    let mut cli = C2Cli { current_agent: 0 };

    cli.run(c2, &args.host, args.port, &args.protocol).await;
}

async fn http_server(args: Args) {
    let (log_tx, log_rx) = mpsc::channel::<LogEntry>(100);
    let app = Arc::new(Mutex::new(App::new(log_rx)));
    let log_tx_sweep = log_tx.clone();


    let c2 = Arc::new(
        Mutex::new(
            C2::create(args.log_file, None).unwrap()));

    let address = format!("{}:{}", args.host, args.port);
    
    //let app_http = Arc::clone(&app);
    let c2_http = Arc::clone(&c2);
    std::thread::spawn(move || {
        // Start a new Actix system
        actix_web::rt::System::new()
            .block_on(async move {
                if let Err(e) = http::run_server(
                    &address, c2_http, log_tx.clone(), false, None, None, false).await {
                    Logging::ERROR.print_message(&format!("HTTP server error: {}", e));
                }
            });
    });

    //let app_sweep = Arc::clone(&app);
    let c2_sweep = Arc::clone(&c2);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(args.sweep)).await; // sweep every X min
            let mut c2 = c2_sweep.lock().await;
            //let mut app = app_sweep.lock().await;
            //app.add_log(Logging::DEBUG, "Sweeping for dead agents".to_string());
            log_tx_sweep.send((Logging::DEBUG, "Sweeping for dead agents".into())).await.ok();
            c2.sweep_dead_agents(120).await;  // FIX: timeout duration
        }
    });

    let app_cli = Arc::clone(&app);
    app_cli.lock().await.c2_cli(c2, &args.host, args.port, &args.protocol).await;

}

async fn https_server(args: Args) {
    let (log_tx, log_rx) = mpsc::channel::<LogEntry>(100);
    let app = Arc::new(Mutex::new(App::new(log_rx)));
    let log_tx_sweep = log_tx.clone();

    let c2 = Arc::new(
        Mutex::new(
            C2::create(args.log_file, None).unwrap()));

    let address = format!("{}:{}", args.host, args.port);

    let cert_path = if args.cert.is_none() { None } else { args.cert.clone() };
    let key_path = if args.key.is_none() { None } else { args.key.clone() };

    let c2_https = Arc::clone(&c2);
    let address_clone = address.clone();
    std::thread::spawn(move || {
        // Start a new Actix system
        actix_web::rt::System::new()
            .block_on(async move {
                if let Err(e) = http::run_server(
                    &address_clone, c2_https, log_tx.clone(), true, cert_path, key_path, args.generate_certs).await {
                    Logging::ERROR.print_message(&format!("HTTPS server error: {}", e));
                }
            });
    });

    let c2_sweep = Arc::clone(&c2);
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(args.sweep)).await; // sweep every X min
            let mut c2 = c2_sweep.lock().await;
            // Logging::DEBUG.print_message("Sweeping for dead agents");
            log_tx_sweep.send((Logging::DEBUG, "Sweeping for dead agents".into())).await.ok();
            c2.sweep_dead_agents(120).await;  // FIX: timeout duration
        }
    });

    let app_cli = Arc::clone(&app);
    app_cli.lock().await.c2_cli(c2, &args.host, args.port, &args.protocol).await;
}

async fn dns_server() {
    todo!()
}

async fn multi_server() {
    todo!()
    /*
    FOR MULTIPLE LISTENERS
    let tcp_listener = TCPCommListener { ... };
    let http_listener = HTTPCommListener { ... };

    let tcp = tokio::spawn(tcp_listener.start(Arc::clone(&c2)));
    let http = tokio::spawn(http_listener.start(Arc::clone(&c2)));

    let _ = tokio::join!(tcp, http);
    */
}


