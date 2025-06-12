use std::net::SocketAddr;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use socket2::{Socket, Domain, Type, Protocol};
use std::sync::Arc;
use tokio::sync::Mutex;


use crate::server::listeners::listener_trait::Listener;
use crate::utils::logging::Logging;
use crate::core::c2::C2; 
use crate::server::communication::tcp_session;



pub struct TCPCommListener {
    pub bind_addr: SocketAddr
}


#[async_trait::async_trait]
impl Listener for TCPCommListener {
    async fn start(&self, c2: Arc<Mutex<C2>>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {

        // let listener = TcpListener::bind(&self.bind_addr).await?;
        let listener = create_listener(&self.bind_addr).await?;

        let bind_msg = format!("[+] TCP Listener started on {}", self.bind_addr);
        Logging::NETWORK.print_message(&bind_msg);

        loop {
            let (socket, addr) = listener.accept().await?;
            let connect_msg = format!("[*] Connection received from {}", addr);
            Logging::NETWORK.print_message(&connect_msg);

            let c2_clone = Arc::clone(&c2);
            // Spawn a new task to handle the connection
            tokio::spawn(async move {
                tcp_session::handle_session(socket, addr, c2_clone).await;
            });
        }
        
    }
}

pub async fn create_listener(bind_addr: &SocketAddr) -> std::io::Result<TcpListener> {
    // Create a socket using socket2
    let socket = Socket::new(Domain::for_address(*bind_addr), Type::STREAM, Some(Protocol::TCP))?;

    // Set reuse_addr and port
    socket.set_reuse_address(true)?;
    // socket.set_reuse_port(true)?;

    // Bind
    let address: socket2::SockAddr = (*bind_addr).into();
    socket.bind(&address)?;
    socket.listen(1024)?;

    // Convert to tokio TcpListener
    socket.set_nonblocking(true)?;
    let listener = TcpListener::from_std(socket.into())?;

    Ok(listener)
}


pub async fn _handle_connection(mut socket: TcpStream, addr: SocketAddr) {
    let mut buf = [0u8; 1024];

    match socket.read(&mut buf).await {
        Ok(n) if n == 0 => {
            println!("[-] Connection from {} closed", addr);
        }
        Ok(n) => {
            println!("[*] Received from {}: {}", addr, String::from_utf8_lossy(&buf[..n]));
            // Echo response
            let _ = socket.write_all(b"ACK\n").await;
        }
        Err(e) => {
            eprintln!("[!] Error reading from {}: {}", addr, e);
        }
    }
}


