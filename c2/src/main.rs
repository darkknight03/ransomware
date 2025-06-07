mod core;
mod server;
mod utils;
mod communication;
mod tasking;

use std::sync::Arc;
use server::listeners::listener_trait::Listener;
use tokio::sync::Mutex;
use clap::Parser;

use crate::utils::logging::Logging;
use crate::core::c2::C2; 
use crate::server::listeners::tcp::TCPCommListener;
use crate::core::cli::cli::C2Cli;

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

    // Path to SSL certificate (only used if protocol  is https)
    // #[arg(long, default_value = None)]
    // cert: String,

    // Path to SSL private key (only used if protocol is https)
    // #[arg(long, default_value = None)]
    // key: String,
}

#[tokio::main]
async fn main() {
    let args = handle_arguments().await;

    Logging::INFO.print_message("Starting up C2 server...");

    match args.protocol.as_str() {
        "tcp" => {tcp_server(args).await}
        "http" => {http_server().await}
        "https" => {https_server().await}
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

    cli.run(c2).await;
}

async fn http_server() {
    todo!()
}

async fn https_server() {
    todo!()
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


